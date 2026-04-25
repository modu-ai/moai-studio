# SPEC-V3-008 Implementation Plan

작성: MoAI (manager-spec, 2026-04-25)
브랜치: 본 SPEC 작성은 `feature/SPEC-V3-004-render` (carry branch) 에서 수행. 구현 시점에는 CLAUDE.local.md §1.3 명명 규칙에 따라 `feature/SPEC-V3-008-git-ui` 신규 분기 권장.
범위: SPEC-V3-008 spec.md 의 RG-G-1 ~ RG-G-9, 13 AC, MS-1 / MS-2 / MS-3 + USER-DECISION 7 게이트.
선행: SPEC-V3-004 RootView (sidebar slot stable 확인 필수), SPEC-V3-006 syntax highlighter (graceful degradation 가능), SPEC-V3-005 File Explorer (병행, 읽기 전용 의존).

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|------|----------------------|-----|----|
| **T1** | MS-1 | USER-DECISION 게이트 — UD-1 (git2 vs gix), UD-3 (UI 통합 패턴), UD-4 (AI suggest opt-in) | (게이트 결정 기록) `.moai/specs/SPEC-V3-008/progress.md` | — | (게이트) |
| **T2** | MS-1 | moai-git 확장: index.rs (stage/unstage), commit.rs (commit) | `crates/moai-git/src/{index,commit}.rs` (신규), `lib.rs` re-export | T1 | AC-A-2, AC-A-3 |
| **T3** | MS-1 | GitStatusPanel Entity + Render | `crates/moai-studio-ui/src/git/{mod,status_panel}.rs` (신규) | T2 | AC-A-1, AC-A-5 |
| **T4** | MS-1 | GitCommitComposer Entity + Render + AI suggest hook (UD-4 결과 반영) | `crates/moai-studio-ui/src/git/commit_composer.rs` (신규) | T2 | AC-A-3, AC-A-4 |
| **T5** | MS-1 | RootView sidebar 통합 + status_bar branch widget | `crates/moai-studio-ui/src/lib.rs` (수정) | T3, T4 | AC-A-1 |
| **T6** | MS-2 | USER-DECISION 게이트 — UD-2 (diff mode), UD-5 (dirty 처리) | (게이트 결정 기록) progress.md | T5 | (게이트) |
| **T7** | MS-2 | moai-git 확장: diff.rs (Diff struct + diff_file/diff_workdir), branch.rs | `crates/moai-git/src/{diff,branch}.rs` (신규) | T6 | AC-A-6, AC-A-8, AC-A-9 |
| **T8** | MS-2 | GitDiffViewer Entity + Render + SPEC-V3-006 hook | `crates/moai-studio-ui/src/git/diff_viewer.rs` (신규) | T7 | AC-A-6, AC-A-7 |
| **T9** | MS-2 | GitBranchSwitcher Entity + Render + dirty confirm dialog | `crates/moai-studio-ui/src/git/branch_switcher.rs` (신규) | T7 | AC-A-8, AC-A-9 |
| **T10** | MS-3 | USER-DECISION 게이트 — UD-6 (graph 알고리즘), UD-7 (stash 범위) | (게이트 결정 기록) progress.md | T9 | (게이트) |
| **T11** | MS-3 | moai-git 확장: log/diff_commit, merge.rs, stash.rs | `crates/moai-git/src/{commit,merge,stash}.rs` (수정/신규) | T10 | AC-A-10~13 |
| **T12** | MS-3 | GitLogView Entity + Render + column-based graph | `crates/moai-studio-ui/src/git/log_view.rs` (신규) | T11 | AC-A-10, AC-A-11 |
| **T13** | MS-3 | GitMergeResolver + GitStashPanel | `crates/moai-studio-ui/src/git/{merge_resolver,stash_panel}.rs` (신규) | T11 | AC-A-12, AC-A-13 |
| **T14** | 전체 | regression + smoke test + commit | (git 작업, progress.md 갱신) | T1~T13 | 전체 |

본 SPEC v1.0.0 은 **시간 추정치 없음** (CLAUDE.md §"Time Estimation"). milestone 간 priority 만 명시: MS-1 = High (가시 가치 1 차), MS-2 = High, MS-3 = Medium (graph + merge 는 enabling but 즉각 사용 빈도 낮음).

---

## 2. T1 — USER-DECISION 게이트 1 차 (UD-1 / UD-3 / UD-4)

### 2.1 UD-1: git2-rs 유지 vs gix 전환

[USER-DECISION-REQUIRED: git-library-choice-v3-008]

