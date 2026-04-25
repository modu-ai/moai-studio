# SPEC-V3-008 Research — Git Management UI

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (작업 중인 carry branch — 정식 v0.1.0 이전 임시)
선행: SPEC-V3-004 Render Layer (진행 중), SPEC-V3-006 Markdown/Code Viewer (선행, syntax-highlight 의존), SPEC-V3-005 File Explorer (병행).
범위: moai-studio 의 Git status / diff / commit / branch / log / merge UI 통합. 기존 `moai-git` crate 를 fan-in 1 → 다(多) 로 확장하여 GPUI render layer 와 직접 결합.

---

## 1. 동기 — Git UI 가 v0.1.0 정식 릴리스의 비전 핵심인 이유

### 1.1 moai-studio 비전 위치

CLAUDE.local.md §8 (현재 v0.0.x pre-release) 와 SPEC-V3-001 ~ SPEC-V3-004 의 누적 결과를 종합하면, v0.1.0 의 사용자 가시 경험은 다음 4 축으로 수렴한다:

1. **Pane/Tab 셸** (SPEC-V3-002/003/004) — 다중 탭, 분할, 리사이즈, 영속화.
2. **Editor + Viewer** (SPEC-V3-005 Explorer / SPEC-V3-006 Markdown+Code) — 파일 탐색 + 미리보기.
3. **Git 통합** (SPEC-V3-008, 본 SPEC) — status / diff / commit / branch / log / merge.
4. **AI 통합** (SPEC-M3-001 등 별도 라인) — Claude Code subprocess hook.

이 중 (3) Git 통합은 "터미널만으로 git 을 다루는 외부 IDE 와 차별화하는 단일 가시 가치" 다. moai-studio 가 "git-aware terminal IDE" 로 자리잡으려면 Status Panel / Diff Viewer / Commit Composer 가 필수다.

### 1.2 기존 moai-git 의 fan_in 폭발 신호

`crates/moai-git/src/lib.rs:36-145` 의 현재 공개 API:

| 메서드 | 현재 caller | SPEC-V3-008 도입 후 caller (예상) |
|--------|-------------|----------------------------------|
| `GitRepo::open` | moai-studio-workspace::persistence | + status_panel, diff_viewer, commit_composer, branch_switcher, log_view, merge_resolver (총 7) |
| `GitRepo::current_branch` | (없음) | + branch_switcher, status_panel header, log_view filter (총 3) |
| `GitRepo::is_dirty` | moai-studio-workspace | + commit_composer (Commit 버튼 활성/비활성), status_panel header (총 3) |
| `GitRepo::status_map` | (없음) | + status_panel (메인 소비자), file_explorer overlay (badge), log_view (working tree marker) (총 3) |
| `GitRepo::status_summary` | moai-studio-workspace::summary | + title bar Git status 위젯 (총 2) |

`status_map` 는 본 SPEC 에서 fan_in ≥ 3 의 새 ANCHOR 가 된다 (mx-tag-protocol §"@MX:ANCHOR" 트리거). 이는 본 SPEC 이 단순 신규 crate 가 아니라 **moai-git 의 공개 API 를 안정화 (stabilize)** 시키는 SPEC 임을 의미한다.

### 1.3 사용자 가시 정의 (escape hatch)

본 SPEC 이 PASS 한 시점에 사용자가 `cargo run -p moai-studio-app` 으로 다음을 직접 관찰할 수 있어야 한다:

1. 좌측 사이드바 (또는 별도 Git 탭) 에 **Status Panel** 이 보이고 staged / unstaged / untracked 섹션으로 구분된 파일 리스트가 등장한다.
2. 파일을 클릭하면 본문 영역에 **Diff Viewer** 가 열려 변경된 줄이 syntax-highlighted 로 표시된다.
3. **Commit Composer** 에 메시지를 입력하고 Cmd/Ctrl+Enter 로 커밋하면 staged 파일이 비워지고 새 HEAD 가 표시된다.
4. **Branch Switcher** 에서 브랜치를 전환하면 status / diff / commit 위젯이 즉시 새 HEAD 기준으로 갱신된다.
5. **Log View** 에서 commit graph 와 메시지가 시간순으로 표시되고, 클릭 시 해당 커밋의 diff 가 본문에 열린다.
6. **Merge Conflict Resolver** 가 conflict 상태에서 자동으로 활성화되어 ours / theirs / merged 3-way diff 를 표시한다.
7. **Stash Management** 에서 stash list 를 보고 push / pop / drop 액션을 키보드 또는 클릭으로 실행한다.

---

## 2. moai-git crate API surface 조사

### 2.1 현재 제공되는 것 (lib.rs)

```rust
pub struct GitRepo { inner: git2::Repository }

impl GitRepo {
    pub fn open(path: &Path) -> Result<Self, GitError>;
    pub fn init(path: &Path) -> Result<Self, GitError>;
    pub fn current_branch(&self) -> Result<String, GitError>;
    pub fn is_dirty(&self) -> Result<bool, GitError>;
    pub fn status_map(&self) -> Result<HashMap<String, String>, GitError>;
    pub fn status_summary(&self) -> Result<GitStatus, GitError>;
}

pub struct GitStatus { pub modified, pub added, pub deleted: usize }

#[derive(Error)]
pub enum GitError {
    Git(#[from] git2::Error),
    DetachedHead,
}
```

핵심 관찰:
- **git2 = 0.20** 에 이미 lock-in. libgit2 binding 사용 중. 본 SPEC 도 이를 기본 채택.
- `status_map` 의 값 enum 은 문자열 (`"clean" | "modified" | "added" | "untracked" | "deleted"`). 본 SPEC 에서 typed enum 도입 검토 (RG-G-1 후보).
- `Result<_, GitError>` 통일된 오류 채널. 본 SPEC 에서 `BranchSwitchFailed`, `CommitFailed`, `MergeConflictUnresolved` 등 enum variant 추가 필요.

### 2.2 빠진 것 (본 SPEC 에서 추가)

| 신규 메서드 | 책임 | 호출자 |
|-------------|------|--------|
| `GitRepo::diff_file(path) -> Result<Diff, GitError>` | 단일 파일 unified diff | diff_viewer |
| `GitRepo::diff_workdir() -> Result<Diff, GitError>` | working tree 전체 diff | (status panel summary) |
| `GitRepo::stage(path) / unstage(path)` | index manipulation | status_panel |
| `GitRepo::commit(msg, author, email) -> Result<Oid, GitError>` | 커밋 생성 | commit_composer |
| `GitRepo::list_branches() -> Result<Vec<BranchInfo>, GitError>` | 로컬 + 원격 브랜치 | branch_switcher |
| `GitRepo::create_branch(name, from) / switch_branch(name)` | 브랜치 조작 | branch_switcher |
| `GitRepo::log(limit) -> Result<Vec<CommitInfo>, GitError>` | commit history | log_view |
| `GitRepo::merge(branch) -> Result<MergeResult, GitError>` | 머지 시도 | merge_resolver |
| `GitRepo::conflict_files() -> Result<Vec<ConflictFile>, GitError>` | conflict 상태 파일 | merge_resolver |
| `GitRepo::stash_push(msg) / stash_pop(idx) / stash_list()` | stash 관리 | stash_panel |

본 SPEC 의 MS-1 ~ MS-3 가 위 표를 모두 커버한다.

### 2.3 git2-rs 의 한계 — gix 비교

