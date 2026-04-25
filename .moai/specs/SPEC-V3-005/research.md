# SPEC-V3-005 Research — File Explorer Surface

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (orchestrator branch — 본 SPEC 산출물은 후속에 `feature/SPEC-V3-005-explorer` 로 분리됨)
선행: SPEC-V3-001 (셸 레이아웃), SPEC-V3-002 (Terminal Core), SPEC-V3-004 (Render Layer Integration — Entity 렌더 패턴 차용).
병행: SPEC-V3-008 (Git Management UI — git status 데이터 동기화 협의 대상).
범위: Sidebar 영역에 들어갈 파일 탐색기 (File Explorer / FileTree) 서피스의 데이터 모델, 렌더, 실시간 변경 반영, git status 표시, 컨텍스트 메뉴, 검색 기능을 정의한다.

---

## 1. 동기 — moai-studio v3 비전에서 File Explorer 의 위치

### 1.1 비전 맥락

`.moai/project/product.md` 와 `CLAUDE.local.md` §1 비전 문구는 moai-studio 가 4 대 surface 로 구성된 Agentic Coding IDE 임을 명시한다.

| Surface | 책임 | 선행 SPEC |
|---------|------|-----------|
| Terminal | Cmd+T / 분할 / 다중 탭 PTY | SPEC-V3-002, SPEC-V3-003, SPEC-V3-004 |
| **File Explorer** | 워크스페이스 트리 브라우징 + 실시간 변화 + git status | **본 SPEC (SPEC-V3-005)** |
| Editor / Markdown View | 코드 편집 + 라이브 미리보기 | (별도 SPEC, 미할당) |
| Git Management | branch / commit / hunk staging UI | SPEC-V3-008 |

본 SPEC 은 두 번째 surface 의 v1 단일 채택 결정이며, Terminal 의 `RootView::tab_container` 옆에 `RootView::file_explorer: Option<Entity<FileExplorer>>` 가 자리 잡는 형태가 된다.

### 1.2 사용자 가시 정의 (escape hatch)

본 SPEC 이 PASS 한 시점에 `cargo run -p moai-studio-app` 으로 다음을 직접 관찰할 수 있어야 한다:

1. Sidebar 영역 (또는 좌측 전용 패널) 에 활성 워크스페이스의 루트 트리가 표시된다.
2. 폴더 행을 클릭하면 자식이 lazy load 되어 펼쳐진다.
3. 외부 도구 (예: `touch a.txt` shell) 로 파일을 만들면 약 100~300ms 내에 트리에 반영된다.
4. 수정된 파일에 `M` 배지, 새 파일에 `A` 배지가 git status 컬럼에 보인다.
5. 폴더 행에 우클릭 시 New File / New Folder / Rename / Delete 메뉴가 등장한다.
6. 검색 박스에 문자열을 입력하면 트리가 fuzzy match 결과로 필터링된다.

### 1.3 이미 존재하는 자산과의 격차

| 자산 | 위치 | 상태 |
|------|------|------|
| `moai-fs::FsWatcher` | `crates/moai-fs/src/lib.rs` | ✅ notify 7 기반 단일 watcher (`Created`/`Modified`/`Removed` 추상화) |
| `moai-fs::FsEventBus` | `crates/moai-fs/src/watcher.rs` | ✅ workspace-키 기반 broadcast 채널 |
| `moai-fs::tree_watcher` | `crates/moai-fs/src/tree_watcher.rs` | ⚠️ 파일이 작음 — 확장 여지 검증 필요 |
| `moai-git::GitRepo::status_map()` | `crates/moai-git/src/lib.rs:79-` | ✅ `HashMap<String, String>` 반환, "M"/"A"/"D"/"??" 라벨 |
| `moai-studio-workspace::Workspace` | `crates/moai-studio-workspace/src/` | ✅ root path / id / display name 보유 |
| `moai-studio-ui::RootView::sidebar` | `crates/moai-studio-ui/src/lib.rs` | ✅ workspace 리스트만 — File Explorer 미수용 |
| **FsNode 도메인 모델** | — | ❌ 미존재 — 본 SPEC 이 도입 |
| **lazy load 알고리즘** | — | ❌ 미존재 |
| **debounce + diff apply** | — | ❌ 미존재 |
| **컨텍스트 메뉴 / drag-and-drop / 검색** | — | ❌ 미존재 |