질문 (AskUserQuestion):
- "moai-git crate 의 git 라이브러리를 어떻게 할까요?"
- (a) **권장: git2-rs (libgit2 0.20) 유지**. 기존 SPEC-V3-001 검증, merge resolver 성숙도 우수, 본 SPEC v1.0.0 즉시 진행 가능.
- (b) gix (gitoxide pure-rust) 전환. Windows 빌드 단순화, FFI 0. but merge 일부 미지원, API churn 위험. 별도 SPEC 필요.

권장안 (a) 채택 시: Cargo.toml 변경 없음, 본 SPEC v1.0.0 즉시 진행.
권장안 (b) 채택 시: 본 SPEC v1.0.0 차단, 별도 SPEC-V3-008-MIGRATION 분기 후 재진입.

### 2.2 UD-3: UI 통합 패턴

[USER-DECISION-REQUIRED: git-ui-integration-pattern-v3-008]

질문 (AskUserQuestion):
- "Git UI 를 어떻게 RootView 에 통합할까요?"
- (a) **권장: Hybrid C** — Status Panel = sidebar, Diff Viewer = leaf payload, Log View = 별도 탭. VS Code / Sublime Merge 패턴.
- (b) Sidebar 단독 — 모든 git UI 가 sidebar. RootView 변경 최소, but sidebar 비좁음.
- (c) 전용 Git Tab — TabContainer 의 한 탭 예약. RootView 무변경, but 사용자 학습 부담.

권장안 (a) 채택 시: spec.md §7.1 그대로 진행.
다른 선택 시: §7.1 다이어그램 재작성 후 본 SPEC version bump (v1.0.1).

### 2.3 UD-4: AI commit suggest

[USER-DECISION-REQUIRED: ai-commit-suggest-v3-008]

질문 (AskUserQuestion):
- "Commit Composer 에 AI 메시지 제안 기능을 어떻게 할까요?"
- (a) **권장: opt-in toggle, 기본 OFF**. SPEC-M2-001 의존, "Suggest" 버튼 표시 / SPEC-M2-001 미존재 시 hide.
- (b) opt-in toggle, 기본 ON. 동일 의존, but 사용자 명시적 비활성 필요.
- (c) 본 SPEC 에서 완전 제외. SPEC-V3-008-AI 별도 SPEC 분기.

권장안 (a) 채택 시: REQ-G-023 / REQ-G-081 그대로 진행, 기본 설정 `git_ui.ai_commit_suggest = false`.
권장안 (c) 채택 시: REQ-G-023 / REQ-G-081 / RG-G-3 의 AI 부분 별도 SPEC 으로 분리, 본 SPEC v1.0.0 에서 제거.

### 2.4 게이트 통과 후

세 결정 모두 progress.md 의 T1 항목에 다음 형식으로 기록:

```
## T1 — USER-DECISION Gates (2026-MM-DD)
- UD-1: [user-choice] (rationale: ...)
- UD-3: [user-choice] (rationale: ...)
- UD-4: [user-choice] (rationale: ...)
```

세 결정 중 하나라도 권장안 외 선택이면 spec.md 영향 분석 + version bump 후 T2 로 진행.

---

## 3. T2 — moai-git 확장: index.rs / commit.rs

### 3.1 index.rs 신규

`crates/moai-git/src/index.rs` (신규):

```rust
//! moai-git index manipulation: stage / unstage 메서드
//! @MX:NOTE: [AUTO] SPEC-V3-008 RG-G-1 (REQ-G-003/004) 의 GitRepo::stage / unstage 책임.
//!   호출자는 GitStatusPanel (T3) — fan_in 는 1 이지만 향후 file explorer 연동 가능.

use std::path::Path;
use crate::{GitRepo, GitError};

impl GitRepo {
    /// 단일 파일을 staged 상태로 만든다 (git add).
    pub fn stage(&self, path: &Path) -> Result<(), GitError> {
        let mut index = self.inner.index()?;
        index.add_path(path)?;
        index.write()?;
        Ok(())
    }

    /// staged 파일을 unstaged 로 되돌린다 (git reset HEAD path).
    pub fn unstage(&self, path: &Path) -> Result<(), GitError> {
        let head = self.inner.head()?.peel_to_commit()?;
        self.inner.reset_default(Some(head.as_object()), [path])?;
        Ok(())
    }
}
```

unit tests (lib.rs `#[cfg(test)]` mod 또는 별도 file): stage 후 status_map 변화 / unstage 후 status_map 복원 검증.

### 3.2 commit.rs 신규

`crates/moai-git/src/commit.rs` (신규):

