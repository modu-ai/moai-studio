---
id: SPEC-V3-005
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-004]
parallel_with: [SPEC-V3-008]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, file-explorer, fs-watch, git-status]
revision: v1.0.0 (initial draft, File Explorer surface v1, render pattern inherited from SPEC-V3-004)
---

# SPEC-V3-005: File Explorer Surface — FsNode 트리 + notify watch + git status 통합

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio v3 비전 4 대 surface 중 File Explorer 의 v1 정의. SPEC-V3-004 의 Render Entity 패턴 차용, `moai-fs::FsWatcher` (notify 7) 와 `moai-git::status_map` 재사용, cross-platform path 정규화 단일 진입점 도입. RG-FE-1 ~ RG-FE-6 의 6 개 요구사항 그룹과 AC-FE-1 ~ AC-FE-12 의 12 개 acceptance criteria 정의. USER-DECISION-A (moai-fs API shape), USER-DECISION-B (gpui test-support), USER-DECISION-C (delete 휴지통 정책) 명시. |

---

## 1. 개요

### 1.1 목적

moai-studio v3 비전의 4 대 surface 중 File Explorer 의 v1 단일 채택을 정의한다. 활성 워크스페이스의 디렉토리 트리를 sidebar 좌측 영역에 깊이별 lazy load 로 표시하고, `notify` 기반 실시간 변경 감지, git status 배지 표시, 우클릭 컨텍스트 메뉴, 트리 내 drag-and-drop reorder, fuzzy search 의 6 가지 능력을 제공한다.

본 SPEC 은 SPEC-V3-004 의 Render Entity 분리 패턴 — 즉 logic-only Pure Rust 모델 (`PaneTree<L>`) 과 GPUI 렌더 계층 (`impl Render for TabContainer`) 의 격리 — 를 그대로 차용한다. 이로써 `FsNode` / `FsTree` / `FsWatchPipeline` 은 GPUI 의존 없이 단위 테스트되고, `impl Render for FileExplorer` 가 GPUI 0.2.2 위에서 그것들을 배선한다.

### 1.2 v3 4 대 surface 와의 관계