격차는 명확히 4 갈래 — (a) 도메인 모델, (b) watcher 디바운싱 pipeline, (c) GPUI 렌더, (d) 인터랙션 (메뉴 / DnD / 검색).

---

## 2. 코드베이스 분석 — 기존 비편입 자산의 활용 전략

### 2.1 `moai-fs` 재사용 vs 확장

`crates/moai-fs/src/lib.rs:51-119` 의 `FsWatcher` 는 단일 워크스페이스를 감시한다 (`Recursive` 모드). 본 SPEC 은 워크스페이스 단위로 1 개의 `FsWatcher` 를 재사용하는 것을 기본 전략으로 한다.

이슈: 현 구현은 `Created/Modified/Removed` 만 노출하며 `Renamed` 가 없다. 파일 rename 은 `Removed(old) + Created(new)` 두 이벤트로 분해되어 들어온다. 본 SPEC 의 diff 엔진은 이 분해된 이벤트를 그대로 받되, debounce 윈도우 내에서 이름 매칭으로 rename 을 추정해 단일 트리 노드 이동으로 처리하는 옵션을 가진다 (RG-FE-2).

또 다른 이슈: 이벤트가 절대 경로로 들어온다. 트리 노드 식별자는 워크스페이스 루트 기준 상대 경로여야 한다. 변환 책임은 explorer 측 (`FileExplorer::on_fs_event`) 에 둔다.

### 2.2 `moai-git::status_map` 재사용

`crates/moai-git/src/lib.rs:79-110` 의 `GitRepo::status_map()` 는 `HashMap<String, String>` (key=상대경로, value="A"/"M"/"D") 를 반환한다.

호출 비용: `git2::Repository::statuses(None)` 는 worktree 전체 스캔이며 큰 모노레포 (10k+ 파일) 에서 100~500ms 수준 비용. 본 SPEC 은 이를 직접 호출하지 않고:

- 트리 expand 이벤트 시 한 번만 호출하여 캐시.
- `FsEvent::Modified/Created/Removed` 가 들어오면 해당 경로만 cherry-pick re-read (Git2 `status_file`) 하거나, 일정 디바운스 윈도우 내 이벤트가 누적되면 `status_map()` 전체 재호출.
- 명확한 게이트는 `RG-FE-3` AC 에서 정의.

### 2.3 SPEC-V3-008 (Git Management UI) 와의 경계

SPEC-V3-008 은 git branch / commit / hunk staging UI 를 다룬다. 본 SPEC 의 git status 는 단순 표시용 (M/A/D 배지) 이며 staging 동작은 포함하지 않는다. 즉 두 SPEC 의 공유 자원은 `GitRepo::status_map()` 호출자 다중화 정책뿐이다 — 추후 SPEC-V3-008 진행 시 둘이 같은 캐시를 공유하는 구조 (예: 별도 `GitStatusService` actor) 로 통합할 수 있도록, 본 SPEC 은 status_map 호출을 `FileExplorer` 의 단일 메서드 (`refresh_git_status`) 로 격리한다.

### 2.4 GPUI 렌더 패턴 — SPEC-V3-004 차용

SPEC-V3-004 가 도입한 "logic-only Pure Rust 모델 + 별도 `impl Render` GPUI Entity" 패턴을 그대로 차용한다:

- `panes::tree::PaneTree<L>` ↔ 본 SPEC 의 `explorer::tree::FsNode`
- `tabs::container::TabContainer` ↔ 본 SPEC 의 `explorer::view::FileExplorer`
- `panes::render::render_pane_tree` ↔ 본 SPEC 의 `explorer::render::render_fs_node`

이로써 (a) 단위 테스트가 GPUI 의존 없이 가능하고, (b) SPEC-V3-004 의 RG-R-2 / RG-R-3 와 같은 형태의 USER-DECISION (`gpui test-support` 채택) 를 본 SPEC 도 동일하게 적용할 수 있다.

### 2.5 Workspace 통합 지점

`crates/moai-studio-ui/src/lib.rs:72-99` 의 `RootView` 는 다음으로 확장된다:

```
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    pub tab_container: Option<Entity<tabs::TabContainer>>,
    // @MX:ANCHOR (예정): SPEC-V3-005 RG-FE-1
    pub file_explorer: Option<Entity<explorer::FileExplorer>>,
}
```

워크스페이스 활성 변경 시 `FileExplorer::set_workspace(new_root)` 호출로 트리 + watcher + git repo 가 재바인딩된다. 다중 워크스페이스 동시 표시는 비목표 (단일 활성).

---

## 3. 도메인 모델 후보 — `FsNode`

### 3.1 자료구조

```rust
pub enum FsNode {
    File { rel_path: PathBuf, name: String, git_status: GitStatus },
    Dir  { rel_path: PathBuf, name: String, children: ChildState, git_status: GitStatus, is_expanded: bool },
}

pub enum ChildState {
    NotLoaded,                  // depth-aware lazy load 의 핵심 상태
    Loading,                    // 비동기 read_dir 진행 중
    Loaded(Vec<FsNode>),
    Failed(FsError),            // 권한 오류 등
}

pub enum GitStatus { Clean, Modified, Added, Deleted, Untracked, Renamed, Conflicted }
```

`rel_path` 는 워크스페이스 루트 기준 상대 경로. 이로써 cross-platform path 정규화가 한 곳 (`FsNode` 생성 함수) 에 수렴된다.

### 3.2 lazy load 트리거

- Dir 의 `is_expanded` 가 `false → true` 전이 시 `ChildState::NotLoaded → Loading` 으로 전이하고 `tokio::spawn` 으로 `read_dir` 실행.
- 결과가 도착하면 `Loading → Loaded(children)` 로 swap 후 `cx.notify()`.
- 깊이 제한은 본 SPEC v1 에서는 없음 — 사용자가 펼칠 때마다 1 단계씩.

### 3.3 cross-platform 경로 처리

- macOS / Linux: UTF-8 PathBuf 를 그대로 String 변환 가능.
- Windows: `OsStr::to_string_lossy()` 를 사용하되 BOM 없는 UTF-16 케이스를 위해 `path.to_string_lossy().into_owned()` 로 안전하게 ratio. 화면 표기 시 `\` 가 아닌 `/` 로 치환 (정규화), git status 매칭 시에는 git2 가 `/` 를 표준으로 쓰므로 이 정규화가 필수.
- 정규화 함수는 `explorer::path::normalize_for_display(path: &Path) -> String` 한 함수로 격리 (RG-FE-1 의 cross-platform 요구).

---

## 4. notify watcher 의존성 결정

### 4.1 현재 상태

- `crates/moai-fs/Cargo.toml:8` → `notify = "7"` (이미 채택됨).
- 본 SPEC 의 사용자 프롬프트는 "notify 6 candidate (workspace 추가 후보)" 라고 기술하지만, 실제 코드베이스는 이미 `notify 7` 을 `moai-fs` 직접 의존성으로 보유한다. 따라서 **workspace 추가 결정은 불필요**하며, 본 SPEC 은 `moai-fs` 를 통해 간접 사용한다 (자체 notify 의존성 추가 없음).

### 4.2 USER-DECISION-A — moai-fs API 확장 vs File Explorer 자체 흡수

[USER-DECISION-REQUIRED: fs-watcher-api-shape-v3-005]

옵션 (a) **권장**: `moai-fs` 에 신규 `WorkspaceWatcher` 헬퍼 추가 (`subscribe(WorkspaceKey) -> Receiver<FsEvent>`). FileExplorer 는 단일 채널만 구독하면 된다. 비용: `moai-fs/src/lib.rs` 100~150 LOC 추가.
옵션 (b) `moai-fs::FsWatcher` 를 그대로 쓰고 FileExplorer 가 직접 wrap 한다. 비용: explorer crate 안에서 watcher 인스턴스 1 개를 lifecycle 관리. 단점: SPEC-V3-008 (Git UI) 가 같은 watcher 를 재사용하기 어렵다.

옵션 (a) 채택 시 본 SPEC 의 산출 파일 일부가 `moai-fs` crate 로 흘러간다. plan.md 의 Task 2 가 이 경로를 가정하고 작성된다.

### 4.3 USER-DECISION-B — gpui test-support feature 채택

SPEC-V3-004 와 동일한 게이트. 본 SPEC v1 에서는 RG-FE-1 / RG-FE-2 의 logic-level 검증이 GPUI 없이도 가능하므로 채택 여부와 무관하게 진행 가능. 단 RG-FE-4 (컨텍스트 메뉴) / RG-FE-5 (DnD) 의 e2e 검증은 채택 시 더 엄밀해진다.

---

## 5. 디바운스 정책 — 100ms 윈도우의 근거

### 5.1 왜 100ms 인가

- macOS FSEvents 코어센스: 트리거 후 OS-level 30~50ms 의 자체 coalescing.
- Linux inotify: 단발 이벤트가 잘게 쪼개져 들어옴 (`echo > a.txt` 하나에 `Create` + `Modify` + `CloseWrite` 3 발).
- Windows ReadDirectoryChangesW: 60~100ms 의 OS coalescing 후 유저 영역 도달.

따라서 user-space debounce 는 모든 플랫폼에서 100ms 가 유의미한 하한이며, 그 이상은 사용자 가시 지연을 만든다. 본 SPEC 은 100ms 를 디폴트로 두되 USER 가 설정 (`config.fs.debounce_ms`) 으로 50~500ms 범위를 조정 가능하게 한다.

### 5.2 debounce 알고리즘 (간소화 버전)

```
한 워크스페이스 안에서:
  WAITING 상태에서 첫 FsEvent 도착 → COLLECTING 진입, timer 시작 (100ms).
  COLLECTING 동안 추가 FsEvent 는 buffer 에 누적.
  timer 만료 시:
    - buffer 의 이벤트들을 (rel_path, kind) 로 dedupe.
    - rename 후보 매칭 (Removed(X) + Created(Y) 가 같은 윈도우에 있고 같은 부모 디렉토리 → Renamed(X→Y)).
    - 결과를 single FsDelta 로 트리에 apply.
    - WAITING 으로 복귀.