```rust
//! moai-git commit creation + history listing.

use crate::{GitRepo, GitError};
use git2::Oid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub oid: String,
    pub short_oid: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,  // unix epoch
    pub summary: String,
    pub body: String,
    pub parent_oids: Vec<String>,
}

impl GitRepo {
    /// 새 커밋을 생성한다. signature 는 git config 에서 자동 추출.
    /// staged 파일 0 개이면 Err(GitError::EmptyCommit).
    pub fn commit(&self, message: &str) -> Result<Oid, GitError> { /* ... */ }

    /// 최근 N 개 commit 을 시간 역순으로 반환한다.
    pub fn log(&self, limit: usize) -> Result<Vec<CommitInfo>, GitError> { /* ... */ }
}
```

`GitError` 확장:
```rust
pub enum GitError {
    Git(#[from] git2::Error),
    DetachedHead,
    EmptyCommit,           // 신규 — REQ-G-022 호환
    AuthorNotConfigured,   // 신규 — REQ-G-024 호환
}
```

### 3.3 lib.rs 수정

`crates/moai-git/src/lib.rs` 끝에:
```rust
pub mod index;
pub mod commit;

pub use commit::CommitInfo;
```

기존 메서드 시그니처는 절대 무변경 (REQ-G-073).

---

## 4. T3 — GitStatusPanel Entity + Render

### 4.1 신규 파일

`crates/moai-studio-ui/src/git/mod.rs`:
```rust
pub mod status_panel;
pub mod commit_composer;
// MS-2/3 시점에 추가:
// pub mod diff_viewer;
// pub mod branch_switcher;
// pub mod log_view;
// pub mod merge_resolver;
// pub mod stash_panel;

pub use status_panel::GitStatusPanel;
pub use commit_composer::GitCommitComposer;
```

`crates/moai-studio-ui/src/git/status_panel.rs`:
```rust
//! @MX:ANCHOR: [AUTO] git-status-panel-entity
//! @MX:REASON: [AUTO] SPEC-V3-008 RG-G-1. fan_in >= 3 (T5 RootView 통합,
//!   T4 commit composer 가 status 변경 후 notify, T8 diff viewer 가 클릭 라우팅).

use gpui::*;
use moai_git::GitRepo;
use std::path::PathBuf;

pub struct GitStatusPanel {
    repo_root: PathBuf,
    staged: Vec<FileEntry>,
    unstaged: Vec<FileEntry>,
    untracked: Vec<FileEntry>,
    pending_refresh: Option<TaskHandle>,
}

#[derive(Clone)]
struct FileEntry { path: String, status: String /* "modified"|"added"|... */ }

impl GitStatusPanel {
    pub fn new(repo_root: PathBuf, cx: &mut Context<Self>) -> Self { /* ... */ }
    pub fn refresh(&mut self, cx: &mut Context<Self>) { /* spawn_blocking ... */ }
}

impl Render for GitStatusPanel {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // header + 3 sections (staged / unstaged / untracked)
        // 각 file row 는 click handler 등록 → cx.emit(FileClicked { path })
    }
}
```

이벤트 정의 (다른 entity 가 구독):
```rust
#[derive(Clone, Debug)]
pub enum GitStatusEvent {
    FileClicked { path: PathBuf },
    StageToggled { path: PathBuf },
}
```

### 4.2 unit tests

- `non_git_directory_returns_empty_panel`: `GitRepo::open` 실패 시 panic 없음, render 결과는 placeholder.
- `staged_unstaged_grouping_correct`: status_map fixture 로 그룹 분류 검증.
- `dirty_marker_visible_when_is_dirty`: REQ-G-006.

### 4.3 AC 매핑

- AC-A-1: staged / unstaged / untracked 3 그룹 렌더 → `staged_unstaged_grouping_correct`.
- AC-A-5: non-git directory 시 hide → `non_git_directory_returns_empty_panel`.

---

## 5. T4 — GitCommitComposer Entity + Render

### 5.1 신규 파일

`crates/moai-studio-ui/src/git/commit_composer.rs`:
```rust
pub struct GitCommitComposer {
    repo_root: PathBuf,
    message: String,         // textarea 내용
    author_name: String,     // git config 에서 로드
    author_email: String,
    staged_count: usize,     // GitStatusPanel 에서 갱신
    ai_suggest_enabled: bool, // UD-4 결정 + SPEC-M2-001 가용성 polling
    pending_commit: Option<TaskHandle>,
}

impl GitCommitComposer {
    pub fn new(repo_root: PathBuf, cx: &mut Context<Self>) -> Self { /* ... */ }
    pub fn handle_commit(&mut self, cx: &mut Context<Self>) { /* GitRepo::commit + spawn_blocking */ }
    pub fn handle_suggest(&mut self, cx: &mut Context<Self>) { /* SPEC-M2-001 호출 */ }
}

impl Render for GitCommitComposer {
    fn render(...) -> impl IntoElement {
        // textarea + author label + Commit/Suggest/Discard buttons
        // staged_count == 0 시 Commit 버튼 disabled (REQ-G-022)
        // ai_suggest_enabled 가 false 시 Suggest 버튼 hide (REQ-G-081)
    }
}
```