| 기준 | git2-rs (libgit2 0.20) | gix (gitoxide pure-rust) |
|------|------------------------|--------------------------|
| 빌드 의존성 | C 라이브러리 (libgit2) FFI, OpenSSL/SSH 정적 링크 가능 | 순수 Rust, FFI 0 |
| 머지 지원 | 완전 (libgit2 의 merge.c) | partial (gix 0.66 기준 — fast-forward + 3-way diff 일부) |
| 성능 (status, diff) | C 최적화, 대형 repo 빠름 | parallelism 우수, 일부 워크로드 더 빠름 |
| Windows 빌드 | OpenSSL 등 의존성 셋업 필요 | 순수 Rust, 더 단순 |
| API 안정성 | 0.20 (안정) | 0.66+ (active 변화) |
| moai-git 현황 | **이미 채택** | 미사용 |

**결정안**: 본 SPEC v1.0.0 은 **git2-rs 유지**. gix 전환은 별도 SPEC (v0.2.0+ 비전). 이유:
1. `moai-studio-workspace` 가 이미 git2 0.20 으로 production 검증됨 (SPEC-V3-001 SqliteWorkspaceStore 와 함께 빌드 검증).
2. merge resolver 는 libgit2 의 성숙도 필요.
3. gix 의 active 변화는 SPEC churn 위험 (RG-G-7 carry).

USER-DECISION 게이트로 명시화 (UD-1 in plan.md §2).

---

## 3. UI 영역 분석 — 어디에 무엇을 그릴 것인가

### 3.1 SPEC-V3-001/004 RootView 와의 통합 지점

`crates/moai-studio-ui/src/lib.rs:72-202` 의 RootView 는 SPEC-V3-004 진행 중 다음 구조로 진화한다:

```
RootView
├── title_bar
├── main_body
│   ├── sidebar (← Git Status Panel 가 여기 또는 별도 영역)
│   └── content_area
│       └── tab_container (Entity<TabContainer>)
│           ├── TabBar
│           └── PaneTree<Entity<...>>  ← Diff Viewer 여기
└── status_bar (← Branch + dirty 표시 위젯)
```

본 SPEC 의 통합 옵션:

| 옵션 | 설명 | 트레이드오프 |
|------|------|-------------|
| **A. Sidebar 확장** | sidebar 에 "Files / Git / Search" 같은 탭 형식 추가 | RootView 변경 최소, but sidebar 가 비좁아질 위험 |
| **B. 별도 Git Tab** | TabContainer 의 한 탭이 "Git" 으로 예약, PaneTree 에 status / diff / log 분할 | RootView 무변경, but 사용자 컨텍스트 (탭 1 = repo overview) 학습 필요 |
| **C. Hybrid** | status panel 만 sidebar, diff viewer 는 활성 탭의 leaf, log 는 별도 탭 | 경험적으로 최적 (VS Code, Sublime Merge 패턴) |

**결정안**: **C. Hybrid** 채택 (USER-DECISION UD-3 in plan.md §4 로 명시화). MS-1 의 Status Panel 은 sidebar, MS-2 의 Diff Viewer 는 leaf payload, MS-3 의 Log View 는 별도 탭.

### 3.2 SPEC-V3-006 Markdown/Code Viewer 와의 의존

Diff Viewer (RG-G-2) 는 변경된 라인을 syntax-highlighted 로 표시한다. SPEC-V3-006 가 syntect (또는 tree-sitter) 기반 highlighter 모듈을 제공할 예정이며, 본 SPEC 은 그 모듈의 공개 API `highlight_line(line: &str, lang: &str) -> Vec<HighlightSpan>` 를 소비한다.

SPEC-V3-006 가 본 SPEC 보다 늦어지면 본 SPEC MS-2 는 **plain-text fallback** 으로 진행하고 SPEC-V3-006 PASS 후 별도 PR 로 highlight 활성화. (RG-G-9 carry: graceful degradation 정책.)

### 3.3 SPEC-V3-005 File Explorer 와의 병행

File Explorer (SPEC-V3-005) 가 파일 트리에 git status badge (M / A / U / D) 를 표시할 예정. 본 SPEC 의 `GitRepo::status_map` 가 그 데이터 소스가 된다. 두 SPEC 은 **읽기 전용 의존** 관계 — File Explorer 가 본 SPEC API 를 호출만 하고 mutation 은 본 SPEC 의 status_panel 만 수행.