| Surface | 책임 | 본 SPEC 와의 관계 |
|---------|------|------------------|
| Terminal | PTY + 다중 탭 + 분할 | SPEC-V3-002/003/004 — 본 SPEC 무관 (R 제약) |
| **File Explorer** | 워크스페이스 트리 + 변경 추적 + git 표시 | **본 SPEC** |
| Editor / Markdown | 파일 편집 + 라이브 미리보기 | 본 SPEC 의 "파일 행 클릭 → open" 이벤트 송출만 정의 (RG-FE-1 REQ-FE-005) |
| Git Management | branch / commit / hunk staging | SPEC-V3-008 — 본 SPEC 은 status read 만, write 없음 (R 제약) |

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-005/research.md` — 코드베이스 분석, FsNode 도메인 모델, debounce 정책, USER-DECISION 게이트, 위험 요약.
- `.moai/specs/SPEC-V3-004/spec.md` §6 — Render Entity 분리 패턴 (본 SPEC 차용).
- `crates/moai-fs/src/lib.rs` — FsWatcher (notify 7 기반).
- `crates/moai-fs/src/watcher.rs` — FsEventBus + WorkspaceEvent 타입.
- `crates/moai-git/src/lib.rs:79-110` — GitRepo::status_map() 시그니처.
- `crates/moai-studio-ui/src/lib.rs:72-99` — RootView 확장 진입점.
- `.moai/project/product.md`, `CLAUDE.local.md` §1 — v3 비전 4 surface 정의.

---

## 2. 배경 및 동기

본 섹션의 상세 분석은 `.moai/specs/SPEC-V3-005/research.md` §1 ~ §6 참조. 요구사항 진입 전 알아야 할 최소 맥락:

- **격차 분석** (research §1.3): `FsWatcher` / `status_map` / `Workspace` 타입은 모두 존재하지만, FsNode 도메인 모델, debounce pipeline, GPUI 렌더, 인터랙션은 미존재. 본 SPEC 이 4 갈래의 격차를 모두 수렴.
- **Render 패턴 차용** (research §2.4): SPEC-V3-004 의 logic ↔ render 분리를 그대로 채택. 단위 테스트는 GPUI 없이 가능.
- **debounce 100ms** (research §5): macOS / Linux / Windows 3 플랫폼의 OS-level coalescing 특성을 분석한 결과 100ms 가 사용자 가시 지연 없이 OS 노이즈를 잡아낼 수 있는 하한.
- **notify 의존성 결정** (research §4): `notify = "7"` 이 이미 `moai-fs/Cargo.toml:8` 에 직접 의존성으로 등록되어 있어 workspace 추가는 불필요. 본 SPEC 은 `moai-fs` 통과 간접 사용.
- **git status 통합** (research §6): `status_map()` 호출은 워크스페이스 진입 + debounce 만료 + 명시적 새로고침의 3 가지 trigger 로만 발생. SPEC-V3-008 미래 통합을 위한 `GitStatusProvider` trait 만 도입.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `FsNode` enum (File / Dir + ChildState NotLoaded/Loading/Loaded/Failed) 가 GPUI 의존 없이 정의되고 단위 테스트된다.
- G2. 활성 워크스페이스의 루트 디렉토리가 자동 표시되며, 폴더 행 클릭 시 자식이 lazy load 된다.
- G3. `moai-fs::FsWatcher` 이벤트가 100ms debounce 윈도우를 거쳐 트리에 diff apply 된다 (Created / Modified / Removed / Renamed).
- G4. 각 파일/폴더 행에 git status 배지 (M/A/D/U/R) 가 표시되며, 폴더는 자식 중 가장 강한 상태로 roll-up.
- G5. 폴더/파일 행 우클릭 시 컨텍스트 메뉴 (New File / New Folder / Rename / Delete / Reveal in Finder) 가 등장하며, 메뉴 액션이 fs 명령으로 실행된다.
- G6. 같은 트리 내에서 drag-and-drop reorder (cross-dir move) 가 지원된다.
- G7. 검색 박스에 입력하면 fuzzy match 로 트리 항목이 필터링되어 보인다.
- G8. macOS 14+, Ubuntu 22.04+, Windows 11 의 3 플랫폼에서 동일한 기능이 동작한다 (Windows 는 `cargo check` 수준만, 실제 e2e 검증은 별도 SPEC).
- G9. SPEC-V3-002 / SPEC-V3-003 / SPEC-V3-004 의 logic 공개 API 는 변경하지 않는다.
- G10. SPEC-V3-008 가 `GitStatusProvider` trait 의 다른 구현체를 주입할 수 있는 hook 이 마련된다.

### 3.2 비목표 (Non-Goals) / Exclusions

본 SPEC 이 의도적으로 만들지 않는 것 (Exclusions — at least one entry, [HARD] manager-spec rule):

- N1. **다중 워크스페이스 동시 표시**: sidebar 에 활성 워크스페이스 1 개의 트리만 표시. 다중 트리 분할 보기는 별도 SPEC.
- N2. **5k+ 자식 디렉토리 가상화**: 깊은 트리 (예: `node_modules`) 의 자식 가상 스크롤은 별도 SPEC. 본 SPEC v1 은 `read_dir` 결과를 전체 메모리 보유.
- N3. **사용자 정의 파일 ordering**: drag-and-drop 으로 fs 의 alphabetical ordering 을 override 하는 사용자 의도 보존은 비목표. cross-dir move 만 fs rename 으로 매핑.
- N4. **trash crate 의존성 추가 결정**: 본 SPEC v1 의 delete 는 OS 휴지통 송부를 기본 정책으로 하되, `trash` crate 채택 vs `std::fs::remove_dir_all` 직접 사용 vs 둘 다 옵션 제공의 결정은 USER-DECISION-C.
- N5. **git stage / commit / branch / hunk 표시**: SPEC-V3-008 의 책임. 본 SPEC 은 read-only 표시만.
- N6. **파일 미리보기 패널**: 파일 행 클릭 시 "open file" 이벤트를 송출만 하고, 실제 편집기 / markdown viewer 는 별도 SPEC.
- N7. **숨김 파일 정책**: `.git`, `node_modules`, `.DS_Store` 등의 숨김/제외 정책은 본 SPEC v1 에서 하드코딩된 디폴트 (사용자 설정 미지원). 사용자 정의 `.moai-ignore` 는 별도 SPEC.
- N8. **Windows e2e 통합 검증**: cross-platform path 정규화 함수의 단위 테스트는 본 SPEC 책임이지만, Windows GPUI 빌드 / 사용자 환경 e2e 는 SPEC-V3-002/003/004 와 동일하게 후속 SPEC.
- N9. **검색 결과 의 정렬 / 그루핑 / 미리보기 하이라이트**: 본 SPEC v1 의 검색은 단순 fuzzy match → 트리 visibility 토글. 결과 패널은 비목표.
- N10. **새 design token 추가**: SPEC-V3-001 / SPEC-V3-003 의 토큰 (`BG_SURFACE`, `FG_PRIMARY`, `ACCENT_MOAI`) 그대로 재사용. git status 배지 색만 새 const 로 추가하되 token 모듈 구조는 유지.

---

## 4. 사용자 스토리

- **US-FE1**: 개발자가 워크스페이스를 활성화하면 sidebar 좌측 영역에 그 워크스페이스의 루트 트리가 자동으로 표시된다 → RootView 가 `Entity<FileExplorer>` 를 보유하며 활성 워크스페이스의 root path 가 트리의 시작점이 된다.
- **US-FE2**: 개발자가 폴더 행을 클릭하면 자식 디렉토리가 펼쳐지고, 깊은 트리는 펼친 만큼만 메모리에 적재된다 → click → `FileExplorer::expand_dir(rel_path)` → `ChildState::NotLoaded → Loading → Loaded` 전이.
- **US-FE3**: 개발자가 외부 도구 (예: `touch a.txt` shell 또는 다른 IDE) 로 파일을 만들면 자동으로 트리에 새 행이 등장한다 → `FsWatcher` event → 100ms debounce → `FsTree::apply_delta` → `cx.notify()`.
- **US-FE4**: 개발자가 git 으로 변경한 파일을 보면 행 우측에 `M` 배지가 yellow 로 보인다 → debounce 만료 후 `GitStatusProvider::status_map()` 호출 → 트리에 배지 부착.
- **US-FE5**: 개발자가 폴더 행을 우클릭하면 메뉴가 등장하고, "New File" 을 선택하면 인라인 입력 박스가 나타나 새 파일 이름을 입력 후 Enter 로 확정한다 → context menu → `FileExplorer::start_inline_edit(InlineEditKind::NewFile, parent)` → fs.create.
- **US-FE6**: 개발자가 한 폴더의 파일을 다른 폴더로 drag-and-drop 하면 파일이 이동되고 트리가 새 위치를 반영한다 → drag start → drop on dir → `std::fs::rename(src, dst)` → FsWatcher event → trigger `apply_delta`.
- **US-FE7**: 개발자가 검색 박스에 "auth" 를 입력하면 트리에 "auth" 가 포함된 노드만 보인다 → input → fuzzy match → `FsNode::is_visible_under_filter` 갱신 → re-render.

---

## 5. 기능 요구사항 (EARS)

### RG-FE-1 — File system tree representation (FsNode)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-001 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/explorer/tree.rs` (신규) 에 `FsNode` enum 을 정의한다. 변형은 `File { rel_path, name, git_status }` 와 `Dir { rel_path, name, children: ChildState, git_status, is_expanded }` 두 가지다. 모든 필드는 GPUI 의존을 갖지 않는다. | The system **shall** define `FsNode` enum in `explorer/tree.rs` with `File` and `Dir` variants, GPUI-free. |
| REQ-FE-002 | Ubiquitous | 시스템은 `ChildState` enum 을 `NotLoaded`, `Loading`, `Loaded(Vec<FsNode>)`, `Failed(FsError)` 4 변형으로 정의한다. lazy load 의 진행 상태가 외부에서 관찰 가능해야 한다. | The system **shall** define `ChildState` with 4 variants representing lazy load lifecycle. |
| REQ-FE-003 | Event-Driven | 사용자가 `Dir` 노드를 펼치는 액션을 트리거할 때, 시스템은 (a) `is_expanded` 를 true 로 바꾸고 (b) `ChildState::NotLoaded` 였다면 `Loading` 으로 전이하며 비동기 read_dir 작업을 시작하고 (c) 결과 도착 시 `Loaded(children)` 로 swap 한다. | When the user expands a `Dir` node, the system **shall** transition `ChildState` from `NotLoaded` to `Loading` to `Loaded` as the async read_dir completes. |
| REQ-FE-004 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/explorer/path.rs` (신규) 에 `normalize_for_display(p: &Path) -> String` 함수를 제공한다. 이 함수는 macOS / Linux / Windows 3 플랫폼에서 워크스페이스 루트 기준 상대 경로를 항상 forward-slash (`/`) 로 정규화한 String 을 반환한다. | The system **shall** provide `normalize_for_display` in `explorer/path.rs`, normalizing paths to forward-slash on all platforms. |
| REQ-FE-005 | Event-Driven | 사용자가 `File` 노드를 클릭할 때, 시스템은 `FileExplorer::on_file_open` 콜백 (사용자 등록 가능) 을 통해 `(rel_path, abs_path)` 튜플을 송출한다. 본 SPEC 은 콜백의 인터페이스만 정의하며, 실제 편집기 통합은 별도 SPEC. | When the user clicks a `File`, the system **shall** invoke the registered `on_file_open` callback with `(rel_path, abs_path)`. |
| REQ-FE-006 | Unwanted | 시스템은 `read_dir` 에 대한 권한 오류 / I/O 오류가 발생해도 panic 하지 않는다. 오류는 `ChildState::Failed(FsError)` 로 흡수되며 트리 행에는 (예: 빨간 자물쇠 아이콘 + tooltip) 시각 표시가 주어진다. | The system **shall not** panic on read_dir errors; errors are absorbed into `ChildState::Failed` with a visual indicator. |

### RG-FE-2 — Real-time file watch (notify + 100ms debounce + diff apply)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-010 | Ubiquitous | 시스템은 활성 워크스페이스의 루트 경로에 대해 `moai-fs::FsWatcher` 인스턴스를 1 개 생성한다. 워크스페이스 전환 시 기존 watcher 는 drop 되고 새 watcher 가 새 루트로 시작된다. | The system **shall** create one `FsWatcher` instance per active workspace, recreating on workspace switch. |
| REQ-FE-011 | Event-Driven | `FsWatcher` 가 첫 `FsEvent` 를 송출할 때, 시스템은 100ms timer 를 시작하여 같은 윈도우 내 후속 이벤트를 buffer 에 누적한다. timer 만료 시 buffer 를 단일 `FsDelta` 로 변환하여 트리에 apply 한다. | When `FsWatcher` emits the first event, the system **shall** start a 100ms timer, buffer subsequent events, and apply as a single `FsDelta` on expiry. |
| REQ-FE-012 | Event-Driven | debounce 윈도우 내에 `FsEvent::Removed(X)` 와 `FsEvent::Created(Y)` 가 같은 부모 디렉토리에서 발생할 때, 시스템은 둘을 `FsDelta::Renamed { from: X, to: Y }` 로 매칭한다. 매칭 실패 시 두 개의 독립 delta 로 처리한다. | When `Removed(X)` and `Created(Y)` occur in the same parent within the debounce window, the system **shall** match them as `Renamed`. |
| REQ-FE-013 | State-Driven | debounce 결과 buffer 의 이벤트 수가 500 건 이상인 동안, 시스템은 individual apply 대신 `FsTree::refresh_root` (전체 reload) 폴백을 선택한다. 사용자에게는 "Reloading..." progress 가 표시된다. | While the debounce buffer has >= 500 events, the system **shall** fall back to `refresh_root` instead of individual diff apply. |
| REQ-FE-014 | Ubiquitous | 시스템은 debounce 윈도우 길이를 `.moai/config/sections/fs.yaml` 의 `debounce_ms` 키 (디폴트 100, 범위 50~500) 에서 읽는다. 설정 미존재 시 디폴트가 적용된다. | The system **shall** read debounce window length from `.moai/config/sections/fs.yaml` `debounce_ms` (default 100, range 50–500). |

### RG-FE-3 — Git status indicator

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-020 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/explorer/git_status.rs` (신규) 에 `GitStatus` enum (Clean / Modified / Added / Deleted / Untracked / Renamed / Conflicted) 과 `roll_up_priority(children: &[GitStatus]) -> GitStatus` 함수를 정의한다. 우선순위는 Conflicted > Modified > Added > Deleted > Renamed > Untracked > Clean. | The system **shall** define `GitStatus` enum and `roll_up_priority` with the specified priority order. |
| REQ-FE-021 | Ubiquitous | 시스템은 `GitStatusProvider` trait 을 정의한다. trait 은 `fn status_map(&self, repo_root: &Path) -> Result<HashMap<String, GitStatus>, GitStatusError>` 메서드를 포함한다. 디폴트 구현 (`MoaiGitStatusProvider`) 은 `moai_git::GitRepo::status_map()` 을 호출하여 String 라벨을 GitStatus enum 으로 매핑한다. | The system **shall** define `GitStatusProvider` trait and provide `MoaiGitStatusProvider` as default impl. |
| REQ-FE-022 | Event-Driven | (a) 워크스페이스 활성 변경 시, (b) debounce 윈도우 만료 시, (c) 사용자가 명시적 새로고침 키 (F5) 를 누를 때, 시스템은 `GitStatusProvider::status_map` 을 비동기 1 회 호출하여 결과를 `FsTree` 에 머지한다. 호출은 `tokio::spawn` 으로 격리되어 UI 스레드를 블록하지 않는다. | When (workspace switch / debounce expiry / F5), the system **shall** invoke `status_map` asynchronously and merge into `FsTree`. |
| REQ-FE-023 | Unwanted | 시스템은 `status_map` 호출이 실패해도 트리 렌더 자체는 계속 보여준다. 실패는 status bar 의 1 줄 메시지 + tracing warn 으로만 보고되며 트리 모든 노드의 git_status 는 `GitStatus::Clean` 으로 fallback. | The system **shall not** block tree rendering on `status_map` failure; failure is reported to status bar and tree falls back to `Clean`. |