key bindings:
- Cmd+Enter (macOS) / Ctrl+Enter (Linux) → `handle_commit`
- Cmd+/ → `handle_suggest` (UD-4 활성 시)

### 5.2 unit tests

- `empty_message_disables_commit_button`: REQ-G-021.
- `zero_staged_disables_commit_button`: REQ-G-022, AC-A-4.
- `commit_clears_textarea_on_success`: REQ-G-025, AC-A-3.
- `author_not_configured_shows_error`: REQ-G-024.
- `suggest_button_hidden_when_subprocess_unavailable`: REQ-G-081.

### 5.3 AC 매핑

- AC-A-3: commit 성공 + textarea clear → `commit_clears_textarea_on_success`.
- AC-A-4: staged 0 시 disabled → `zero_staged_disables_commit_button`.

---

## 6. T5 — RootView sidebar 통합

### 6.1 변경 대상

`crates/moai-studio-ui/src/lib.rs` 의 `RootView::render` 의 sidebar 영역에 git 슬롯 삽입.

Before (개념적, SPEC-V3-004 진행 중 형태):
```rust
fn render(...) -> impl IntoElement {
    let sidebar = div().w_64().bg(rgb(0x252525))
        .child(workspaces_list(...))
        .child(files_section(...));
    /* main_body, status_bar */
}
```

After:
```rust
let sidebar = div().w_64().bg(rgb(0x252525))
    .child(workspaces_list(...))
    .child(files_section(...))
    // @MX:NOTE: [AUTO] SPEC-V3-008 RG-G-1 / RG-G-3 sidebar slot
    .child(self.git_status_panel.clone().map(|p| p.into_element()))
    .child(self.git_commit_composer.clone().map(|c| c.into_element()))
    .child(self.git_branch_switcher.clone().map(|b| b.into_element())) // MS-2
    .child(self.git_stash_panel.clone().map(|s| s.into_element()));    // MS-3
```

### 6.2 RootView struct 확장

```rust
pub struct RootView {
    // 기존 필드 ...
    pub git_status_panel: Option<Entity<git::GitStatusPanel>>,    // MS-1
    pub git_commit_composer: Option<Entity<git::GitCommitComposer>>, // MS-1
    pub git_branch_switcher: Option<Entity<git::GitBranchSwitcher>>, // MS-2
    pub git_stash_panel: Option<Entity<git::GitStashPanel>>,         // MS-3
}
```

각 필드는 active workspace 가 git repo 일 때만 `Some` 으로 채워진다 (REQ-G-082).

### 6.3 status_bar branch widget

status_bar 에 단순한 텍스트 widget 추가:
```rust
fn status_bar(...) -> impl IntoElement {
    div().h_6().px_2().flex().flex_row()
        .child(branch_widget(self.active_repo()))  // "● feature/SPEC-V3-008-git-ui ⚠ dirty"
        // 기존 status 항목들
}
```

---

## 7. T6 — USER-DECISION 게이트 2 차 (UD-2 / UD-5)

### 7.1 UD-2: Diff Viewer mode

[USER-DECISION-REQUIRED: diff-viewer-mode-v3-008]

질문 (AskUserQuestion):
- "Diff Viewer 의 표시 모드를 어떻게 할까요?"
- (a) **권장: Unified 만 v1.0.0 필수, side-by-side 는 best-effort**. 기본 모드는 unified, `git_ui.diff_view_mode = side_by_side` 설정 시 가능하면 2-column 제공.
- (b) Unified + side-by-side 둘 다 v1.0.0 필수. 작업량 약 1.5x.
- (c) Unified 만 v1.0.0, side-by-side 는 v1.1.0 별도 SPEC.

권장안 (a) 채택 시: REQ-G-015 그대로 (Optional EARS 패턴), 작업량 그대로.
권장안 (b) 채택 시: REQ-G-015 를 Ubiquitous 로 격상, T8 작업량 증가 → 본 SPEC version bump.

### 7.2 UD-5: Branch switch 시 dirty 처리

[USER-DECISION-REQUIRED: dirty-branch-switch-v3-008]

질문 (AskUserQuestion):
- "Dirty working tree 에서 브랜치 전환 시 어떻게 처리할까요?"
- (a) **권장: confirm dialog (discard / autostash / cancel)**. 사용자에게 선택권, autostash 채택 시 stash push + switch.
- (b) 무조건 차단 (cancel only). 데이터 손실 0, but UX 경직.
- (c) git default behavior 그대로 위임 (libgit2 가 차단하면 차단).

권장안 (a) 채택 시: REQ-G-032 그대로, T9 작업량 그대로.