---

## 4. 6 영역 UI 컴포넌트 구조

### 4.1 Status Panel (RG-G-1)

```
┌─ Status Panel ────────────────────┐
│ Branch: feature/SPEC-V3-008 ⚠ dirty │   ← header
├───────────────────────────────────┤
│ ▼ Staged (2)                       │
│   M  src/auth.rs                   │
│   A  src/git/mod.rs                │
│ ▼ Unstaged (3)                     │
│   M  Cargo.toml                    │
│   M  README.md                     │
│   D  src/legacy.rs                 │
│ ▼ Untracked (1)                    │
│   ?  notes.md                      │
└───────────────────────────────────┘
```

상호작용:
- 파일 클릭 → diff viewer 에 해당 파일 diff 로드 (RG-G-2)
- 파일 좌측 아이콘 클릭 → stage/unstage 토글 (`GitRepo::stage` / `unstage`)
- 우클릭 → 컨텍스트 메뉴 (discard, rename, ...)

### 4.2 Diff Viewer (RG-G-2)

```
┌─ src/auth.rs ─ unified ────────────┐
│ @@ -10,3 +10,5 @@                  │
│  fn login(user: &User) -> Result {  │
│-    user.password == HARDCODED      │
│+    bcrypt::verify(&user.pwd, ...)? │
│+    log_audit(user.id);              │
│ }                                    │
└─────────────────────────────────────┘
```

표시 모드 옵션 (RG-G-2 의 USER-DECISION UD-2):
- **Unified** (기본, git diff 와 동일) — 한 줄에 -/+ 표시
- **Side-by-side** — 좌측 ours / 우측 theirs, hunk 정렬

본 SPEC v1.0.0 은 **Unified 만 필수**. Side-by-side 는 best-effort (MS-2 carry-over 후보).

### 4.3 Commit Composer (RG-G-3)

```
┌─ Commit ──────────────────────────┐
│ Message:                           │
│ ┌──────────────────────────────┐  │
│ │ feat(auth): add bcrypt verify│  │   ← textarea
│ │                              │  │
│ │ AC-A-1, AC-A-2 통과.         │  │
│ └──────────────────────────────┘  │
│ Author: GOOS행님 <goos@afamily.kr>│   ← from .moai/config sections
│ [✓ AI 메시지 제안]                │   ← USER-DECISION UD-4
│ [Commit (Cmd+Enter)] [Discard]    │
└───────────────────────────────────┘
```

AI 메시지 제안 (UD-4):
- staged diff 를 Claude Code subprocess 로 보내어 메시지 1 줄 + 본문 초안 생성.
- 본 SPEC v1.0.0 은 **opt-in toggle** 로 제공 (기본 OFF). MS-1 시점 결정.

### 4.4 Branch Switcher (RG-G-4)

```
┌─ Branches ────────────────────────┐
│ Search: [feat        ]            │
├───────────────────────────────────┤
│ ★ feature/SPEC-V3-004-render      │   ← current
│   feature/SPEC-V3-008-git-ui      │
│   develop                         │
│   main                             │
│ ─ remote ─                         │
│   origin/develop                   │
│   origin/main                      │
│ [+ New Branch]                    │
└───────────────────────────────────┘
```

상호작용:
- 클릭 → `GitRepo::switch_branch` 호출. dirty 상태에서는 confirm dialog (UD-5).
- 새 브랜치 → 분기점 선택 (현재 HEAD 기본).

### 4.5 Log View (RG-G-5)

```
┌─ Log (50 commits) ────────────────┐
│ * dec518a feat(persist): SPEC-V3-003 MS-3 panes-v1 │
│ * c7a97dc feat(tabs): SPEC-V3-003 MS-2 TabContainer│
│ * 866b859 chore(ops): moai v2.14.0 + output-style │
│ |\                                                 │
│ | * a6b6fe8 chore(specs): M0/M1/M2-001/002 archive│
│ |/                                                 │
│ * e0ed220 merge(develop): Enhanced GitHub Flow    │
└──────────────────────────────────────────────────┘
```