```

다중 워크스페이스 시 워크스페이스 키 별로 독립된 상태 머신.

### 5.3 backpressure

이벤트가 1 윈도우에 1k 건 이상 (예: `git checkout` 으로 5k 파일 swap) 들어오면 debounce 결과도 1k 건의 트리 mutate 가 된다. 이 경우 트리 전체 reload (`refresh_root`) 가 1k 건 individual apply 보다 빠르다 — 본 SPEC 은 debounce 결과 N >= 500 시 `refresh_root` 폴백으로 전환 (RG-FE-2 AC 에서 명시).

---

## 6. git status 통합 정책

### 6.1 컬럼 디자인

- 파일 행 우측 끝 (혹은 이름 옆) 에 1 글자 배지: M (yellow) / A (green) / D (red strikethrough) / U (gray) / R (cyan).
- 폴더 행은 자식 중 가장 "강한" 상태를 collapsed 상태에서 표시 (M > A > D > U). 이 우선순위는 `GitStatus::roll_up_priority` 로 단일 정의.

### 6.2 호출 빈도

- 워크스페이스 진입 시 1 회 `status_map()` 호출.
- FsEvent 디바운스 윈도우 만료 후 1 회 `status_map()` 재호출.
- 명시적 새로고침 (예: 사용자가 F5 누르면) 은 별도 RG-FE-3 AC 로.

### 6.3 SPEC-V3-008 와 합쳐질 때

본 SPEC 의 `refresh_git_status` 메서드는 `FileExplorer::set_git_status_provider(impl GitStatusProvider)` 로 추상화되어, 추후 SPEC-V3-008 가 동일 trait 의 다른 구현체 (캐시 + invalidation) 를 주입할 수 있도록 한다. 본 SPEC v1 의 디폴트 구현은 `GitRepo::status_map()` direct call.

---

## 7. 변경 금지 zone

- **SPEC-V3-002 terminal core**: `crates/moai-studio-terminal/**` 무변경. 본 SPEC 은 sidebar 만 만지고 content_area 는 SPEC-V3-004 의 `tab_container` 가 그대로 차지한다.
- **SPEC-V3-003 pane/tab logic**: 무변경. 본 SPEC 의 file explorer 는 sidebar 와 panes 본문 사이의 좌측 영역에 자리 잡는다.
- **SPEC-V3-004 render layer**: 변경 없음 — 본 SPEC 은 별도 `impl Render for FileExplorer` 를 추가하며 `render_pane_tree` 를 침범하지 않는다.
- **`moai-git::GitRepo::status_map`**: read-only 사용 — signature 변경 금지.

---

## 8. 위험 요약

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R1 | 큰 모노레포 (10k+ 파일) 에서 `status_map()` 100~500ms 비용 | UI freeze | 디바운스 만료 시 1 회만 호출, async tokio task 로 격리 |
| R2 | rename 이벤트가 Remove+Create 로 분해 | 트리에서 새로 만들고 옛 노드 삭제하면서 expand 상태 / 스크롤 위치 잃음 | debounce 윈도우 내 매칭으로 Renamed 합성 |
| R3 | Windows 경로 separator (`\` vs `/`) | git status 매칭 실패 | `normalize_for_display` 단일 함수로 정규화, git2 호출은 항상 `/` |
| R4 | 깊은 트리 (예: `node_modules` 5k 자식) lazy load 시 read_dir 단발 비용 | 펼침 시 spinner 표시 후 1~2 초 멈춤 | ChildState::Loading 상태로 명시적 progress 표기 + 자식 N >= 1k 시 가상화 (별도 SPEC, 본 SPEC v1 비목표) |
| R5 | notify가 native fs platform 이벤트를 일부 누락 (예: NFS / sshfs) | 사용자가 외부 변경을 안 봄 | 명시적 새로고침 키 (F5) 제공, 한계 사항 README 에 명시 |
| R6 | 컨텍스트 메뉴 의 delete 가 OS 휴지통 vs 영구 삭제 | 데이터 손실 | OS 휴지통 송부 (trash crate) 를 디폴트, USER-DECISION-C 에서 "휴지통 미사용 옵션" 차단 여부 결정 |
| R7 | DnD reorder 가 fs ordering (alphabetical) 를 무시한 사용자 의도 vs fs 의 자동 reorder 충돌 | 동작 혼란 | 본 SPEC v1 에서는 같은 부모 디렉토리 내의 file → file 이동은 fs rename 으로만 허용, 부모 변경 시 cross-dir move 로 처리. 사용자 정의 ordering 보존은 비목표 |
| R8 | gpui 0.2.2 가 wheel-event 컨텍스트 메뉴 / 우클릭 모델 미지원 가능성 | RG-FE-4 검증 불가 | Spike 0 (USER-DECISION-B 와 묶음) — 빌드 못하면 RG-FE-4 를 MS-3 carry-over 로 |

---

## 9. 의존 SPEC 정리

| SPEC | 관계 | 책임 분담 |
|------|------|-----------|
| SPEC-V3-001 | precedent | 셸 4 영역 레이아웃에 sidebar 가 이미 존재 — 본 SPEC 은 그 영역의 컨텐츠 모델을 정의 |
| SPEC-V3-002 | unchanged | terminal core 무변경 (R 제약) |
| SPEC-V3-003 | precedent | tabs/panes — 본 SPEC 과 직접 의존 없음 |
| SPEC-V3-004 | precedent | Render Entity 패턴 차용 (USER-DECISION-B 도 동일) |
| SPEC-V3-008 | concurrent | `GitStatusProvider` trait 으로 미래 통합 hook 만 둠. 현재는 direct call |

---

## 10. 영문 보조 요약

This document analyzes the gap between existing assets (`moai-fs::FsWatcher`, `moai-git::GitRepo::status_map`) and the unbuilt File Explorer surface. Four-part gap: (a) `FsNode` domain model, (b) debounced watcher pipeline, (c) GPUI Entity rendering, (d) interactions (context menu / DnD / search). Two USER-DECISION gates are surfaced: moai-fs API shape and gpui test-support adoption. Cross-platform path handling consolidates into a single `normalize_for_display` function. Dependency on SPEC-V3-004 is render-pattern-only; on SPEC-V3-008 is interface-only via `GitStatusProvider` trait.

---

작성 완료: 2026-04-25
다음 산출: spec.md (EARS 요구사항 + AC), plan.md (milestone × task).