### 7.3 게이트 통과 후

T1 과 동일하게 progress.md 에 결정 기록 후 T7 진행.

---

## 8. T7 — moai-git 확장: diff.rs / branch.rs

### 8.1 diff.rs 신규

`crates/moai-git/src/diff.rs`:
```rust
#[derive(Debug, Clone)]
pub struct Diff {
    pub file_path: String,
    pub hunks: Vec<Hunk>,
    pub language_hint: Option<String>,  // SPEC-V3-006 highlighter 입력용
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,  // Added | Removed | Context
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

impl GitRepo {
    /// 단일 파일의 working tree vs HEAD diff.
    pub fn diff_file(&self, path: &Path) -> Result<Diff, GitError> { /* ... */ }
    /// staged (index vs HEAD) diff.
    pub fn diff_index(&self, path: &Path) -> Result<Diff, GitError> { /* ... */ }
    /// 단일 commit 의 diff (parent vs commit).
    pub fn diff_commit(&self, oid: &str) -> Result<Vec<Diff>, GitError> { /* ... */ }
}
```

### 8.2 branch.rs 신규

```rust
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub is_head: bool,
    pub upstream: Option<String>,
}

impl GitRepo {
    pub fn list_branches(&self) -> Result<Vec<BranchInfo>, GitError> { /* ... */ }
    pub fn create_branch(&self, name: &str, from_oid: Option<&str>) -> Result<(), GitError> { /* ... */ }
    pub fn switch_branch(&self, name: &str) -> Result<(), GitError> { /* ... */ }
}
```

### 8.3 unit tests

- `diff_file_unified_format`: fixture (단일 hunk, 다중 hunk).
- `list_branches_includes_local_and_remote`.
- `create_branch_from_head`: 현재 HEAD 기반 새 브랜치 생성 검증.

---

## 9. T8 — GitDiffViewer Entity + Render

### 9.1 신규 파일

`crates/moai-studio-ui/src/git/diff_viewer.rs`:
```rust
pub struct GitDiffViewer {
    diff: Option<Diff>,
    view_mode: DiffViewMode,  // Unified | SideBySide (UD-2)
    highlighter: Option<Box<dyn HighlighterTrait>>, // SPEC-V3-006 hook
}

impl Render for GitDiffViewer {
    fn render(...) -> impl IntoElement {
        let Some(diff) = &self.diff else { return placeholder("No diff selected"); };
        match self.view_mode {
            Unified => render_unified(diff, &self.highlighter),
            SideBySide => render_side_by_side(diff, &self.highlighter),
        }
    }
}

fn render_unified(diff: &Diff, hl: &Option<Box<dyn HighlighterTrait>>) -> impl IntoElement {
    // 가상 스크롤 (visible viewport 만 렌더, REQ-G-014)
    // 각 라인:
    //   Added → bg(rgb(0x1f4d1f))   // dark green
    //   Removed → bg(rgb(0x4d1f1f)) // dark red
    //   Context → default
    // hl 가 Some 이면 highlight_line 결과 적용, None 이면 plain text (REQ-G-080)
}
```

### 9.2 SPEC-V3-006 hook 추상화

본 SPEC v1.0.0 시점에 SPEC-V3-006 가 미완성일 가능성 → trait 정의로 격리:
```rust
pub trait HighlighterTrait: Send + Sync {
    fn highlight_line(&self, line: &str, lang: &str) -> Vec<HighlightSpan>;
}

// SPEC-V3-006 완료 시 그 crate 가 impl 제공.
// 미완성 시 self.highlighter = None.
```

### 9.3 unit tests

- `unified_diff_renders_correct_colors`: AC-A-6.
- `plain_text_fallback_no_panic`: AC-A-7.
- `virtualization_for_100plus_hunks`: NFR-G-3 검증.

---

## 10. T9 — GitBranchSwitcher Entity + Render

### 10.1 신규 파일

`crates/moai-studio-ui/src/git/branch_switcher.rs`:
```rust
pub struct GitBranchSwitcher {
    repo_root: PathBuf,
    branches: Vec<BranchInfo>,
    search_query: String,
    pending_switch: Option<TaskHandle>,
}

impl GitBranchSwitcher {
    pub fn handle_click_branch(&mut self, name: &str, cx: &mut Context<Self>) {
        // is_dirty 검사
        if self.repo().is_dirty().unwrap_or(false) {
            self.show_dirty_dialog(name, cx);  // UD-5 권장안
        } else {
            self.do_switch(name, cx);
        }
    }

    pub fn handle_create_branch(&mut self, name: &str, cx: &mut Context<Self>) {
        // CLAUDE.local.md §1.2/1.3 명명 규칙 위반 시 경고 표시 (REQ-G-034, blocking 아님)
        if !is_valid_branch_name(name) { self.show_naming_warning(name); }
        self.repo().create_branch(name, None)?;
        self.do_switch(name, cx);
    }
}
```