graph 알고리즘 (UD-6):
- 자체 구현: `GitRepo::log` 가 parent oids 를 함께 반환하고 UI 가 column-based graph 그림.
- 외부: `git log --graph --oneline` 호출 후 ASCII 파싱.

본 SPEC 는 **자체 구현 권장**. 이유: ASCII 파싱은 fragile, GPUI 좌표계와 부조화.

### 4.6 Merge Conflict Resolver (RG-G-6)

```
┌─ Merge Conflict — src/auth.rs ────┐
│ ┌──── Ours ─────┬──── Theirs ────┐ │
│ │ fn login() { │ fn login() {   │ │
│ │   bcrypt(...) │   argon2(...)  │ │
│ │ }            │ }              │ │
│ └─────────────┴────────────────┘ │
│ ┌──── Merged ───────────────────┐ │   ← editable
│ │ fn login() {                  │ │
│ │   <<<<<<< HEAD                │ │   ← marker (편집 가능)
│ │   bcrypt(...)                 │ │
│ │   =======                     │ │
│ │   argon2(...)                 │ │
│ │   >>>>>>> theirs              │ │
│ │ }                             │ │
│ └───────────────────────────────┘ │
│ [Accept Ours] [Accept Theirs] [Mark Resolved] │
└──────────────────────────────────────────────┘
```

본 SPEC v1.0.0:
- 3-way diff **표시만 필수**. text editor 는 SPEC-V3-006 의 코드 에디터에 위임.
- "Mark Resolved" 클릭 시 `GitRepo::stage(path)` 호출, conflict 상태 해소.

### 4.7 Stash Management (RG-G-7)

```
┌─ Stashes ────────────────────────┐
│ stash@{0} WIP on feature/SPEC-V3-008 │
│ stash@{1} debug log additions      │
│ [Push Stash] [Pop] [Drop]         │
└───────────────────────────────────┘
```

본 SPEC v1.0.0 은 stash list / push / pop / drop 만. apply (without removing) 는 best-effort.

---

## 5. AI Commit Message 통합 (UD-4)

### 5.1 hook 아키텍처

```
Commit Composer (UI)
   │
   ▼ "Suggest" 버튼 클릭 또는 Cmd+/
   │
   ▼ staged diff 추출 (GitRepo::diff_index)
   │
   ▼ Claude Code subprocess 호출:
   │   claude code --headless \
   │     --prompt "Write a Conventional Commits commit message for this diff:" \
   │     --input <staged-diff>
   │
   ▼ stdout parse → commit message text
   │
   ▼ Composer textarea 에 채우기 (사용자 편집 가능)
```

### 5.2 SPEC-M2-001 (Claude Code subprocess) 와의 의존

- SPEC-M2-001 가 Claude Code subprocess 호출 abstraction 을 제공.
- 본 SPEC 은 그 abstraction 을 **opt-in 으로** 호출. SPEC-M2-001 미존재 시 toggle 자동 hide.

### 5.3 비채택 시

UD-4 에서 사용자가 "비활성" 선택하면:
- "Suggest" 버튼 숨김.
- Commit Composer 는 단순 textarea + Conventional Commits regex hint 만 제공.

---

## 6. 영속화 / 설정

### 6.1 사용자 설정 (.moai/config 또는 ~/.moai/config)

본 SPEC 도입 항목:

```yaml
git_ui:
  diff_view_mode: unified | side_by_side  # 기본: unified
  log_limit: 50                            # 기본: 50
  ai_commit_suggest: false                 # 기본: false (UD-4)
  graph_algorithm: native | external        # 기본: native (UD-6)
```