### RG-FE-4 — Right-click context menu

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-030 | Event-Driven | 사용자가 `Dir` 행에서 우클릭할 때, 시스템은 `New File` / `New Folder` / `Rename` / `Delete` / `Reveal in Finder/Files` 5 개 메뉴 항목을 가진 popup 을 표시한다. | When the user right-clicks a `Dir`, the system **shall** show a popup with the 5 specified menu items. |
| REQ-FE-031 | Event-Driven | 사용자가 `File` 행에서 우클릭할 때, 시스템은 `Rename` / `Delete` / `Reveal in Finder/Files` 3 개 메뉴 항목을 가진 popup 을 표시한다 (New File/Folder 는 dir 전용). | When the user right-clicks a `File`, the system **shall** show a popup with the 3 specified menu items. |
| REQ-FE-032 | Event-Driven | 사용자가 `New File` 또는 `New Folder` 를 선택할 때, 시스템은 해당 부모 디렉토리 아래에 인라인 입력 행을 추가하고 입력 박스에 포커스를 맞춘다. Enter 입력 시 fs 에 생성을 시도하고, Esc 입력 시 입력을 취소한다. | When the user selects `New File`/`New Folder`, the system **shall** insert an inline edit row, focus the input, and on Enter create the entity (or cancel on Esc). |
| REQ-FE-033 | Event-Driven | 사용자가 `Rename` 을 선택할 때, 시스템은 해당 행의 이름 텍스트를 인라인 입력 박스로 swap 하고 기존 이름을 select all 상태로 둔다. Enter 입력 시 `std::fs::rename` 으로 변경하고, Esc 입력 시 취소한다. | When the user selects `Rename`, the system **shall** swap the name text to an inline input pre-selected, and on Enter call `fs::rename`. |
| REQ-FE-034 | Event-Driven | 사용자가 `Delete` 를 선택할 때, 시스템은 confirmation modal 을 표시하고 (USER-DECISION-C 결과에 따라) (a) `trash` crate 으로 OS 휴지통에 송부, 또는 (b) `std::fs::remove_dir_all` / `remove_file` 로 영구 삭제한다. | When the user selects `Delete`, the system **shall** show a confirmation modal and dispatch (per USER-DECISION-C). |
| REQ-FE-035 | Unwanted | 시스템은 컨텍스트 메뉴의 모든 fs 액션 (생성/이름변경/삭제) 이 fs 오류로 실패할 때 panic 하지 않는다. 오류는 status bar + tracing error 로 보고된다. | The system **shall not** panic on context menu fs action failure; errors are reported. |