### 10.2 dirty confirm dialog

option 선택지:
- "Discard changes and switch" → `git checkout -f <branch>` 동등.
- "Stash and switch" → `stash_push("auto-stash on branch switch")` → `switch_branch`.
- "Cancel".

### 10.3 unit tests

- `fuzzy_filter_case_insensitive`: 검색 동작 검증.
- `dirty_branch_switch_shows_dialog`: AC-A-8.
- `create_branch_with_feature_prefix`: AC-A-9.
- `naming_violation_warns_but_not_blocks`: REQ-G-034.

---

## 11. T10 — USER-DECISION 게이트 3 차 (UD-6 / UD-7)

### 11.1 UD-6: Log graph 알고리즘

[USER-DECISION-REQUIRED: log-graph-algorithm-v3-008]

질문 (AskUserQuestion):
- "Log View 의 commit graph 를 어떻게 그릴까요?"
- (a) **권장: 자체 구현 (column-based)**. parent oid 관계로 column 할당, GPUI 좌표계와 자연스러움. unit test 가능. 작업량 중간.
- (b) `git log --graph --oneline` subprocess 호출 + ASCII 파싱. 작업량 작음. but ASCII 파싱 fragile, 색상 / 클릭 통합 어려움.
- (c) 외부 crate (예: gix-graph). 의존성 추가, API 안정성 위험.

권장안 (a) 채택 시: REQ-G-042 그대로.

### 11.2 UD-7: Stash 범위

[USER-DECISION-REQUIRED: stash-scope-v3-008]

질문 (AskUserQuestion):
- "Stash 관리 기능 범위를 어떻게 할까요?"
- (a) **권장: list / push / pop / drop (4 핵심), apply best-effort**. v1.0.0 가시 핵심.
- (b) list / push / pop / drop / apply 모두 v1.0.0 필수. 작업량 약 1.2x.
- (c) list / push / pop 만 (drop / apply 제거). 데이터 안전성 ↑, but 사용자 불편.

권장안 (a) 채택 시: REQ-G-064 (Optional EARS) 그대로.

### 11.3 게이트 통과 후

progress.md 기록 후 T11 진행.

---

## 12. T11 — moai-git 확장: log/diff_commit, merge.rs, stash.rs

### 12.1 commit.rs 보강

T2 에서 정의한 `log` 메서드의 graph 정보 추가:
```rust
#[derive(Debug, Clone)]
pub struct CommitInfo {
    // 기존 필드
    pub parent_oids: Vec<String>,  // graph 알고리즘 입력 (UD-6)
}
```

### 12.2 merge.rs 신규

```rust
#[derive(Debug)]
pub enum MergeResult {
    FastForward { new_head: String },
    Merged { merge_commit: String },
    Conflict { conflicted_files: Vec<ConflictFile> },
}

#[derive(Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
    pub ours: String,    // HEAD 측 내용
    pub theirs: String,  // incoming 측 내용
    pub merged: String,  // working tree (with markers)
}

impl GitRepo {
    pub fn merge(&self, branch: &str) -> Result<MergeResult, GitError> { /* ... */ }
    pub fn conflict_files(&self) -> Result<Vec<ConflictFile>, GitError> { /* ... */ }
    pub fn abort_merge(&self) -> Result<(), GitError> { /* ... */ }
}
```

### 12.3 stash.rs 신규

```rust
#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub timestamp: i64,
}

impl GitRepo {
    pub fn stash_push(&self, message: &str) -> Result<(), GitError> { /* ... */ }
    pub fn stash_pop(&self, index: usize) -> Result<MergeResult, GitError> { /* conflict 가능 */ }
    pub fn stash_drop(&self, index: usize) -> Result<(), GitError> { /* ... */ }
    pub fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> { /* ... */ }
    pub fn stash_apply(&self, index: usize) -> Result<MergeResult, GitError> { /* best-effort, UD-7 */ }
}
```

### 12.4 unit tests

- `merge_fast_forward`: 단순 ff 케이스.
- `merge_with_conflict_returns_conflict_files`: REQ-G-051 source.
- `stash_push_pop_roundtrip`: AC-A-13.
- `log_with_branched_history`: graph 알고리즘 입력 검증.

---

## 13. T12 — GitLogView Entity + Render + column-based graph

### 13.1 신규 파일