본 SPEC v1.0.0 은 **YAML 직접 편집** 만 지원. UI Settings Panel 은 별도 SPEC.

### 6.2 SPEC-V3-003 panes-v1 schema 무변경

본 SPEC 은 panes-v1 영속화 schema 에 **읽기 전용** 으로 의존. Git 탭이 추가되면 panes-v1 의 leaf payload 에 `GitView::Status | Diff | Log | ...` enum variant 가 추가될 가능성. → **별도 SPEC** (panes-v2 schema migration) 으로 분리. 본 SPEC 은 in-memory 만.

---

## 7. 위험 요약 (Risks)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R1 | git2 0.20 → 0.21 breaking change | 빌드 실패 | git2 = "0.20" 명시 pin, dependabot opt-out |
| R2 | merge conflict 시 corrupted state | 데이터 손실 | `GitRepo::merge` Result 에 `RollbackInstructions` 포함, UI 에서 "Abort Merge" 항상 가시 |
| R3 | 대형 repo 의 status / log 호출이 UI freeze | UX 저하 | 모든 git2 호출은 `tokio::task::spawn_blocking` 또는 별도 thread, UI 는 loading state |
| R4 | SPEC-V3-006 syntax highlight 미완성 | Diff Viewer plain-text | RG-G-9 graceful degradation, plain text fallback |
| R5 | AI commit suggest 가 Claude Code 미설치 환경에서 panic | 크래시 | UD-4 비채택 또는 `which claude` 체크 후 toggle hide |
| R6 | log graph 자체 구현 버그 | 잘못된 시각화 | unit tests with fixture commit graphs (linear, branched, octopus merge) |
| R7 | gix vs git2 결정 번복 | 대규모 rewrite | UD-1 명시 결정 + 본 SPEC v1.0.0 git2 lock-in |
| R8 | merge resolver textbox 가 SPEC-V3-006 editor 미완 시 폴백 부재 | MS-3 차질 | textarea 폴백 (no-syntax) + SPEC-V3-006 dependency comment |
| R9 | macOS / Linux 키 바인딩 차이 (Cmd vs Ctrl) | 크로스 OS 버그 | SPEC-V3-003 에서 검증된 dispatch_tab_key 패턴 재사용 |
| R10 | 동시 git 작업 (외부 CLI + UI) race | index corruption | UI 의 모든 mutation 은 advisory lock 또는 best-effort retry |

---

## 8. 결정 게이트 (USER-DECISION 후보)

| ID | 결정 사항 | 권장안 | plan.md 위치 |
|----|----------|--------|--------------|
| UD-1 | git2-rs 유지 vs gix 전환 | **git2-rs 유지** | §2 T1 |
| UD-2 | Diff Viewer mode | **Unified 필수, side-by-side best-effort** | §6 T6 |
| UD-3 | UI 통합 패턴 | **C. Hybrid (sidebar + leaf + log tab)** | §3 T2 |
| UD-4 | AI commit suggest | **opt-in (기본 OFF)** | §7 T7 |
| UD-5 | Branch switch 시 dirty 처리 | **confirm dialog + autostash 옵션** | §8 T8 |
| UD-6 | Log graph 알고리즘 | **자체 구현 (column-based)** | §10 T10 |
| UD-7 | Stash 범위 | **list/push/pop/drop, apply best-effort** | §11 T11 |

---

## 9. 다음 단계

본 research.md 의 결과를 받아 spec.md 가 EARS 형식 요구사항을 정의하고 plan.md 가 milestone × task 분해를 수행한다.

- **MS-1 (T1~T5)**: Status Panel + Commit Composer (basic). UD-1, UD-3, UD-4 게이트 통과.
- **MS-2 (T6~T9)**: Diff Viewer + Branch Switcher. UD-2, UD-5 게이트 통과.
- **MS-3 (T10~T13)**: Log Graph + Merge Conflict Resolver + Stash. UD-6, UD-7 게이트 통과.

---

작성 완료: 2026-04-25