### RG-FE-5 — Drag-and-drop reorder (cross-dir move)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-040 | Event-Driven | 사용자가 `File` 또는 `Dir` 행을 마우스로 drag 하기 시작할 때, 시스템은 그 행의 `rel_path` 를 drag payload 로 보유하고 시각적 drag preview 를 표시한다. | When the user starts dragging a row, the system **shall** capture the `rel_path` and show a drag preview. |
| REQ-FE-041 | Event-Driven | 사용자가 `Dir` 행 위에 drop 할 때 (drag source 와 다른 경로일 때), 시스템은 `std::fs::rename(src, dst_dir.join(name))` 을 실행한다. 결과 fs 변경은 watcher 의 debounce 윈도우를 통해 트리에 자연스럽게 반영된다. | When the user drops on a `Dir` (different path), the system **shall** invoke `fs::rename` to move into the target directory. |
| REQ-FE-042 | Unwanted | 시스템은 drop target 이 source 의 자기 자신 또는 source 의 하위 디렉토리일 때 fs 호출을 건너뛴다. 사용자에게는 status bar 1 줄 메시지로 거부 사실이 보고된다. | The system **shall not** invoke `fs::rename` if the drop target is the source itself or a descendant of source. |
| REQ-FE-043 | Unwanted | 시스템은 drop 으로 인한 fs::rename 이 실패해도 (예: cross-device link) panic 하지 않는다. 실패 시 status bar + tracing warn 으로 보고하며 트리는 변경 없이 유지된다. | The system **shall not** panic on `fs::rename` failure; the error is reported and the tree is left unchanged. |