`crates/moai-studio-ui/src/git/log_view.rs`:
```rust
pub struct GitLogView {
    commits: Vec<CommitInfo>,
    graph: Vec<GraphRow>,  // column-based graph (UD-6)
    pending_load: Option<TaskHandle>,
}

#[derive(Debug, Clone)]
struct GraphRow {
    column: usize,        // 0-based
    parent_columns: Vec<usize>,
    is_merge: bool,       // parents.len() >= 2
}

impl GitLogView {
    fn build_graph(commits: &[CommitInfo]) -> Vec<GraphRow> {
        // column-based 알고리즘:
        // 1. HEAD 부터 시작, column 0 할당.
        // 2. 각 commit 의 parent oid 를 추적.
        // 3. parent 가 미할당이면 next column 할당.
        // 4. merge commit 은 이전 column 들을 흡수.
        // 5. octopus (parents >= 3) 는 best-effort: 첫 2 parent 만 line drawn.
        // 자세한 의사 알고리즘은 본 SPEC research.md §3.5 향후 작성.
    }
}

impl Render for GitLogView {
    fn render(...) -> impl IntoElement {
        // 좌측 graph column (svg 또는 div+border 조합) + 우측 commit row
        // commit row 클릭 → cx.emit(LogEvent::CommitClicked { oid })
        // 최상단 "Uncommitted changes" 가상 row (REQ-G-044)
    }
}
```

### 13.2 unit tests

- `graph_linear_history_single_column`: AC-A-10 (linear case).
- `graph_branched_history_two_columns`: AC-A-10 (branched case).
- `graph_merge_commit_collapses_columns`: AC-A-10 (merge case).
- `commit_click_emits_event`: AC-A-11.

### 13.3 Octopus merge

v1.0.0 best-effort: parents >= 3 이면 graph 에 첫 2 parent line 만 표시, 나머지는 ":: more parents ::" 텍스트.

---

## 14. T13 — GitMergeResolver + GitStashPanel

### 14.1 GitMergeResolver

`crates/moai-studio-ui/src/git/merge_resolver.rs`:
```rust
pub struct GitMergeResolver {
    conflicts: Vec<ConflictFile>,
    active_index: usize,  // 현재 보고 있는 conflict 파일
}

impl Render for GitMergeResolver {
    fn render(...) -> impl IntoElement {
        // 상단 파일 리스트 + 본문 3 영역 (ours / theirs / merged)
        // 버튼: Accept Ours / Accept Theirs / Mark Resolved / Abort Merge
    }
}
```

자동 활성화는 RootView 가 polling 으로 conflict_files 비어있지 않음을 감지 후 leaf 를 GitMergeResolver 로 swap (REQ-G-051).

### 14.2 GitStashPanel

`crates/moai-studio-ui/src/git/stash_panel.rs`:
```rust
pub struct GitStashPanel {
    stashes: Vec<StashEntry>,
    pending: Option<TaskHandle>,
}

impl Render for GitStashPanel {
    fn render(...) -> impl IntoElement {
        // 헤더 ("Stashes (N)") + 리스트
        // 리스트 row 당: stash@{i} message + Pop / Drop / Apply (UD-7) buttons
        // 하단: Push Stash 버튼 + 메시지 input
    }
}
```

### 14.3 unit tests

- `merge_resolver_auto_activates_on_conflict`: AC-A-12 (REQ-G-051).
- `mark_resolved_calls_stage`: AC-A-12 (REQ-G-054).
- `stash_panel_shows_list`: AC-A-13.
- `stash_pop_with_conflict_routes_to_merge_resolver`: REQ-G-062 + REQ-G-051 통합.

---

## 15. T14 — Regression + Smoke + Commit

### 15.1 Regression matrix

각 milestone 종결 시:

```bash
# 1. moai-git unit tests
cargo test -p moai-git
# 기대: 기존 7 + T2/T7/T11 추가 tests

# 2. moai-studio-ui unit tests
cargo test -p moai-studio-ui
# 기대: SPEC-V3-002/003/004 기존 + 본 SPEC 신규

# 3. integration tests
cargo test -p moai-studio-ui --test integration_git_ui

# 4. workspace 빌드 (warning 0)
cargo build --workspace 2>&1 | grep "warning:" | wc -l
# 기대: 본 SPEC 진입 전과 동일

# 5. full clippy
cargo clippy --workspace --all-targets -- -D warnings
```

### 15.2 Smoke test (manual, macOS + Linux)

`cargo run -p moai-studio-app` 실행 후:

1. (MS-1) Sidebar 에 status panel 보임 + 테스트 파일 modify 후 그룹 분류 정확 + commit 성공 + textarea clear.
2. (MS-2) status panel 파일 클릭 → diff viewer 본문 등장 + branch switcher 에서 dummy 브랜치 전환 성공.
3. (MS-3) log view 탭 열기 → graph + commit 메시지 + commit 클릭 → diff 표시. 의도적 conflict 만들어서 merge 시도 → resolver 자동 등장 + Mark Resolved 동작. stash push → working tree clean → stash pop → 복원.

