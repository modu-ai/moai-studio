# SPEC-V3-005 Implementation Plan — File Explorer Surface

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-005-explorer` (예정 — 본 산출물은 `feature/SPEC-V3-004-render` 위에서 작성되며 orchestrator 가 후속 분리)
범위: SPEC-V3-005 spec.md 의 6 RG × 3 MS × 12 AC × 3 USER-DECISION 의 task 분해.
선행: SPEC-V3-004 가 RootView::tab_container 배선까지 완료한 상태를 가정한다 (RootView 에 `file_explorer: Option<Entity<FileExplorer>>` 필드 추가만 추가 작업).

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|------|----------------------|-----|----|
| **T0** | 전체 | USER-DECISION 게이트 3 개 동시 호출 + Spike 0 (gpui test-support) + Spike 1 (trash crate) | `Cargo.toml` (가능 시), Spike 보고서 inline → progress.md | — | (게이트) — |
| **T1** | MS-1 | `explorer/tree.rs` — FsNode + ChildState 정의 + 단위 테스트 | `crates/moai-studio-ui/src/explorer/tree.rs` (신규), `crates/moai-studio-ui/src/explorer/mod.rs` (신규) | T0 | AC-FE-1, AC-FE-2 |
| **T2** | MS-1 | `explorer/path.rs` — cross-platform normalize_for_display + cfg-gated 단위 테스트 | `crates/moai-studio-ui/src/explorer/path.rs` (신규) | T1 | AC-FE-3 |
| **T3** | MS-1 | `moai-fs::WorkspaceWatcher` (USER-DECISION-A 결과에 따른 분기) | USER-DECISION-A=(a) 시: `crates/moai-fs/src/workspace_watcher.rs` (신규), `moai-fs/src/lib.rs` re-export. (b) 시: 본 task 스킵, T5 가 직접 wrap. | T0 | (T5 의존) |
| **T4** | MS-1 | `explorer/view.rs` — FileExplorer struct + impl Render placeholder + on_file_open callback | `crates/moai-studio-ui/src/explorer/view.rs` (신규), `crates/moai-studio-ui/src/lib.rs:72-99` (RootView 필드 추가) | T1, T2 | AC-FE-4 |
| **T5** | MS-2 | `explorer/watch.rs` — debounce 100ms timer + buffer + FsDelta 매칭 (rename) + backpressure 폴백 | `crates/moai-studio-ui/src/explorer/watch.rs` (신규) | T3 | AC-FE-5, AC-FE-6, AC-FE-7 |
| **T6** | MS-2 | `.moai/config/sections/fs.yaml` 디폴트 + load helper | `.moai/config/sections/fs.yaml` (신규), `crates/moai-studio-ui/src/explorer/config.rs` (신규) | T5 | AC-FE-5 (디폴트 100ms 검증) |
| **T7** | MS-3 | `explorer/git_status.rs` — GitStatus enum + GitStatusProvider trait + MoaiGitStatusProvider 구현 + roll_up_priority | `crates/moai-studio-ui/src/explorer/git_status.rs` (신규) | T1 | AC-FE-8 |
| **T8** | MS-3 | `explorer/menu.rs` — context menu popup + inline edit row (New File / New Folder / Rename) + fs 액션 디스패치 | `crates/moai-studio-ui/src/explorer/menu.rs` (신규), `crates/moai-studio-ui/src/explorer/view.rs` (우클릭 핸들러 배선) | T4 | AC-FE-9 |
| **T9** | MS-3 | delete dispatch (USER-DECISION-C 분기) | T8 와 같은 파일 + (USER-DECISION-C=(a) 또는 (c) 시) `Cargo.toml` `trash = "5"` 추가 | T8 | AC-FE-10 |
| **T10** | MS-3 | `explorer/dnd.rs` — drag start / drop target validation / fs::rename 디스패치 | `crates/moai-studio-ui/src/explorer/dnd.rs` (신규) | T4, T5 (delta apply 호환) | AC-FE-11 |
| **T11** | MS-3 | `explorer/search.rs` — fuzzy match + visibility 갱신 + 검색 input element | `crates/moai-studio-ui/src/explorer/search.rs` (신규) | T4 | AC-FE-12 |
| **T12** | MS-3 | integration test `tests/integration_explorer.rs` — RG-FE-1/2/3 e2e | `crates/moai-studio-ui/tests/integration_explorer.rs` (신규) | T5, T7, T8, T10, T11 | AC-FE-4, AC-FE-9, AC-FE-11 |
| **T13** | 전체 | regression sweep + clippy/fmt + smoke + progress.md 갱신 + commit | (git 작업) | T1~T12 | 모든 AC |

---

## 2. T0 — USER-DECISION 게이트 + Spike 0 / Spike 1

### 2.1 호출 (AskUserQuestion 3 질문 동시 송출)

[USER-DECISION-REQUIRED: fs-watcher-api-shape-v3-005]
[USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-005] (SPEC-V3-004 carry-over consistency)
[USER-DECISION-REQUIRED: delete-trash-policy-v3-005]

순서: AskUserQuestion 1 회로 4 옵션 max 제한이 있어 두 라운드로 분리할 가능성. 본 plan 은 manager-spec 자율로 두 라운드까지 허용.

라운드 1 — USER-DECISION-A (`fs-watcher-api-shape`):
- (a) 권장: `moai-fs` 에 `WorkspaceWatcher` helper 추가. SPEC-V3-008 와 공유 가능.
- (b) explorer crate 자체 wrap. SPEC-V3-008 통합 시 추가 리팩 필요.

라운드 2 — USER-DECISION-B (`gpui-test-support`) + USER-DECISION-C (`delete-trash-policy`):
- B(a) 권장: gpui test-support 추가. e2e AC 가 GPUI 환경 검증.
- B(b): logic-level fallback. ~80 LOC 우회 코드.
- C(a) 권장: trash crate 추가. delete 시 OS 휴지통.
- C(b): std::fs::remove_* 영구 삭제. 의존성 추가 없음.
- C(c): 둘 다 옵션 — modal 에 "Move to Trash" / "Delete Permanently" 두 버튼.

### 2.2 Spike 0 (USER-DECISION-B=(a) 채택 시)

- `Cargo.toml` `[dev-dependencies]` 에 `gpui = { version = "0.2", features = ["test-support"] }` 추가.
- `cargo test -p moai-studio-ui --no-run` 빌드 통과 검증.
- 실패 시 자동 fallback → B=(b), progress.md 기록.

### 2.3 Spike 1 (USER-DECISION-C=(a) 또는 (c) 채택 시)

- `Cargo.toml` `[dependencies]` 에 `trash = "5"` 추가 → `crates/moai-studio-ui/Cargo.toml` 의존성 등록.
- `cargo build -p moai-studio-ui` 통과 검증.
- 빌드 실패 시 자동 fallback → C=(b), progress.md 기록 + 사용자 통지.

### 2.4 Spike 결과 기록

- `progress.md` 의 USER-DECISION 섹션에 (A/B/C) × (선택, 실측 영향, 우회 사실 여부) 기록.
- T1 부터 결정 결과를 가정하고 진행.

---

## 3. T1 — FsNode + ChildState (RG-FE-1 REQ-FE-001/002/003 부분)

### 3.1 변경 대상

신규 파일 `crates/moai-studio-ui/src/explorer/tree.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-1. FsNode 는 GPUI 의존 없는 logic-only 도메인 모델.
//!   File Explorer 의 핵심 엔티티이며 watch / git_status / search 모듈이 이 트리를 mutate 한다.
//! @MX:SPEC: SPEC-V3-005

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FsError {
    #[error("권한 거부: {0}")]
    PermissionDenied(PathBuf),
    #[error("I/O: {0}")]
    Io(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChildState {
    NotLoaded,
    Loading,
    Loaded(Vec<FsNode>),
    Failed(FsError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FsNode {
    File { rel_path: PathBuf, name: String, git_status: super::git_status::GitStatus, is_visible_under_filter: bool },
    Dir  { rel_path: PathBuf, name: String, children: ChildState, git_status: super::git_status::GitStatus, is_expanded: bool, is_visible_under_filter: bool },
}

impl FsNode {
    pub fn new_dir(rel_path: PathBuf, name: String) -> Self { /* ... */ }
    pub fn new_file(rel_path: PathBuf, name: String) -> Self { /* ... */ }
    pub fn rel_path(&self) -> &PathBuf { /* ... */ }
    pub fn name(&self) -> &str { /* ... */ }
}
```

### 3.2 단위 테스트 (AC-FE-1, AC-FE-2)

- `test_new_dir_default_state`: rel_path / name / children=NotLoaded / is_expanded=false 검증.
- `test_new_file_default_state`: rel_path / name / git_status=Clean / is_visible_under_filter=true.
- `test_child_state_variants`: 4 변형 모두 생성 가능 + PartialEq 동치.

### 3.3 mod.rs 진입점

신규 `crates/moai-studio-ui/src/explorer/mod.rs`:

```rust
//! @MX:ANCHOR: [AUTO] explorer-module-root
//! @MX:REASON: [AUTO] SPEC-V3-005 의 7 개 서브모듈 (tree/path/view/watch/git_status/menu/dnd/search/config) 의 단일 진입점.
//!   fan_in >= 3: lib.rs (RootView 필드), integration_explorer.rs, 다른 SPEC (V3-008) 향후 참조.
//! @MX:SPEC: SPEC-V3-005

pub mod config;
pub mod dnd;
pub mod git_status;
pub mod menu;
pub mod path;
pub mod search;
pub mod tree;
pub mod view;
pub mod watch;

pub use view::FileExplorer;
```

---

## 4. T2 — path::normalize_for_display (RG-FE-1 REQ-FE-004)

### 4.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/path.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-1 REQ-FE-004. cross-platform path 정규화 단일 진입점.
//!   git2 / FsWatcher / display 가 모두 forward-slash 표준을 따르도록 보장.
//! @MX:SPEC: SPEC-V3-005

use std::path::Path;

pub fn normalize_for_display(p: &Path) -> String {
    let s = p.to_string_lossy();
    #[cfg(windows)]
    { s.replace('\\', "/") }
    #[cfg(not(windows))]
    { s.into_owned() }
}
```

### 4.2 단위 테스트 (AC-FE-3)

- `test_normalize_unix`: `Path::new("foo/bar")` → `"foo/bar"`.
- `test_normalize_windows` (cfg(windows)): `Path::new(r"foo\bar")` → `"foo/bar"`.
- `test_normalize_unicode`: `Path::new("한글/파일.txt")` → `"한글/파일.txt"`.
- `test_normalize_dotdot`: `Path::new("../escape")` → `"../escape"` (이 함수는 단순 정규화만 책임, escape 검증은 view 책임).

cfg-gated 테스트는 `cfg!(windows)` 또는 `cfg(target_os = "windows")` 로 분기.

---

## 5. T3 — moai-fs WorkspaceWatcher (USER-DECISION-A 분기)

### 5.1 USER-DECISION-A=(a) 경로

신규 `crates/moai-fs/src/workspace_watcher.rs`:

```rust
//! @MX:ANCHOR: [AUTO] workspace-watcher-multiplexer
//! @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-2 + SPEC-V3-008 가 같은 watcher 인스턴스를 공유하기 위한 multiplexer.
//!   fan_in >= 2: explorer (V3-005), git UI (V3-008 미래).
//! @MX:SPEC: SPEC-V3-005

use crate::{FsEvent, FsWatcher, FsWatcherError, WorkspaceKey};
use std::path::Path;
use tokio::sync::broadcast;

pub struct WorkspaceWatcher {
    inner: FsWatcher,
    bus: broadcast::Sender<(WorkspaceKey, FsEvent)>,
}

impl WorkspaceWatcher {
    pub fn new(workspace_key: WorkspaceKey, root: &Path) -> Result<Self, FsWatcherError> { /* ... */ }
    pub fn subscribe(&self) -> broadcast::Receiver<(WorkspaceKey, FsEvent)> { /* ... */ }
}
```

`crates/moai-fs/src/lib.rs` 의 re-export 한 줄 추가: `pub mod workspace_watcher; pub use workspace_watcher::WorkspaceWatcher;`.

### 5.2 USER-DECISION-A=(b) 경로

본 task 스킵. T5 가 `FsWatcher` 를 직접 wrap. SPEC-V3-008 진행 시 별도 리팩이 필요할 수 있음을 progress.md 에 기록.

### 5.3 단위 테스트

(a) 채택 시:
- `test_workspace_watcher_subscribe_count`: 2 개 subscriber 가 동일 이벤트를 받는지 검증.
- `test_workspace_watcher_drop_releases`: drop 시 watcher 가 해제되어 외부 fs 변경이 더는 송출되지 않는지.

(b) 채택 시:
- 본 task 검증은 T5 의 단위 테스트로 흡수.

---

## 6. T4 — FileExplorer struct + Render placeholder + RootView 필드 (RG-FE-1 REQ-FE-005)

### 6.1 변경 대상 1: 신규 `crates/moai-studio-ui/src/explorer/view.rs`

```rust
//! @MX:ANCHOR: [AUTO] file-explorer-entity
//! @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-1. FileExplorer 는 RootView::file_explorer 의 진입점이며,
//!   tree / watch / git_status / menu / dnd / search 의 mutation 이 모두 이 Entity 로 수렴한다.
//!   fan_in >= 4: RootView (T4), watch (T5), menu (T8), dnd (T10), search (T11).
//! @MX:SPEC: SPEC-V3-005

use gpui::{Context, Entity, IntoElement, Render, Window};
use std::path::PathBuf;

pub struct FileExplorer {
    pub workspace_root: PathBuf,
    pub tree: super::tree::FsNode,            // root Dir node
    pub search_query: String,
    pub on_file_open: Option<Box<dyn Fn(PathBuf, PathBuf) + 'static>>,
    // T5 가 추가: pub watch_handle: Option<JoinHandle<()>>,
    // T7 가 추가: pub git_status_provider: Box<dyn GitStatusProvider>,
}

impl FileExplorer {
    pub fn new(workspace_root: PathBuf) -> Self { /* ... */ }
    pub fn set_on_file_open<F: Fn(PathBuf, PathBuf) + 'static>(&mut self, cb: F) { /* ... */ }
    pub fn expand_dir(&mut self, rel_path: &PathBuf, cx: &mut Context<Self>) { /* T1 ChildState 전이 */ }
    pub fn open_file(&self, rel_path: &PathBuf) { /* on_file_open 호출 */ }
}

impl Render for FileExplorer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // T4: placeholder — 트리 첫 레벨만 평면 렌더 (자식 lazy load 트리거는 T8 이 추가)
        // T11 가 search input 추가, T8 이 우클릭 핸들러 추가, T10 이 drag handler 추가.
        // ...
    }
}
```

### 6.2 변경 대상 2: `crates/moai-studio-ui/src/lib.rs:16-19, 72-99`

```rust
pub mod explorer;  // <- 신규 모듈 등록 (16~19 line 근처)

pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    pub tab_container: Option<Entity<tabs::TabContainer>>,
    // @MX:ANCHOR: [AUTO] root-view-file-explorer-binding
    // @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-1. file_explorer 는 sidebar 좌측 영역의
    //   진입점이며 워크스페이스 활성 변경 시 set_workspace 로 재바인딩된다.
    //   fan_in >= 3: T4 init, T5 watch event, T7 git_status refresh.
    pub file_explorer: Option<Entity<explorer::FileExplorer>>,
}
```

`Render for RootView` 의 sidebar 영역에서 `file_explorer` 가 Some 이면 `cx.new(...)` 의 결과를 child 로 마운트. None 이면 SPEC-V3-001 의 workspace 리스트 placeholder 유지.

### 6.3 단위 테스트 (AC-FE-4)

- `test_file_explorer_new_default_state`: workspace_root 보유 / tree=Dir(NotLoaded) / on_file_open=None 검증.
- `test_set_on_file_open_callback_invoked`: callback 등록 후 `open_file(rel)` 호출 → callback 1 회 invocation 검증 (logic-level, GPUI 미사용).
- (USER-DECISION-B=(a) 시) GPUI integration: `cx.new(\|cx\| FileExplorer::new(...))` 로 Entity 생성 가능 검증.

---

## 7. T5 — debounce 100ms + FsDelta 매칭 + backpressure (RG-FE-2 REQ-FE-010~013)

### 7.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/watch.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-2. notify 이벤트를 100ms debounce 윈도우로 묶고
//!   Removed+Created 페어를 Renamed 로 매칭. backpressure 시 refresh_root 폴백.
//! @MX:WARN: [AUTO] timer 누적 / buffer 메모리 / async cancellation 의 3 축에서 leak 가능.
//! @MX:REASON: [AUTO] tokio task lifecycle 이 FileExplorer Entity drop 과 묶여야 한다.
//!   ChildState mutation 이 GPUI thread 와 watcher thread 두 곳에서 수행되므로 racing 위험.
//! @MX:SPEC: SPEC-V3-005

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use moai_fs::FsEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum FsDelta {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
    BulkRefresh,  // backpressure 폴백 시그널
}

pub struct DebounceBuffer {
    events: Vec<FsEvent>,
    started_at: Option<std::time::Instant>,
    debounce_ms: u64,
    backpressure_threshold: usize,
}

impl DebounceBuffer {
    pub fn new(debounce_ms: u64) -> Self { /* ... */ }
    pub fn push(&mut self, e: FsEvent) { /* ... */ }
    pub fn try_flush(&mut self) -> Option<Vec<FsDelta>> { /* timer 만료 시 호출, 매칭 + dedupe + backpressure 결정 */ }
}

pub fn match_renames(events: &[FsEvent]) -> Vec<FsDelta> { /* Removed+Created 같은 부모 → Renamed */ }
```

### 7.2 통합 — FileExplorer 와의 결선

T4 의 FileExplorer 에 다음 추가:

```rust
impl FileExplorer {
    pub fn start_watch(&mut self, workspace_key: WorkspaceKey, cx: &mut Context<Self>) -> Result<(), FsWatcherError> {
        // T3 의 WorkspaceWatcher (USER-DECISION-A=(a)) 또는 직접 FsWatcher (USER-DECISION-A=(b))
        // 의 이벤트 수신 task 를 tokio::spawn.
        // 받은 FsEvent → DebounceBuffer 에 push.
        // 100ms tick 마다 try_flush → Vec<FsDelta> → cx.update(... apply_delta(delta) ...)
    }
    pub fn apply_delta(&mut self, delta: FsDelta, cx: &mut Context<Self>) { /* tree mutation + cx.notify() */ }
    pub fn refresh_root(&mut self, cx: &mut Context<Self>) { /* RG-FE-2 REQ-FE-013 폴백 */ }
}
```

### 7.3 단위 테스트 (AC-FE-5, AC-FE-6, AC-FE-7)

- `test_debounce_coalesce_5_events_in_window`: 50ms 간격 5 이벤트 → try_flush 결과 = 1 개의 batch (`Vec<FsDelta>` len=5, dedupe 후) — `tokio::time::pause()` + `advance(100ms)` 사용.
- `test_match_renames_same_parent`: `Removed("foo/a.txt")` + `Created("foo/b.txt")` → `vec![FsDelta::Renamed { from, to }]`.
- `test_match_renames_different_parent_no_match`: `Removed("foo/a")` + `Created("bar/b")` → 두 독립 delta (Removed + Created).
- `test_backpressure_fallback_at_500`: 600 mock 이벤트 push → try_flush 결과 = `vec![FsDelta::BulkRefresh]` 1 건.

---

## 8. T6 — fs.yaml 디폴트 + load helper

### 8.1 변경 대상

신규 `.moai/config/sections/fs.yaml`:

```yaml
fs:
  debounce_ms: 100  # range 50~500
  backpressure_threshold: 500
  hidden_patterns:
    - ".git"
    - "node_modules"
    - ".DS_Store"
    - "target"
```

신규 `crates/moai-studio-ui/src/explorer/config.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 T6 — fs.yaml 로드 helper.
//! 미존재 시 기본값 사용.
//! @MX:SPEC: SPEC-V3-005

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FsConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_backpressure")]
    pub backpressure_threshold: usize,
    #[serde(default)]
    pub hidden_patterns: Vec<String>,
}

fn default_debounce_ms() -> u64 { 100 }
fn default_backpressure() -> usize { 500 }

impl FsConfig {
    pub fn load_or_default(config_path: &std::path::Path) -> Self { /* serde_yaml + fallback */ }
}
```

### 8.2 단위 테스트

- `test_load_default_when_missing`: 미존재 path → 디폴트 (100, 500, hidden=[]).
- `test_load_partial_yaml`: `{ fs: { debounce_ms: 250 } }` → debounce_ms=250, 나머지 디폴트.
- `test_load_invalid_yaml_falls_back`: parse 실패 시 디폴트 + tracing warn.

---

## 9. T7 — GitStatusProvider trait + MoaiGitStatusProvider (RG-FE-3 REQ-FE-020/021)

### 9.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/git_status.rs`:

```rust
//! @MX:ANCHOR: [AUTO] git-status-provider-trait
//! @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-3 + SPEC-V3-008 미래 통합 hook.
//!   fan_in >= 2: FileExplorer (V3-005), git UI (V3-008 미래).
//! @MX:SPEC: SPEC-V3-005

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus { Clean, Modified, Added, Deleted, Untracked, Renamed, Conflicted }

impl GitStatus {
    pub fn priority(self) -> u8 {
        match self {
            GitStatus::Conflicted => 6,
            GitStatus::Modified   => 5,
            GitStatus::Added      => 4,
            GitStatus::Deleted    => 3,
            GitStatus::Renamed    => 2,
            GitStatus::Untracked  => 1,
            GitStatus::Clean      => 0,
        }
    }
}

pub fn roll_up_priority(children: &[GitStatus]) -> GitStatus {
    children.iter().copied().max_by_key(|s| s.priority()).unwrap_or(GitStatus::Clean)
}

#[derive(Debug, thiserror::Error)]
pub enum GitStatusError {
    #[error("git: {0}")]
    Git(#[from] moai_git::GitError),
    #[error("repo not found at {0:?}")]
    RepoNotFound(std::path::PathBuf),
}

pub trait GitStatusProvider: Send + Sync {
    fn status_map(&self, repo_root: &Path) -> Result<HashMap<String, GitStatus>, GitStatusError>;
}

pub struct MoaiGitStatusProvider;

impl GitStatusProvider for MoaiGitStatusProvider {
    fn status_map(&self, repo_root: &Path) -> Result<HashMap<String, GitStatus>, GitStatusError> {
        let repo = moai_git::GitRepo::open(repo_root)?;
        let raw = repo.status_map()?;
        Ok(raw.into_iter().map(|(p, label)| (p, parse_label(&label))).collect())
    }
}

fn parse_label(s: &str) -> GitStatus {
    match s {
        "M" => GitStatus::Modified,
        "A" => GitStatus::Added,
        "D" => GitStatus::Deleted,
        "??" | "U" => GitStatus::Untracked,
        "R" => GitStatus::Renamed,
        "C" => GitStatus::Conflicted,
        _   => GitStatus::Clean,
    }
}
```

### 9.2 FileExplorer 통합

T4 의 FileExplorer 에 추가:

```rust
pub struct FileExplorer {
    // ...
    pub git_status_provider: Box<dyn GitStatusProvider>,  // 디폴트는 MoaiGitStatusProvider
}

impl FileExplorer {
    pub fn refresh_git_status(&mut self, cx: &mut Context<Self>) {
        // tokio::spawn 으로 status_map 호출, 결과를 트리에 머지
        // 실패 시 tracing warn + tree 모든 노드 git_status=Clean fallback (REQ-FE-023)
    }
}
```

### 9.3 단위 테스트 (AC-FE-8)

- `test_priority_ordering`: Conflicted=6 > Modified=5 > Added=4 > Deleted=3 > Renamed=2 > Untracked=1 > Clean=0.
- `test_roll_up_picks_max`: `[Modified, Untracked, Clean]` → Modified.
- `test_roll_up_empty_returns_clean`: `[]` → Clean.
- `test_parse_label_all_variants`: M/A/D/??/U/R/C 매핑.
- `test_provider_with_tempdir_git`: tempdir + git2::Repository::init + 파일 생성 → MoaiGitStatusProvider::status_map 결과에 Untracked 포함 검증.

---

## 10. T8 — context menu (RG-FE-4 REQ-FE-030~033)

### 10.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/menu.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-4. context menu popup + inline edit row.
//! @MX:SPEC: SPEC-V3-005

#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuKind { Dir, File }

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    NewFile,
    NewFolder,
    Rename,
    Delete,
    RevealInFinder,
}

pub fn menu_items_for(kind: ContextMenuKind) -> Vec<MenuAction> {
    match kind {
        ContextMenuKind::Dir => vec![MenuAction::NewFile, MenuAction::NewFolder, MenuAction::Rename, MenuAction::Delete, MenuAction::RevealInFinder],
        ContextMenuKind::File => vec![MenuAction::Rename, MenuAction::Delete, MenuAction::RevealInFinder],
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InlineEditKind { NewFile, NewFolder, Rename }
```

T4 의 view.rs 에 우클릭 핸들러 + inline edit state 추가:

```rust
pub struct FileExplorer {
    // ...
    pub inline_edit: Option<InlineEditState>,
    pub context_menu: Option<ContextMenuState>,
}
```

### 10.2 단위 테스트 (AC-FE-9 logic-level)

- `test_menu_items_for_dir`: 5 항목 — NewFile / NewFolder / Rename / Delete / RevealInFinder.
- `test_menu_items_for_file`: 3 항목 — Rename / Delete / RevealInFinder.
- `test_start_new_file_inline_edit`: `start_inline_edit(NewFile, parent_rel)` → inline_edit=Some(...) + 부모 디렉토리 기준 placeholder.
- `test_confirm_inline_edit_creates_file`: tempdir + inline_edit confirm → fs::File::create 호출 검증.
- `test_rename_action_confirms_rename`: tempdir + rename action → fs::rename 호출 검증.

USER-DECISION-B=(a) 시 GPUI integration 추가:
- `test_right_click_shows_popup_5_items`: GPUI 환경 e2e — 우클릭 시뮬레이션 → menu element 5 child 검증.

---

## 11. T9 — delete dispatch (RG-FE-4 REQ-FE-034, USER-DECISION-C 분기)

### 11.1 USER-DECISION-C 분기

```rust
impl FileExplorer {
    fn confirm_delete(&mut self, rel_path: &PathBuf, cx: &mut Context<Self>) {
        let abs = self.workspace_root.join(rel_path);
        let result = match TRASH_POLICY {
            TrashPolicy::AlwaysTrash => trash::delete(&abs).map_err(...),  // (a) 채택 시
            TrashPolicy::AlwaysPermanent => {                              // (b) 채택 시
                if abs.is_dir() { std::fs::remove_dir_all(&abs) } else { std::fs::remove_file(&abs) }
            },
            TrashPolicy::UserChoice(choice) => match choice {              // (c) 채택 시
                UserDeleteChoice::Trash => trash::delete(&abs).map_err(...),
                UserDeleteChoice::Permanent => /* (b) 와 동일 */,
            },
        };
        if let Err(e) = result { tracing::error!(?e, "delete failed"); /* status bar */ }
    }
}
```

`Cargo.toml` (USER-DECISION-C=(a) or (c) 채택 시):

```toml
[dependencies]
trash = "5"
```

### 11.2 단위 테스트 (AC-FE-10)

- `test_delete_dispatch_trash_policy_a` (cfg(feature = "trash")): tempdir 파일 → confirm_delete → trash::delete 1 회 호출 검증 (mock).
- `test_delete_dispatch_permanent_policy_b`: tempdir 파일 → confirm_delete → fs::remove_file 1 회 호출 + 파일 사라짐 검증.
- `test_delete_dispatch_user_choice_c`: 두 분기 모두 검증.
- `test_delete_failure_does_not_panic`: 권한 거부 mock → tracing error + tree 변경 없음.

---

## 12. T10 — drag-and-drop (RG-FE-5 REQ-FE-040~042)

### 12.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/dnd.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-5. drag start / drop validate / fs::rename dispatch.
//! @MX:WARN: [AUTO] descendant 검사 누락 시 무한 루프 / 자기 자신 이동 데이터 손실.
//! @MX:REASON: [AUTO] is_descendant_of 가 path strip_prefix 로 정확히 동작해야 한다.
//! @MX:SPEC: SPEC-V3-005

use std::path::Path;

pub fn is_self_or_descendant(src: &Path, target: &Path) -> bool {
    target == src || target.starts_with(src)
}

pub fn validate_and_compute_dst(src: &Path, target_dir: &Path, name: &str) -> Result<std::path::PathBuf, DnDError> {
    if is_self_or_descendant(src, target_dir) { return Err(DnDError::SelfOrDescendant); }
    Ok(target_dir.join(name))
}

#[derive(Debug, thiserror::Error)]
pub enum DnDError {
    #[error("drop target is source itself or its descendant")]
    SelfOrDescendant,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
```

### 12.2 단위 테스트 (AC-FE-11)

- `test_is_descendant_self_true`: same path → true.
- `test_is_descendant_child_true`: `foo` → `foo/bar` true.
- `test_is_descendant_sibling_false`: `foo` → `bar` false.
- `test_validate_rejects_self_drop`: dst=src → `Err(SelfOrDescendant)`.
- `test_dnd_rename_success`: tempdir 에 `a/file.txt` + `b/` → drop file.txt on b/ → fs::rename 호출 + b/file.txt 존재 검증.
- `test_dnd_rename_failure_no_panic`: cross-device mock → tracing warn + 원본 변경 없음 검증.

---

## 13. T11 — fuzzy search filter (RG-FE-6 REQ-FE-050~052)

### 13.1 변경 대상

신규 `crates/moai-studio-ui/src/explorer/search.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-005 RG-FE-6. fuzzy match (case-insensitive subsequence match).
//! @MX:SPEC: SPEC-V3-005

use crate::explorer::tree::FsNode;

pub fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() { return true; }
    let q = query.to_lowercase();
    let t = target.to_lowercase();
    let mut q_iter = q.chars();
    let mut current = q_iter.next();
    for c in t.chars() {
        if let Some(qc) = current {
            if c == qc { current = q_iter.next(); }
        }
    }
    current.is_none()
}

pub fn apply_filter(node: &mut FsNode, query: &str) -> bool {
    // 재귀: visibility 갱신 후 자기 자신 또는 자식 중 하나라도 visible 이면 true.
    // empty query → 모두 true.
}
```

T4 의 view.rs 의 search input element 추가 (위치: 트리 상단).

### 13.2 단위 테스트 (AC-FE-12)

- `test_fuzzy_match_subsequence`: query="auth", target="src/auth/mod.rs" → true.
- `test_fuzzy_match_case_insensitive`: query="AUTH", target="src/Auth/mod.rs" → true.
- `test_fuzzy_match_no_match`: query="xyz", target="auth" → false.
- `test_fuzzy_match_empty_query_true`: query="" → true.
- `test_apply_filter_keeps_visible_path`: 트리 = [src/main.rs, src/auth/mod.rs, tests/auth_test.rs] + query="auth" → 2 노드 visible (src/auth/mod.rs, tests/auth_test.rs), src/main.rs invisible.
- `test_apply_filter_empty_restores`: query="" → 모두 visible.

---

## 14. T12 — integration test (RG-FE-1/2/3 e2e)

### 14.1 변경 대상

신규 `crates/moai-studio-ui/tests/integration_explorer.rs`:

```rust
//! Integration tests for SPEC-V3-005 File Explorer.
//! Test harness: tempdir + git2::Repository::init + tokio::time::pause + (USER-DECISION-B 결과에 따라) GPUI cx.

#[tokio::test(flavor = "current_thread")]
async fn test_explorer_full_lifecycle() {
    // 1. tempdir 생성 + git init + 파일 추가.
    // 2. FileExplorer::new(workspace_root) + start_watch().
    // 3. external fs 변경 (touch new.txt) → 100ms 대기 → tree 에 new.txt 등장 검증.
    // 4. refresh_git_status → status_map 의 new.txt = Untracked 검증.
    // 5. context menu New File → confirm → another.txt 생성 + tree 등장.
    // 6. dnd: another.txt → src/ → fs::rename + tree 반영.
    // 7. search "another" → another.txt visible only.
}
```

USER-DECISION-B=(a) 시: GPUI cx 사용.
USER-DECISION-B=(b) 시: logic-level FileExplorer 메서드만 직접 호출.

### 14.2 검증 AC

- AC-FE-4 (file open callback)
- AC-FE-9 (context menu)
- AC-FE-11 (dnd)

---

## 15. T13 — regression sweep + smoke + commit

### 15.1 검증 명령

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
cargo test -p moai-studio-ui --test integration_explorer
cargo build -p moai-studio-app  # smoke
```

### 15.2 progress.md 갱신

- USER-DECISION-A/B/C 결정 결과
- Spike 0/1 빌드 결과
- AC-FE-1 ~ AC-FE-12 PASS 여부
- @MX 태그 변경 보고 (NEW: ANCHOR x 4, NOTE x 4, WARN x 1)

### 15.3 commit (현재 브랜치 — feature/SPEC-V3-004-render)

본 SPEC 산출물은 docs only. orchestrator 가 이후 별도 브랜치로 분리.

```
docs(spec): SPEC-V3-005 File Explorer v1.0.0 (research/plan/spec)

🗿 MoAI <email@mo.ai.kr>
```

NO PUSH, NO PR.

---

## 16. 위험 요약 (plan 레벨)

| 위험 | 완화 |
|------|------|
| USER-DECISION-A=(b) 채택 시 SPEC-V3-008 와 watcher 인스턴스 공유 어려움 | progress.md 에 향후 리팩 부담 명시, SPEC-V3-008 진입 시 재평가 게이트 |
| USER-DECISION-B=(b) 채택 시 GPUI e2e 우회 코드 ~80 LOC 추가 | logic-level fallback 명세를 test 모듈에 격리, GPUI 채택 시 그대로 swap |
| trash crate 빌드 실패 (Linux 일부 distro) | Spike 1 으로 사전 검증, 실패 시 자동 fallback C=(b) |
| 5k 파일 모노레포에서 status_map 호출 비용 500ms+ | T7 의 호출은 tokio::spawn 격리, UI thread 블록 없음 |
| Windows path 정규화 cfg-gated 테스트가 macOS/Linux CI 에서 실행 안 됨 | T2 의 단위 테스트에 `cfg(target_os = "windows")` 가지를 명시, Windows runner CI 도입 시 자동 활성 |
| context menu / inline edit 의 GPUI element 빌드가 0.2.2 에서 미지원 | Spike 0 결과로 결정. 빌드 실패 시 RG-FE-4 / RG-FE-5 의 e2e AC 는 logic-level fallback |

---

## 17. 영문 보조 요약

This plan decomposes SPEC-V3-005 into 13 tasks across 3 milestones. T0 surfaces 3 USER-DECISION gates (moai-fs API shape, gpui test-support adoption, delete trash policy) and runs 2 Spikes (gpui test-support, trash crate). MS-1 (T1–T4) builds the FsNode domain model, cross-platform path normalization, optional moai-fs WorkspaceWatcher helper, and the FileExplorer Entity skeleton with RootView wiring. MS-2 (T5–T6) adds the 100ms debounce + rename matching + backpressure pipeline plus fs.yaml configuration. MS-3 (T7–T11) layers GitStatusProvider trait and MoaiGitStatusProvider, context menu with inline edit, delete dispatch (per USER-DECISION-C), drag-and-drop validation, and fuzzy search. T12 provides the integration harness; T13 runs regression sweep and writes progress.md. Each task maps explicitly to AC-FE-1 through AC-FE-12 from spec.md §8.

---

작성 완료: 2026-04-25
산출물: research.md / spec.md / plan.md (3-file 표준 준수).