### RG-FE-6 — Search / fuzzy filter

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FE-050 | Ubiquitous | 시스템은 sidebar 의 트리 상단에 검색 input 을 배치한다. input 은 빈 상태일 때 placeholder "Search files..." 를 표시한다. | The system **shall** place a search input at the top of the sidebar tree, with placeholder "Search files...". |
| REQ-FE-051 | Event-Driven | 사용자가 검색 input 에 문자열을 입력할 때, 시스템은 trim 된 query 를 `FsTree::apply_filter(query)` 에 전달하여 각 노드의 `is_visible_under_filter` 플래그를 fuzzy match (case-insensitive subsequence match) 로 갱신한다. | When the user types in search input, the system **shall** apply fuzzy match to update each node's visibility flag. |
| REQ-FE-052 | State-Driven | 검색 query 가 비어 있는 동안, 시스템은 모든 노드의 visibility 를 true 로 복원하여 평소 트리 표시를 유지한다. | While the search query is empty, the system **shall** restore visibility for all nodes. |

---

## 6. 비기능 요구사항

### 6.1 성능

- P1. 워크스페이스 루트 read_dir (1k 자식 미만) 는 100ms 내 완료 — macOS local SSD 기준.
- P2. debounce 윈도우 만료 후 트리 mutation + cx.notify() 까지 16ms 미만 — 60Hz 프레임 보존.
- P3. 5k 파일 모노레포의 `status_map()` 호출은 500ms 미만 (git2 자체 비용에 의존, 본 SPEC 의 책임은 호출 빈도 제어만).
- P4. 5k 파일 트리에서 fuzzy filter 적용은 50ms 미만.