### 15.3 Commit (final)

`git add` → `git commit` 메시지:

```
docs(spec): SPEC-V3-008 Git Management UI v1.0.0 (research/plan/spec)

- moai-git crate fan_in 1 → 다(多) 안정화 SPEC. status_map 등 기존 API 시그니처 무변경.
- 7 영역 UI: status panel / diff viewer / commit composer / branch switcher / log view / merge resolver / stash panel.
- git2 = 0.20 lock-in (UD-1), Hybrid C 통합 패턴 (UD-3), AI suggest opt-in (UD-4).
- SPEC-V3-006 syntax highlight / SPEC-M2-001 Claude subprocess 미완성 시 graceful degradation (RG-G-9).
- AC-A-1 ~ AC-A-13, MS-1 / MS-2 / MS-3.

🗿 MoAI <email@mo.ai.kr>
```

push 금지 (사용자 요청에 명시).

---

## 16. AC ↔ Task ↔ Test 매트릭스

| AC | 책임 Task | Verification |
|----|----------|--------------|
| AC-A-1 (status panel 3 그룹) | T3 | unit `staged_unstaged_grouping_correct` + manual MS-1 |
| AC-A-2 (stage 토글) | T2, T3 | unit `stage_then_status_map_changes` + manual |
| AC-A-3 (commit 성공) | T2, T4 | integration `commit_clears_textarea_on_success` + manual |
| AC-A-4 (Commit 버튼 disabled) | T4 | unit `zero_staged_disables_commit_button` |
| AC-A-5 (non-git hide) | T3, T5 | unit `non_git_directory_returns_empty_panel` |
| AC-A-6 (unified diff colors) | T7, T8 | unit `unified_diff_renders_correct_colors` + manual |
| AC-A-7 (highlighter fallback) | T8 | unit `plain_text_fallback_no_panic` |
| AC-A-8 (branch switch) | T7, T9 | integration + manual MS-2 |
| AC-A-9 (new branch creation) | T7, T9 | unit `create_branch_with_feature_prefix` + manual |
| AC-A-10 (graph 3 cases) | T11, T12 | unit (linear / branched / merge fixtures) |
| AC-A-11 (commit click → diff) | T11, T12 | integration `commit_click_emits_event` + manual |
| AC-A-12 (merge resolver auto) | T11, T13 | integration `merge_resolver_auto_activates_on_conflict` + manual |
| AC-A-13 (stash push/pop) | T11, T13 | integration `stash_push_pop_roundtrip` + manual |

---

## 17. 위험 모니터링 (research.md §7 carry)

| ID | 트리거 / 모니터 | 대응 |
|----|---------------|------|
| R1 (git2 0.21 출시) | T7 / T11 진입 시 `cargo update` dry-run | 0.20 pin 유지, 별도 SPEC 으로 migration |
| R2 (merge corruption) | T13 통합 테스트 실패 | abort_merge 우선 검증, REQ-G-055 |
| R3 (UI freeze on large repo) | NFR-G-2 ~ NFR-G-5 측정 | spawn_blocking 미적용 코드 즉시 fix |
| R4 (V3-006 미완) | MS-2 진입 시 highlighter trait impl 가용성 확인 | RG-G-9 fallback path 활성 |
| R5 (Claude Code subprocess crash) | UD-4 채택 + T4 시 `which claude` 체크 | REQ-G-081 hide |
| R6 (graph 알고리즘 bug) | T12 unit fixture 추가 시 | bug 발견 시 v1.0.1 patch SPEC |
| R7 (gix 결정 번복) | UD-1 결정 후 변경 요청 | 본 SPEC v1.0.0 차단 + SPEC-V3-008-MIG 분기 |
| R10 (외부 CLI race) | manual smoke test | best-effort retry, 사용자 경고 |

---

## 18. 종결 조건 (Sprint Exit, spec.md §12 carry)

본 SPEC 가 PASS 로 닫히려면:

1. AC-A-1 ~ AC-A-13 모두 PASS (또는 carry-over 명시).
2. UD-1 ~ UD-7 모두 progress.md 에 결정 기록 + 사용자 final 확인.
3. SPEC-V3-002/003/004 의 모든 기존 tests regression 0.
4. moai-git 기존 7 tests regression 0 + 본 SPEC 신규 tests 모두 PASS.
5. `cargo build --workspace` warning 증가 0, `cargo clippy --workspace -- -D warnings` PASS.
6. macOS 14 + Ubuntu 22.04 양쪽 manual smoke test PASS.
7. CLAUDE.local.md §1 명명 규칙 위반 0 (브랜치 / 커밋 메시지).

---

작성 완료: 2026-04-25