### 6.2 보안

- S1. fs 액션 (생성/삭제/이름변경/이동) 은 절대 워크스페이스 루트 외부로 나가지 않는다 — `rel_path` 가 `..` 로 escape 시도하면 거부.
- S2. context menu 의 inline edit input 에서 OS path separator (`/` 또는 `\`) 를 포함한 이름은 거부.

### 6.3 cross-platform

- C1. `normalize_for_display` 는 macOS / Linux / Windows 3 플랫폼에서 동일한 결과 (forward-slash 정규화) 를 산출 — 단위 테스트로 검증 (AC-FE-3).
- C2. Windows 빌드는 본 SPEC v1 에서 `cargo check` 까지만 검증, GPUI 환경 e2e 는 별도 SPEC.

### 6.4 변경 금지 zone (R 제약)

- R1. `crates/moai-studio-terminal/**` 무변경 (SPEC-V3-002 carry).
- R2. `crates/moai-studio-ui/src/panes/**`, `crates/moai-studio-ui/src/tabs/**` 의 logic 공개 API 무변경 (SPEC-V3-003 carry).
- R3. `crates/moai-studio-ui/src/lib.rs` 의 `RootView` 는 새 필드 추가만 허용 (`file_explorer: Option<Entity<...>>`), 기존 필드 rename / delete 금지.
- R4. `moai-git::GitRepo::status_map` 시그니처 변경 금지 (read-only 사용).
- R5. `moai-fs::FsWatcher` 시그니처 변경 금지 단, USER-DECISION-A (a) 채택 시 `moai-fs` 에 신규 helper 추가는 허용.

---

## 7. USER-DECISION 게이트

### 7.1 USER-DECISION-A — moai-fs API shape

[USER-DECISION-REQUIRED: fs-watcher-api-shape-v3-005]

질문:
- (a) **권장**: `moai-fs` 에 `WorkspaceWatcher` helper 추가 — `subscribe(WorkspaceKey) -> tokio::sync::mpsc::Receiver<FsDelta>` 인터페이스 노출. FileExplorer / SPEC-V3-008 모두 같은 helper 구독 가능.
- (b) `moai-fs::FsWatcher` 를 그대로 쓰고 explorer crate 가 자체 wrap. SPEC-V3-008 와 watcher 인스턴스 공유 어려움.

영향: (a) 채택 시 `moai-fs/src/workspace_watcher.rs` 신규 (~150 LOC). plan.md MS-1 T3 가 이 경로를 가정하고 작성됨.

### 7.2 USER-DECISION-B — gpui test-support feature 채택 (SPEC-V3-004 carry-over)

질문 (AskUserQuestion):
- (a) **권장**: `gpui` dev-dependencies 에 `features = ["test-support"]` 추가. AC-FE-4 / AC-FE-7 등 GPUI 환경 e2e 가 실제 GPUI 기반에서 검증된다.
- (b) 추가하지 않음. logic-level fallback 으로 진행. 우회 코드 ~80 LOC.

영향: SPEC-V3-004 의 동일 게이트 결정과 일관성 권장.

### 7.3 USER-DECISION-C — delete 시 OS 휴지통 정책

[USER-DECISION-REQUIRED: delete-trash-policy-v3-005]

질문:
- (a) **권장**: `trash = "5"` crate 추가 → 모든 delete 가 OS 휴지통 송부. 비가역 삭제 방지.
- (b) `std::fs::remove_*` 직접 사용 → 영구 삭제. 의존성 추가 없음.
- (c) 둘 다 옵션으로 제공 → confirmation modal 에 "Move to Trash" / "Delete Permanently" 두 버튼. 비용 = (a) + UI 분기.

영향: REQ-FE-034 의 디스패치 분기. plan.md MS-3 T9 가 결정 결과를 반영.

---

## 8. Acceptance Criteria

본 SPEC 의 acceptance.md 는 본 spec.md 의 8 개 AC 를 단일 출처로 한다 (별도 acceptance.md 파일 미생성, AC 는 본 섹션 + plan.md 의 T별 매핑으로 충분). 본 SPEC 은 manager-spec 의 3-file 표준 (spec.md / plan.md / research.md) 을 따르며, AC 는 spec.md §8 단일 진실원칙으로 통합 — SPEC-V3-004 와 동일한 형식.

| AC ID | 검증 대상 | 검증 방법 | Definition of Done |
|-------|-----------|-----------|-------------------|
| AC-FE-1 | RG-FE-1 REQ-FE-001/002 | 단위 테스트: `FsNode::new_dir`, `FsNode::new_file`, `ChildState` 모든 변형 생성 | `cargo test -p moai-studio-ui explorer::tree::tests` PASS, GPUI 의존 없이 빌드 |
| AC-FE-2 | RG-FE-1 REQ-FE-003 | 단위 테스트: `expand_dir(rel_path)` 호출 → `is_expanded == true`, `ChildState::Loading` 전이 / mock async 완료 후 `Loaded(children)` 검증 | 위 두 시나리오 단위 테스트 PASS |
| AC-FE-3 | RG-FE-1 REQ-FE-004 (cross-platform) | 단위 테스트: `normalize_for_display` 가 macOS/Linux/Windows 입력 (각 cfg-gated) 에서 동일한 forward-slash 출력 | `cargo test -p moai-studio-ui explorer::path::tests` PASS, Windows runner CI 에서 별도 검증 |
| AC-FE-4 | RG-FE-1 REQ-FE-005 (file open 콜백) | integration: `cx.new(\|cx\| FileExplorer::new())` 후 callback 등록 + click 이벤트 시뮬레이션 → callback 1 회 호출 검증 | USER-DECISION-B (a) 채택 시 GPUI 환경 e2e, (b) 채택 시 logic-level fallback unit |
| AC-FE-5 | RG-FE-2 REQ-FE-010/011 (debounce 100ms) | 단위 테스트: mock FsWatcher 가 50ms 간격으로 5 개 이벤트 송출 → 100ms 후 단일 `FsDelta::Batch(5)` 가 트리에 apply | `cargo test -p moai-studio-ui explorer::watch::tests::test_debounce_coalesce` PASS |
| AC-FE-6 | RG-FE-2 REQ-FE-012 (rename 매칭) | 단위 테스트: `Removed("foo/a.txt")` + `Created("foo/b.txt")` 같은 윈도우 → `FsDelta::Renamed { from, to }` 1 건으로 매칭 | 단위 테스트 PASS |
| AC-FE-7 | RG-FE-2 REQ-FE-013 (대량 backpressure) | 단위 테스트: mock 600 이벤트 송출 → individual apply 대신 `refresh_root` 폴백 호출 검증 | 단위 테스트 PASS |
| AC-FE-8 | RG-FE-3 REQ-FE-021/022 (git status) | 단위 테스트: tempdir 에 git init + 파일 추가 → `MoaiGitStatusProvider::status_map` 호출 → `GitStatus::Untracked` enum 매핑 검증; `roll_up_priority([Modified, Untracked, Clean]) == Modified` | `cargo test -p moai-studio-ui explorer::git_status::tests` PASS |
| AC-FE-9 | RG-FE-4 REQ-FE-030~033 (context menu) | integration: 우클릭 시뮬레이션 → 메뉴 5/3 항목 구성 검증; New File 선택 → 인라인 입력 → Enter → tempdir 에 파일 생성 검증 | USER-DECISION-B 결과에 따라 GPUI e2e 또는 logic-level fallback PASS |
| AC-FE-10 | RG-FE-4 REQ-FE-034 (delete dispatch) | 단위 테스트: USER-DECISION-C 결정 결과 분기 → (a) trash crate mock 호출 검증, (b) `fs::remove_*` 호출 검증, (c) 분기 모두 검증 | 결정에 따른 분기 단위 테스트 PASS |
| AC-FE-11 | RG-FE-5 REQ-FE-040~042 (DnD) | integration: drag(file) → drop(dir) → fs::rename 호출 검증; drop on self/descendant → 거부 + status bar 메시지 검증 | USER-DECISION-B 결과에 따라 GPUI e2e 또는 logic-level fallback PASS |
| AC-FE-12 | RG-FE-6 REQ-FE-050~052 (search/filter) | 단위 테스트: 트리에 ["src/main.rs", "src/auth/mod.rs", "tests/auth_test.rs"] → query "auth" → visibility 갱신 → 2 노드 visible, "main" 1 노드 visible / query "" → 모두 visible | `cargo test -p moai-studio-ui explorer::search::tests` PASS |

각 AC 의 PASS 조건은 `cargo test -p moai-studio-ui --lib`, `cargo test -p moai-studio-ui --test integration_explorer` (신규), `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` 의 4 게이트 모두 GREEN.

---

## 9. 의존 SPEC 정리

| SPEC | 관계 | 차용 / 분담 |
|------|------|-------------|
| SPEC-V3-001 | precedent | 셸 4 영역 (TitleBar / Sidebar / Body / StatusBar) 기존 — 본 SPEC 이 sidebar 컨텐츠 내용 정의 |
| SPEC-V3-002 | unchanged | terminal core (R1 제약) |
| SPEC-V3-003 | unchanged | tabs/panes logic 공개 API (R2 제약) |
| SPEC-V3-004 | precedent | Render Entity 분리 패턴 차용; USER-DECISION-B (gpui test-support) 동일 게이트 일관성 권장 |
| SPEC-V3-008 | concurrent (parallel) | `GitStatusProvider` trait 으로 미래 통합 hook. 현재는 본 SPEC 의 `MoaiGitStatusProvider` 디폴트 구현. SPEC-V3-008 진행 시 캐시/invalidation 가진 별도 구현체 주입 가능 |

---

## 10. Milestone 매핑 (plan.md 와 동기)

| Milestone | 핵심 산출 | 검증 AC |
|-----------|-----------|---------|
| MS-1 | FsNode + ChildState + lazy load + path::normalize_for_display + 기본 GPUI render (placeholder content) | AC-FE-1, AC-FE-2, AC-FE-3, AC-FE-4 |
| MS-2 | moai-fs WorkspaceWatcher (USER-DECISION-A) + 100ms debounce + diff apply + rename 매칭 + backpressure 폴백 | AC-FE-5, AC-FE-6, AC-FE-7 |
| MS-3 | git status (GitStatusProvider) + context menu + DnD + search + e2e 통합 | AC-FE-8, AC-FE-9, AC-FE-10, AC-FE-11, AC-FE-12 |

자세한 task 분해는 plan.md 참조.

---

## 11. 위험 / 미해결 항목

- **Spike 0** (gpui test-support): SPEC-V3-004 와 동일하게 빌드 검증 필요. 빌드 실패 시 RG-FE-4 / RG-FE-5 의 e2e AC 는 logic-level fallback.
- **Spike 1** (trash crate): USER-DECISION-C (a) 또는 (c) 채택 시 빌드 검증.
- **`tree_watcher.rs` 검토**: `crates/moai-fs/src/tree_watcher.rs` 가 작은 파일 — USER-DECISION-A (a) 채택 시 이 파일을 확장하거나 `workspace_watcher.rs` 로 분리할지 결정 (T3 책임).
- **검색 결과 ordering**: 단순 fuzzy match score 만으로 정렬할지, 깊이 우선 정렬할지 v1 에서 명시 안 함 → MS-3 T7 진행 시 구현 자유 (단위 테스트 시 stable ordering 만 검증).

---

## 12. 영문 보조 요약 (Executive Summary)

SPEC-V3-005 defines the v1 File Explorer surface for moai-studio v3. It introduces an `FsNode` enum with `File`/`Dir` variants and lazy-loaded `ChildState`, a 100ms debounced pipeline on top of `moai-fs::FsWatcher` (notify 7), git status integration via a new `GitStatusProvider` trait (default impl wraps `moai-git::status_map`), right-click context menus (New/Rename/Delete/Reveal), drag-and-drop cross-directory move, and fuzzy search. The render layer follows SPEC-V3-004's logic-render separation pattern. Three USER-DECISION gates: moai-fs API shape, gpui test-support adoption, and delete trash policy. Cross-platform path normalization is consolidated into `explorer/path.rs::normalize_for_display`. 6 EARS requirement groups, 12 acceptance criteria, 3 milestones. Excludes multi-workspace simultaneous view, virtualization for 5k+ children, custom file ordering, git staging, file preview, hidden file customization, Windows e2e, and new design tokens.

---

작성 완료: 2026-04-25
다음 산출: plan.md (milestone × task × file × AC 매핑).
