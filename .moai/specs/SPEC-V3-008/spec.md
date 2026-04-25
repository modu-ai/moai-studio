---
id: SPEC-V3-008
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-004, SPEC-V3-005, SPEC-V3-006]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, git, vision]
revision: v1.0.0 (initial draft, moai-git fan_in 안정화 + UI 통합)
---

# SPEC-V3-008: Git Management UI — status / diff / commit / branch / log / merge / stash

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio v0.1.0 비전의 4 축 중 (3) Git 통합. 기존 `moai-git` crate (`git2 = 0.20`) 의 fan_in 1 → 다(多) 확장. RG-G-1 ~ RG-G-7 의 7 개 기능 그룹 + RG-G-8/9 비변경/완화 정책. AC-A-1 ~ AC-A-13 의 13 개 acceptance criteria. USER-DECISION 7 게이트 (UD-1 ~ UD-7) 명시. SPEC-V3-004 RootView + SPEC-V3-006 syntax-highlight 의존 차단 시 graceful degradation. |

---

## 1. 개요

### 1.1 목적

moai-studio 의 사용자 가시 git 작업을 단일 UI 로 통합한다. 기존 터미널 의존 (`git status`, `git diff`, `git log` CLI) 을 GPUI 위 native widget 으로 대체하여 (1) Status Panel, (2) Diff Viewer, (3) Commit Composer, (4) Branch Switcher, (5) Log View, (6) Merge Conflict Resolver, (7) Stash Management 의 7 개 영역을 제공한다.

본 SPEC 의 핵심 원칙:

- **`moai-git` crate 우선 활용**: 기존 `crates/moai-git/src/lib.rs` 의 공개 API 를 확장하되, 신규 git 라이브러리 도입은 별도 SPEC. 본 SPEC v1.0.0 은 `git2 = 0.20` 유지 (UD-1 결정).
- **터미널 / pane / tab logic 무변경**: SPEC-V3-002 terminal core 와 SPEC-V3-003 panes / tabs 의 공개 API 를 절대 수정하지 않는다 (RG-G-8 carry).
- **Graceful degradation**: SPEC-V3-006 syntax highlight 미완성 시 plain-text 폴백, AI commit suggest (UD-4) 미채택 시 toggle hide.

### 1.2 근거 문서

- `.moai/specs/SPEC-V3-008/research.md` — moai-git API surface 조사, 6 영역 UI 구조, 위험 요약, USER-DECISION 게이트 정의.
- `.moai/specs/SPEC-V3-008/plan.md` — milestone × task 분해, 일정 분리, AC 매핑.
- `crates/moai-git/src/lib.rs` — 기존 API (open / current_branch / is_dirty / status_map / status_summary).
- `crates/moai-git/Cargo.toml` — `git2 = "0.20"` lock-in.
- `crates/moai-studio-ui/src/lib.rs` — RootView 구조 (SPEC-V3-004 진행 중).
- SPEC-V3-004 spec.md §7.1 — RootView 의 sidebar / content_area / status_bar 통합 지점.
- SPEC-V3-006 (선행) — syntax highlight 모듈 의존 (RG-G-2).
- SPEC-V3-005 (병행) — File Explorer 가 본 SPEC `status_map` 소비 (읽기 전용).
- CLAUDE.local.md §1 — Enhanced GitHub Flow (feature/* / hotfix/* / release/*) 와 본 UI 의 branch_switcher 동작 일관성.

---

## 2. 배경 및 동기

상세 분석은 `.moai/specs/SPEC-V3-008/research.md` §1 ~ §8 참조. SPEC 독자가 요구사항 진입 전에 알아야 할 최소 맥락만 요약한다.

- **비전 위치** (research §1.1): moai-studio v0.1.0 의 사용자 가시 가치 4 축 (pane/tab 셸, editor+viewer, **git 통합**, AI 통합) 중 본 SPEC 이 (3). "git-aware terminal IDE" 차별화의 단일 핵심.
- **moai-git fan_in 폭발** (research §1.2): 본 SPEC 도입 후 `GitRepo::open` 의 caller 가 1 → 7, `status_map` 가 0 → 3 으로 증가. fan_in ≥ 3 트리거로 `@MX:ANCHOR` 등록 필수.
- **git2 lock-in** (research §2.3): 본 SPEC v1.0.0 은 `git2 = 0.20` 유지. gix 전환은 v0.2.0+ 별도 SPEC.
- **UI 통합 패턴 — Hybrid C** (research §3.1): Status Panel = sidebar, Diff Viewer = leaf payload, Log View = 별도 탭. SPEC-V3-004 RootView 위에서 자연스러운 배치.
- **SPEC-V3-006 syntax highlight 의존**: 선행 SPEC. 미완성 시 본 SPEC MS-2 는 plain-text 로 진행 후 별도 PR 로 활성화 (RG-G-9 carry).

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. 7 개 영역 (status / diff / commit / branch / log / merge / stash) 모두에 대해 GPUI native widget 을 제공한다.
- G2. `crates/moai-git/src/lib.rs` 의 공개 API 를 확장하되, **기존 메서드 시그니처는 보존** 한다. 추가는 가능, 제거 / 변경 금지.
- G3. 사용자가 `cargo run -p moai-studio-app` 으로 (a) repo 의 modified 파일 가시, (b) 한 파일 클릭 시 diff 가시, (c) 메시지 입력 + Cmd/Ctrl+Enter 로 커밋 성공, (d) 브랜치 전환 가시, (e) commit log graph 가시, (f) merge conflict 시 3-way diff 가시, (g) stash list / push / pop 가시 — 7 가지를 직접 관찰할 수 있다.
- G4. SPEC-V3-002 terminal 과 SPEC-V3-003 panes/tabs 의 공개 API 는 변경하지 않는다.
- G5. 모든 git2 호출은 UI thread 를 차단하지 않는다 (NFR-G-1).
- G6. 사용자가 `feature/*` / `hotfix/*` / `release/*` 등 CLAUDE.local.md §1 명명 규칙에 따른 브랜치를 UI 에서 그대로 생성 / 전환할 수 있다.
- G7. macOS 14+ / Ubuntu 22.04+ 양쪽에서 동일한 동작 (status accuracy / diff 표시 / commit 성공) 을 보장한다.

### 3.2 비목표 (Non-Goals)

- N1. **Remote 작업 (push / pull / fetch / clone)** — 본 SPEC v1.0.0 은 **로컬 작업 한정**. push/pull 은 별도 SPEC. terminal 내 `git push` 명령은 그대로 사용 가능.
- N2. **Rebase / cherry-pick UI** — interactive rebase, cherry-pick 는 별도 SPEC.
- N3. **Submodule / LFS 관리 UI** — 별도 SPEC.
- N4. **gix (gitoxide) 전환** — UD-1 결정에 따라 git2 유지. gix 는 v0.2.0+ 비전.
- N5. **Diff side-by-side 모드 필수화** — UD-2 결정. v1.0.0 은 unified 만 필수. side-by-side 는 best-effort.
- N6. **Settings UI** — git_ui 설정은 YAML 직접 편집. UI Settings Panel 별도 SPEC.
- N7. **Pane persistence schema 변경** — SPEC-V3-003 panes-v1 schema 는 본 SPEC 에서 read 만. Git 탭 leaf payload 영속화는 별도 SPEC.
- N8. **AI commit message 자동 적용** — UD-4 채택 시에도 사용자 명시적 클릭 / 단축키 필요. 자동 작성 후 자동 커밋 금지.
- N9. **Worktree 관리 UI** — `WorktreeManager` (moai-git/src/worktree.rs) 의 UI 노출은 별도 SPEC.
- N10. **Windows 빌드** — SPEC-V3-002/003/004 N10 와 동일.

---

## 4. 사용자 스토리

- **US-G1**: 개발자가 사이드바의 Git 섹션을 열면 staged / unstaged / untracked 3 그룹으로 분류된 파일 리스트가 보인다 → `GitRepo::status_map` 호출 결과를 group-by 한 결과 표시.
- **US-G2**: 개발자가 unstaged 파일을 클릭하면 본문에 해당 파일의 unified diff 가 syntax-highlighted 로 보인다 → diff_viewer entity 활성화 + SPEC-V3-006 highlight 모듈 호출.
- **US-G3**: 개발자가 unstaged 파일 좌측 아이콘을 클릭하면 staged 그룹으로 이동한다 → `GitRepo::stage(path)` 호출 + 즉시 re-render.
- **US-G4**: 개발자가 commit composer 의 textarea 에 메시지를 입력하고 Cmd/Ctrl+Enter 를 누르면 staged 변경분이 커밋되고 staged 그룹이 비워진다 → `GitRepo::commit(msg, author, email)` 호출 후 `cx.notify()`.
- **US-G5**: 개발자가 branch switcher 를 열고 브랜치 이름을 검색 후 클릭하면 working tree 가 그 브랜치로 전환된다. dirty 상태이면 confirm dialog 가 먼저 뜬다 → UD-5 결정 적용.
- **US-G6**: 개발자가 log view 탭을 열면 commit graph 와 메시지가 시간 역순으로 보이고, 한 commit 을 클릭하면 그 커밋의 변경 내역 (diff) 이 본문에 등장한다 → `GitRepo::log(50)` + per-commit `diff_commit(oid)` 호출.
- **US-G7**: 개발자가 다른 브랜치를 머지하다 conflict 가 발생하면 본문이 자동으로 conflict resolver 모드로 전환되어 ours / theirs / merged 영역을 표시한다 → `GitRepo::conflict_files()` polling + UI 자동 라우팅.
- **US-G8**: 개발자가 stash push 버튼을 누르면 현재 working tree 변경분이 stash 로 저장되고 working tree 가 깨끗해진다 → `GitRepo::stash_push(msg)`.

---

## 5. 기능 요구사항 (EARS)

### RG-G-1 — Status Panel (staged / unstaged / untracked file list)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-001 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/status_panel.rs` (신규) 에 GPUI Entity 인 `GitStatusPanel` 을 제공한다. 해당 entity 는 `impl Render` 를 구현한다. | The system **shall** provide a `GitStatusPanel` GPUI Entity in `git/status_panel.rs` implementing `Render`. |
| REQ-G-002 | Event-Driven | 활성 workspace 가 git repo 인 동안, 시스템은 매 50 ms 이상의 간격으로 (또는 명시적 refresh 트리거 시) `GitRepo::status_map` 을 호출하여 staged / unstaged / untracked 3 그룹으로 분류된 파일 리스트를 렌더한다. | When the active workspace is a git repo, the system **shall** poll `status_map` at intervals >= 50ms (or on explicit refresh) and render files grouped into staged / unstaged / untracked. |
| REQ-G-003 | Event-Driven | 사용자가 unstaged 또는 untracked 파일의 stage 토글 아이콘을 클릭하면, 시스템은 `GitRepo::stage(path)` 를 호출하고 성공 시 `cx.notify()` 를 발화한다. | When the user clicks the stage toggle icon on an unstaged/untracked file, the system **shall** call `stage(path)` and notify on success. |
| REQ-G-004 | Event-Driven | 사용자가 staged 파일의 unstage 아이콘을 클릭하면, 시스템은 `GitRepo::unstage(path)` 를 호출하고 성공 시 `cx.notify()` 를 발화한다. | When the user clicks the unstage icon on a staged file, the system **shall** call `unstage(path)` and notify on success. |
| REQ-G-005 | Unwanted | 시스템은 활성 workspace 가 git repo 가 아닌 경우 (`GitRepo::open` 실패) `GitStatusPanel` 을 렌더하지 않으며 panic 하지 않는다. | The system **shall not** render `GitStatusPanel` or panic when the workspace is not a git repo. |
| REQ-G-006 | State-Driven | 활성 repo 의 `is_dirty()` 가 `true` 인 동안, 시스템은 status panel header 에 시각적 dirty marker (예: 경고 아이콘 + "modified" 텍스트) 를 표시한다. | While the active repo is dirty, the system **shall** display a dirty marker in the status panel header. |

### RG-G-2 — Diff Viewer (unified diff with syntax highlight)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-010 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/diff_viewer.rs` (신규) 에 GPUI Entity 인 `GitDiffViewer` 를 제공한다. 해당 entity 는 단일 파일의 unified diff 를 렌더한다. | The system **shall** provide a `GitDiffViewer` GPUI Entity rendering a single file's unified diff. |
| REQ-G-011 | Event-Driven | 사용자가 status panel 에서 파일을 클릭하면, 시스템은 `GitRepo::diff_file(path)` 를 호출하고 결과 hunks 를 `GitDiffViewer` 에 로드한다. | When the user clicks a file in the status panel, the system **shall** call `diff_file(path)` and load hunks into `GitDiffViewer`. |
| REQ-G-012 | Ubiquitous | 시스템은 `Diff::hunks` 를 라인 단위로 렌더하며, `-` 라인은 빨간 배경, `+` 라인은 초록 배경, context 라인은 기본 배경을 사용한다. | The system **shall** render diff hunks line-by-line with red background for `-` lines, green for `+`, default for context. |
| REQ-G-013 | State-Driven | SPEC-V3-006 syntax highlight 모듈이 가용한 동안, 시스템은 각 라인의 코드 부분에 `highlight_line(line, lang)` 결과를 적용한다. 미가용 시 plain-text 로 fallback 한다 (RG-G-9). | While the SPEC-V3-006 highlighter is available, the system **shall** apply `highlight_line` to code portions; fall back to plain text if not. |
| REQ-G-014 | Event-Driven | `GitDiffViewer` 가 hunks 가 100 개 이상인 diff 를 받으면, 시스템은 가상 스크롤 (visible viewport 만 렌더) 을 적용하여 첫 paint ≤ 50 ms 를 보장한다. | When hunks count >= 100, the system **shall** virtualize rendering to ensure first-paint <= 50ms. |
| REQ-G-015 | Optional | UD-2 가 side-by-side 모드를 채택한 경우, 시스템은 `git_ui.diff_view_mode = side_by_side` 설정 시 좌측 ours / 우측 theirs 의 2 column 레이아웃을 제공할 수 있다. v1.0.0 에서는 best-effort. | Where UD-2 enables side-by-side, the system **may** provide a 2-column layout when `diff_view_mode = side_by_side`. |

### RG-G-3 — Commit Composer (message editor + AI suggestion hook)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-020 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/commit_composer.rs` (신규) 에 GPUI Entity 인 `GitCommitComposer` 를 제공한다. 해당 entity 는 (a) 메시지 textarea, (b) author 표시, (c) Commit 버튼, (d) Discard 버튼을 포함한다. | The system **shall** provide a `GitCommitComposer` GPUI Entity with a message textarea, author display, Commit button, and Discard button. |
| REQ-G-021 | Event-Driven | 사용자가 textarea 에 비어있지 않은 메시지를 입력하고 Cmd+Enter (macOS) 또는 Ctrl+Enter (Linux) 를 누르거나 Commit 버튼을 클릭하면, 시스템은 `GitRepo::commit(msg, author, email)` 을 호출한다. | When the user inputs a non-empty message and presses Cmd/Ctrl+Enter or clicks Commit, the system **shall** call `commit(msg, author, email)`. |
| REQ-G-022 | State-Driven | staged 파일 수가 0 인 동안, 시스템은 Commit 버튼을 비활성 (disabled) 상태로 유지하고 Cmd/Ctrl+Enter 단축키도 무시한다. | While staged file count is 0, the system **shall** keep the Commit button disabled and ignore Cmd/Ctrl+Enter. |
| REQ-G-023 | Optional | UD-4 가 AI commit suggest 를 활성화한 경우, 시스템은 "Suggest" 버튼을 표시하고 클릭 시 staged diff 를 SPEC-M2-001 의 Claude Code subprocess 로 보내어 메시지 초안을 생성한다. UD-4 비채택 시 버튼은 표시되지 않는다. | Where UD-4 enables AI suggest, the system **shall** display a "Suggest" button that sends staged diff to Claude Code subprocess. |
| REQ-G-024 | Unwanted | 시스템은 author 또는 email 이 git config 에 설정되지 않은 경우 commit 을 시도하지 않으며, 명확한 에러 메시지 (`git config user.name 미설정`) 를 composer 하단에 표시한다. | The system **shall not** attempt commit if author/email is not configured; show explicit error in composer footer. |
| REQ-G-025 | Event-Driven | commit 이 성공하면, 시스템은 textarea 를 비우고 `cx.notify()` 를 발화하여 status panel 의 staged 그룹이 새로 fetch 되도록 한다. | When commit succeeds, the system **shall** clear the textarea and notify so the status panel re-fetches. |

### RG-G-4 — Branch Switcher + Creator

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-030 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/branch_switcher.rs` (신규) 에 GPUI Entity 인 `GitBranchSwitcher` 를 제공한다. 해당 entity 는 로컬 + 원격 (read-only) 브랜치 리스트와 검색 입력을 포함한다. | The system **shall** provide a `GitBranchSwitcher` GPUI Entity with local + remote (read-only) branch list and search input. |
| REQ-G-031 | Event-Driven | 사용자가 검색 입력에 텍스트를 입력하면, 시스템은 fuzzy match (case-insensitive substring) 로 브랜치 리스트를 필터링한다. | When the user inputs search text, the system **shall** filter branches by fuzzy match (case-insensitive substring). |
| REQ-G-032 | Event-Driven | 사용자가 브랜치를 클릭하면, 시스템은 `GitRepo::is_dirty()` 가 `true` 인 경우 confirm dialog (UD-5: discard / autostash / cancel) 를 먼저 표시한 후 `GitRepo::switch_branch(name)` 을 호출한다. | When the user clicks a branch, if `is_dirty()`, the system **shall** show a confirm dialog (discard / autostash / cancel) before calling `switch_branch`. |
| REQ-G-033 | Event-Driven | 사용자가 "+ New Branch" 버튼을 클릭하고 이름 (예: `feature/SPEC-V3-009-foo`) 을 입력 후 Enter 를 누르면, 시스템은 `GitRepo::create_branch(name, HEAD)` 를 호출하고 새 브랜치로 전환한다. | When the user enters a new branch name and presses Enter, the system **shall** call `create_branch(name, HEAD)` and switch to it. |
| REQ-G-034 | Unwanted | 시스템은 CLAUDE.local.md §1.2/1.3 명명 규칙 (feature/* / hotfix/* / release/*) 위반 이름에 대해 경고를 표시하나, 입력은 차단하지 않는다 (사용자 결정 존중). | The system **shall not** block but **shall** warn on names violating CLAUDE.local.md branch naming rules. |
| REQ-G-035 | State-Driven | switch_branch 가 진행 중인 동안, 시스템은 GitBranchSwitcher 의 모든 상호작용을 일시 차단하고 loading spinner 를 표시한다. | While `switch_branch` is in progress, the system **shall** block interactions and show a loading spinner. |

### RG-G-5 — Log View (commit graph + messages)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-040 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/log_view.rs` (신규) 에 GPUI Entity 인 `GitLogView` 를 제공한다. 해당 entity 는 commit 리스트 + graph column + 메시지 / author / 단축 oid 를 렌더한다. | The system **shall** provide a `GitLogView` GPUI Entity rendering commit list + graph column + message / author / short oid. |
| REQ-G-041 | Event-Driven | `GitLogView` 가 처음 렌더되거나 사용자가 refresh 를 트리거하면, 시스템은 `GitRepo::log(limit)` (limit 은 `git_ui.log_limit` 설정값, 기본 50) 을 호출하여 commit 리스트를 로드한다. | When `GitLogView` initializes or refreshes, the system **shall** call `log(limit)` with `git_ui.log_limit` (default 50). |
| REQ-G-042 | Ubiquitous | 시스템은 UD-6 결정에 따라 column-based graph 알고리즘 (자체 구현) 으로 parent oid 관계를 시각화한다. octopus merge (parents >= 3) 는 v1.0.0 에서 best-effort. | The system **shall** visualize parent oid relationships using a column-based graph algorithm (UD-6 native impl); octopus merges are best-effort in v1.0.0. |
| REQ-G-043 | Event-Driven | 사용자가 commit row 를 클릭하면, 시스템은 `GitRepo::diff_commit(oid)` 를 호출하고 결과를 `GitDiffViewer` 에 로드한다. | When the user clicks a commit row, the system **shall** call `diff_commit(oid)` and load into `GitDiffViewer`. |
| REQ-G-044 | State-Driven | working tree 에 commit 되지 않은 변경분이 있는 동안, 시스템은 log view 최상단에 "Uncommitted changes" 가상 row 를 표시한다 (HEAD 위쪽). | While there are uncommitted changes, the system **shall** display an "Uncommitted changes" virtual row above HEAD. |

### RG-G-6 — Merge Conflict Resolver (3-way diff)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-050 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/merge_resolver.rs` (신규) 에 GPUI Entity 인 `GitMergeResolver` 를 제공한다. 해당 entity 는 ours / theirs / merged 3 영역을 표시한다. | The system **shall** provide a `GitMergeResolver` GPUI Entity displaying ours / theirs / merged 3 regions. |
| REQ-G-051 | Event-Driven | repo 가 merge conflict 상태에 진입하면 (`GitRepo::conflict_files()` 가 비어있지 않음), 시스템은 본문 영역을 자동으로 `GitMergeResolver` 로 전환한다. | When the repo enters merge conflict state, the system **shall** auto-switch the main area to `GitMergeResolver`. |
| REQ-G-052 | Ubiquitous | 시스템은 conflict 파일 각각에 대해 ours (HEAD) / theirs (incoming) / merged (working tree with markers) 3 영역의 텍스트를 표시한다. merged 영역은 read-only 또는 SPEC-V3-006 editor 가 도입되면 편집 가능. v1.0.0 에서는 read-only 표시 + 외부 편집 후 refresh 패턴. | The system **shall** display ours/theirs/merged for each conflict file; merged is read-only in v1.0.0 (editable when SPEC-V3-006 editor lands). |
| REQ-G-053 | Event-Driven | 사용자가 "Accept Ours" 버튼을 클릭하면, 시스템은 해당 파일의 working tree 내용을 ours 로 교체하고 `GitRepo::stage(path)` 를 호출한다. "Accept Theirs" 도 동일 패턴. | When the user clicks "Accept Ours" or "Accept Theirs", the system **shall** replace the working tree content and call `stage(path)`. |
| REQ-G-054 | Event-Driven | 사용자가 "Mark Resolved" 버튼을 클릭하면, 시스템은 `GitRepo::stage(path)` 를 호출하여 conflict 상태를 해소한다. | When the user clicks "Mark Resolved", the system **shall** call `stage(path)` to clear conflict state. |
| REQ-G-055 | Event-Driven | 사용자가 "Abort Merge" 버튼을 클릭하면, 시스템은 `GitRepo::abort_merge()` 를 호출하여 머지 시도 전 상태로 복원한다. | When the user clicks "Abort Merge", the system **shall** call `abort_merge()` to restore pre-merge state. |
| REQ-G-056 | State-Driven | merge 가 활성 상태인 동안, 시스템은 status panel header 와 commit composer 에 "Merging" 표시를 한다. commit composer 의 Commit 버튼 라벨은 "Commit Merge" 로 변경된다. | While merge is active, the system **shall** display "Merging" indicator and change Commit button label to "Commit Merge". |

### RG-G-7 — Stash Management

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-060 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/git/stash_panel.rs` (신규) 에 GPUI Entity 인 `GitStashPanel` 을 제공한다. 해당 entity 는 stash 리스트와 push / pop / drop 버튼을 포함한다. | The system **shall** provide a `GitStashPanel` GPUI Entity with stash list and push/pop/drop buttons. |
| REQ-G-061 | Event-Driven | 사용자가 "Push Stash" 버튼을 클릭하고 메시지를 입력하면, 시스템은 `GitRepo::stash_push(msg)` 를 호출하고 working tree 가 깨끗해지면 stash 리스트를 재로드한다. | When the user clicks "Push Stash" and enters a message, the system **shall** call `stash_push(msg)` and reload list. |
| REQ-G-062 | Event-Driven | 사용자가 stash row 의 "Pop" 버튼을 클릭하면, 시스템은 `GitRepo::stash_pop(idx)` 를 호출한다. conflict 발생 시 자동으로 `GitMergeResolver` 로 전환 (RG-G-6 의 REQ-G-051 carry). | When the user clicks "Pop", the system **shall** call `stash_pop(idx)`; on conflict, auto-switch to merge resolver. |
| REQ-G-063 | Event-Driven | 사용자가 stash row 의 "Drop" 버튼을 클릭하면, 시스템은 confirm dialog 후 `GitRepo::stash_drop(idx)` 를 호출한다. | When the user clicks "Drop", the system **shall** confirm and call `stash_drop(idx)`. |
| REQ-G-064 | Optional | 시스템은 "Apply" 버튼 (stash 를 working tree 에 적용하되 stash 자체는 보존) 을 v1.0.0 에서 best-effort 로 제공할 수 있다. 미구현 시 UI 에 표시되지 않는다. | Where feasible, the system **may** provide an "Apply" button (apply without removing) in v1.0.0. |

### RG-G-8 — 무변경 영역 (RG-P-7 carry from SPEC-V3-002/003/004)

| REQ ID | 패턴 | 요구사항 (한국어) |
|--------|------|-------------------|
| REQ-G-070 | Ubiquitous | 시스템은 `crates/moai-studio-terminal/**` 의 어떤 파일도 수정하지 않는다. SPEC-V3-002 의 13 tests 가 본 SPEC 모든 milestone 에서 regression 0 으로 유지된다. |
| REQ-G-071 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/{panes, tabs, terminal}/**` 의 공개 API 를 변경하지 않는다. SPEC-V3-002/003 의 unit tests 와 SPEC-V3-004 의 render layer 통합 결과가 그대로 보존된다. |
| REQ-G-072 | Ubiquitous | 시스템은 `crates/moai-studio-workspace/src/persistence.rs` 의 SPEC-V3-003 MS-3 산출 schema (`moai-studio/panes-v1`) 를 변경하지 않는다. 본 SPEC 은 read 만 허용. |
| REQ-G-073 | Ubiquitous | 시스템은 `crates/moai-git/src/lib.rs` 의 기존 공개 메서드 (`open`, `init`, `current_branch`, `is_dirty`, `status_map`, `status_summary`) 의 시그니처를 변경하지 않는다. 신규 메서드 추가는 허용. |
| REQ-G-074 | Ubiquitous | 시스템은 `git2` crate 버전을 `0.20` 에서 변경하지 않는다 (UD-1). gix 전환은 별도 SPEC. |

### RG-G-9 — Graceful Degradation (의존 SPEC 미완성 대응)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-G-080 | State-Driven | SPEC-V3-006 syntax highlight 모듈이 가용하지 않은 동안, 시스템은 `GitDiffViewer` 의 코드 영역을 plain text 로 렌더하며 panic 하지 않는다. | While SPEC-V3-006 highlighter is unavailable, the system **shall** render diff code as plain text without panic. |
| REQ-G-081 | State-Driven | SPEC-M2-001 Claude Code subprocess 가 가용하지 않은 동안, 시스템은 commit composer 의 "Suggest" 버튼을 hide 한다 (UD-4 채택 여부와 무관). | While Claude Code subprocess is unavailable, the system **shall** hide the "Suggest" button regardless of UD-4. |
| REQ-G-082 | State-Driven | 활성 workspace 가 git repo 가 아닌 동안, 시스템은 모든 git UI 영역 (status / diff / commit / branch / log / merge / stash) 을 hide 하며 placeholder ("Not a git repository") 를 sidebar 에 표시한다. | While the workspace is not a git repo, the system **shall** hide all git UI areas and show a placeholder. |

---

## 6. 비기능 요구사항

### 6.1 성능

- NFR-G-1. 모든 git2 호출은 UI thread 를 차단하지 않는다. `tokio::task::spawn_blocking` 또는 별도 worker thread 사용. UI 는 작업 중 loading state 를 표시한다.
- NFR-G-2. status panel 첫 paint 시간 ≤ 100 ms (1000 파일 미만 repo 기준).
- NFR-G-3. diff viewer 첫 paint 시간 ≤ 50 ms (단일 파일, hunks 100 미만 기준).
- NFR-G-4. log view 첫 paint 시간 ≤ 200 ms (`git_ui.log_limit = 50` 기본값 기준).
- NFR-G-5. branch switch 작업 시간 ≤ 2 s (working tree 100 MB 미만 기준). 초과 시 loading spinner + 진행 상태 표시.
- NFR-G-6. commit 작업 시간 ≤ 500 ms (staged 파일 50 미만 기준).

### 6.2 안정성

- NFR-G-7. `GitRepo` 의 어떤 메서드도 panic 하지 않는다 (모든 오류는 `Result<_, GitError>` 로 반환).
- NFR-G-8. merge conflict 도중 앱이 충돌 / 종료되어도 `git status` CLI 는 그대로 conflict 상태를 인식할 수 있어야 한다 (advisory lock 등 추가 state 도입 금지).
- NFR-G-9. `cargo run -p moai-studio-app` 5 분 idle 후 git2 관련 메모리 사용량 증가 ≤ 5 MB.
- NFR-G-10. 외부 CLI (`git commit`, `git checkout`) 와 UI 의 동시 작업 시 race 가 발생해도 데이터 손실 없음. UI 는 best-effort retry + 사용자에게 명시적 경고.

### 6.3 접근성

- NFR-G-11. 모든 git UI widget 은 키보드만으로 조작 가능하다 (Tab 으로 focus 이동, Enter 로 활성화, Esc 로 닫기). v1.0.0 에서는 best-effort.
- NFR-G-12. status panel / branch switcher / log view 의 항목은 GPUI 의 list role 또는 등가 의미 구조를 사용한다.

### 6.4 호환성

- NFR-G-13. macOS 14 + Ubuntu 22.04 양쪽에서 동일한 git 작업 결과 (status accuracy / commit hash / branch list) 를 보장한다.
- NFR-G-14. `git config user.name` / `user.email` 미설정 환경에서 commit 시도는 명확한 에러 메시지 (REQ-G-024) 를 반환하고 panic 하지 않는다.

### 6.5 보안

- NFR-G-15. AI commit suggest (UD-4 채택 시) 는 staged diff 만 Claude Code subprocess 로 보내며, 다른 파일 / 환경 변수 / credentials 는 노출하지 않는다.
- NFR-G-16. branch switcher 의 검색 입력은 shell escape 가 필요한 외부 명령에 그대로 전달되지 않는다 (git2 API 만 사용).

---

## 7. 아키텍처

### 7.1 RootView 통합 지점 (Hybrid C, UD-3)

```
RootView (SPEC-V3-004 진행 중)
├── title_bar
├── main_body
│   ├── sidebar
│   │   ├── (existing) Workspaces / Files
│   │   └── Git
│   │       ├── GitStatusPanel        ← RG-G-1
│   │       ├── GitCommitComposer     ← RG-G-3
│   │       ├── GitBranchSwitcher     ← RG-G-4 (collapsible)
│   │       └── GitStashPanel         ← RG-G-7 (collapsible)
│   └── content_area
│       └── tab_container
│           └── PaneTree<LeafPayload>
│               ├── TerminalSurface (기존)
│               └── LeafPayload::GitView::Diff(GitDiffViewer)   ← RG-G-2
│               └── LeafPayload::GitView::Log(GitLogView)        ← RG-G-5
│               └── LeafPayload::GitView::Merge(GitMergeResolver) ← RG-G-6
└── status_bar
    └── (new) Branch + dirty marker widget
```

- LeafPayload enum 확장은 본 SPEC 의 신규 항목. SPEC-V3-004 의 PaneTree<L> generic 위에 작동.
- panes-v1 영속화는 `GitView::*` variant 를 알지 못해도 무방 (read 만, in-memory 신규 leaf).

### 7.2 git2 호출 thread 모델

```
UI Thread (GPUI main loop)
   │
   ▼ event (click, key)
   │
   ▼ entity.update(cx, |e, cx| e.trigger_git_call(...))
       │
       ▼ tokio::task::spawn_blocking(move || {
       │     let repo = GitRepo::open(path)?;
       │     repo.<git2_call>()
       │  })
       │
       ▼ await result
       │
       ▼ entity.update(cx, |e, cx| {
            e.apply_result(result);
            cx.notify();
         })
```

- `cx.spawn` 또는 `cx.background_executor()` 등 GPUI 0.2.2 가 제공하는 비동기 패턴 사용 (MS-1 시점 SPEC-V3-001 spike 결과 재활용).
- 각 entity 는 진행 중인 git 작업을 1 개로 제한하기 위해 `pending: Option<TaskHandle>` 필드 보유.

### 7.3 moai-git crate 확장 영역

```
moai-git/src/lib.rs (기존)
   ↓ 확장
moai-git/src/
├── lib.rs              — re-exports + GitRepo struct (기존 + 확장)
├── worktree.rs         — 무변경 (SPEC-V3-001)
├── diff.rs             — 신규: Diff struct, hunks, diff_file, diff_workdir, diff_commit
├── index.rs            — 신규: stage, unstage
├── commit.rs           — 신규: commit, log (Vec<CommitInfo>)
├── branch.rs           — 신규: list_branches, create_branch, switch_branch, BranchInfo
├── merge.rs            — 신규: merge, conflict_files, abort_merge, MergeResult, ConflictFile
└── stash.rs            — 신규: stash_push, stash_pop, stash_drop, stash_list, stash_apply
```

각 모듈은 `pub use` 로 lib.rs 에 노출. 기존 메서드 시그니처는 손대지 않음 (REQ-G-073).

---

## 8. Milestone

본 SPEC 은 3 milestone 으로 분할한다. milestone 간 regression gate 는 SPEC-V3-002/003 정책 carry.

### MS-1: Status Panel + Commit Composer (basic) — REQ-G-001 ~ REQ-G-006, REQ-G-020 ~ REQ-G-025, RG-G-8 / RG-G-9 부분

- **범위**: GitStatusPanel + GitCommitComposer + sidebar 통합. moai-git 확장: `diff_file`, `stage`, `unstage`, `commit`. UD-1 (git2 유지), UD-3 (Hybrid C), UD-4 (AI suggest opt-in) 결정.
- **포함 요구사항**: RG-G-1 전체, RG-G-3 전체, RG-G-8 전체, RG-G-9 의 REQ-G-082.
- **시연 가능 상태**: `cargo run -p moai-studio-app` 실행 시 사이드바에 modified / staged / untracked 파일이 보이고, 메시지 입력 + Cmd/Ctrl+Enter 로 커밋 성공.

### MS-2: Diff Viewer + Branch Switcher — REQ-G-010 ~ REQ-G-015, REQ-G-030 ~ REQ-G-035

- **범위**: GitDiffViewer (unified, plain-text or syntax-highlighted) + GitBranchSwitcher. moai-git 확장: `diff_file`, `list_branches`, `create_branch`, `switch_branch`. UD-2 (unified 필수), UD-5 (dirty 처리) 결정.
- **포함 요구사항**: RG-G-2 전체, RG-G-4 전체, RG-G-9 의 REQ-G-080/081.
- **시연 가능 상태**: status panel 의 파일 클릭 시 diff 가 본문 leaf 에 등장. branch switcher 에서 브랜치 전환 시 status / diff 가 새 HEAD 기준으로 갱신.

### MS-3: Log Graph + Merge Conflict Resolver + Stash — REQ-G-040 ~ REQ-G-064

- **범위**: GitLogView (graph + 메시지) + GitMergeResolver (3-way) + GitStashPanel. moai-git 확장: `log`, `diff_commit`, `merge`, `conflict_files`, `abort_merge`, `stash_*`. UD-6 (column-based graph), UD-7 (stash 범위) 결정.
- **포함 요구사항**: RG-G-5 전체, RG-G-6 전체, RG-G-7 전체.
- **시연 가능 상태**: 별도 탭 (Log) 에서 commit graph 표시 + commit 클릭 시 diff 등장. 머지 시도 시 conflict 자동 감지 + resolver 표시 + Mark Resolved 동작. stash push / pop 가시.

---

## 9. 파일 레이아웃 (canonical)

### 9.1 신규

- `crates/moai-studio-ui/src/git/mod.rs` — re-exports.
- `crates/moai-studio-ui/src/git/status_panel.rs` — `GitStatusPanel` Entity + Render.
- `crates/moai-studio-ui/src/git/diff_viewer.rs` — `GitDiffViewer` Entity + Render.
- `crates/moai-studio-ui/src/git/commit_composer.rs` — `GitCommitComposer` Entity + Render.
- `crates/moai-studio-ui/src/git/branch_switcher.rs` — `GitBranchSwitcher` Entity + Render.
- `crates/moai-studio-ui/src/git/log_view.rs` — `GitLogView` Entity + Render + graph 알고리즘.
- `crates/moai-studio-ui/src/git/merge_resolver.rs` — `GitMergeResolver` Entity + Render.
- `crates/moai-studio-ui/src/git/stash_panel.rs` — `GitStashPanel` Entity + Render.
- `crates/moai-git/src/diff.rs` — `Diff`, `Hunk`, `Line`, `diff_file`, `diff_workdir`, `diff_commit`, `diff_index`.
- `crates/moai-git/src/index.rs` — `stage`, `unstage`.
- `crates/moai-git/src/commit.rs` — `commit`, `log`, `CommitInfo`.
- `crates/moai-git/src/branch.rs` — `list_branches`, `create_branch`, `switch_branch`, `BranchInfo`.
- `crates/moai-git/src/merge.rs` — `merge`, `conflict_files`, `abort_merge`, `MergeResult`, `ConflictFile`.
- `crates/moai-git/src/stash.rs` — `stash_push`, `stash_pop`, `stash_drop`, `stash_list`, `stash_apply`.
- `crates/moai-studio-ui/tests/integration_git_ui.rs` — TestAppContext 기반 통합 테스트 (MS-1/2/3 별).

### 9.2 수정

- `crates/moai-git/src/lib.rs` — 신규 모듈 re-export. `GitError` enum variant 추가 (`BranchSwitchFailed`, `CommitFailed`, `MergeConflictUnresolved`, `StashFailed`). 기존 메서드 무변경.
- `crates/moai-git/Cargo.toml` — `git2 = "0.20"` 유지. dev-dependencies 에 `tempfile`, `tokio` (test runtime) 추가.
- `crates/moai-studio-ui/src/lib.rs` — RootView 의 sidebar 슬롯에 git 영역 통합. status_bar 에 branch widget 추가. git mod 선언.
- `crates/moai-studio-ui/Cargo.toml` — `moai-git` workspace dependency 추가 (이미 모종 형태로 있을 수 있음, MS-1 시점 검증).
- `.moai/config/sections/git_ui.yaml` (신규) — `diff_view_mode`, `log_limit`, `ai_commit_suggest`, `graph_algorithm` 기본값.

### 9.3 비변경 (REQ-G-070 ~ REQ-G-074 carry)

- `crates/moai-studio-terminal/**` — 어떤 파일도 변경 금지.
- `crates/moai-studio-ui/src/{panes, tabs, terminal}/**` — 공개 API 변경 금지.
- `crates/moai-studio-workspace/src/persistence.rs` — schema 변경 금지.
- `crates/moai-git/src/{lib.rs, worktree.rs}` 의 기존 공개 메서드 시그니처 — 변경 금지.

---

## 10. 의존성 / 위험 (요약, 상세는 research.md §7)

| ID | 의존 / 위험 | 영향 | 완화 |
|----|------------|------|------|
| D1 | SPEC-V3-004 RootView 진행 중 | sidebar / leaf 통합 지점 변동 가능 | MS-1 진입 시 SPEC-V3-004 완료 또는 sidebar slot stable 확인. 미완 시 본 SPEC delay. |
| D2 | SPEC-V3-006 syntax highlight 미완성 | Diff Viewer plain-text fallback | RG-G-9 graceful degradation. 별도 PR 로 활성화 가능. |
| D3 | SPEC-V3-005 File Explorer 병행 | status_map 동시 호출 | read-only 의존, advisory lock 불필요. cache 도입 검토 (별도 SPEC). |
| D4 | SPEC-M2-001 Claude Code subprocess 미완성 | UD-4 AI suggest hide | RG-G-9 의 REQ-G-081 polling. |
| R1 | git2 0.20 → 0.21 breaking | 빌드 실패 | Cargo.toml 명시 pin (REQ-G-074). |
| R2 | 대형 repo UI freeze | UX 저하 | NFR-G-1 (spawn_blocking). |
| R3 | log graph 자체 구현 버그 | 잘못된 시각화 | unit tests with fixture commit graphs. |

---

## 11. Acceptance Criteria (13)

| AC ID | 설명 | 검증 방법 | Milestone |
|-------|------|----------|-----------|
| **AC-A-1** | `GitStatusPanel` 이 staged / unstaged / untracked 3 그룹으로 분류된 파일 리스트를 렌더한다 | unit test (status_map mock) + manual `cargo run` | MS-1 |
| **AC-A-2** | unstaged 파일의 stage 토글 클릭 시 staged 그룹으로 이동 | unit test (stage call mock) + manual | MS-1 |
| **AC-A-3** | commit composer 에 메시지 입력 + Cmd/Ctrl+Enter 시 커밋 성공, staged 그룹 비워짐 | integration test (TestAppContext) + manual | MS-1 |
| **AC-A-4** | staged 파일 0 개일 때 Commit 버튼 disabled, 단축키 무시 | unit test (REQ-G-022) | MS-1 |
| **AC-A-5** | non-git directory 에서 GitStatusPanel hide + placeholder 표시 (REQ-G-082) | unit test (open Err mock) | MS-1 |
| **AC-A-6** | `GitDiffViewer` 가 unified diff 를 -/+/context 색상으로 렌더 | unit test (Diff fixture) + manual | MS-2 |
| **AC-A-7** | SPEC-V3-006 highlighter 미가용 시 plain-text fallback (panic 없음) | unit test (highlighter trait mock) | MS-2 |
| **AC-A-8** | branch switcher 검색 + 클릭으로 브랜치 전환, dirty 시 confirm dialog | integration test + manual | MS-2 |
| **AC-A-9** | branch switcher "+ New Branch" 로 feature/SPEC-XXX-foo 형식 생성 + 즉시 전환 | integration test + manual | MS-2 |
| **AC-A-10** | `GitLogView` 가 column-based graph 와 commit 메시지 시간 역순 렌더 | unit test (fixture: linear / branched / merge) | MS-3 |
| **AC-A-11** | log view commit row 클릭 시 diff_commit 결과 등장 | integration test + manual | MS-3 |
| **AC-A-12** | merge conflict 자동 감지 + GitMergeResolver 자동 전환, "Mark Resolved" 동작 | integration test (conflict fixture) + manual | MS-3 |
| **AC-A-13** | stash push 후 working tree 깨끗, stash pop 후 복원 | integration test + manual | MS-3 |

---

## 12. 종결 조건 (Sprint Exit Criteria)

본 SPEC 의 PASS 는 다음 조건이 모두 만족되어야 한다:

1. AC-A-1 ~ AC-A-13 모두 PASS (또는 carry-over 처리 명시).
2. SPEC-V3-002 (terminal) 의 13 tests + SPEC-V3-003 (panes/tabs) 의 53 unit + 2 integration tests + SPEC-V3-004 의 render layer tests 가 regression 0 으로 유지.
3. moai-git 의 기존 7 tests (lib.rs + worktree.rs) 가 regression 0 으로 유지.
4. `cargo build -p moai-git -p moai-studio-ui -p moai-studio-app` 에서 warning 증가 0.
5. `cargo run -p moai-studio-app` 에서 위 §1.3 의 7 가지 사용자 가시 동작 모두 수동 검증 PASS (macOS 14 + Ubuntu 22.04).
6. UD-1 ~ UD-7 의 7 결정 게이트가 모두 progress.md 에 기록되고, 사용자 final 확인 완료.
7. CLAUDE.local.md §1 명명 규칙 위반 없음 (본 SPEC 작업 브랜치 + 생성 가능한 브랜치 모두).

---

## 13. 용어 정의

- **staged**: `git add` 완료 (index 에 등록) 상태.
- **unstaged**: working tree 에는 변경, index 미반영 상태.
- **untracked**: git 이 아직 인지하지 못한 새 파일.
- **HEAD**: 현재 브랜치의 최신 commit.
- **hunk**: diff 의 연속된 변경 블록 (e.g. `@@ -10,3 +10,5 @@`).
- **3-way diff**: ours (HEAD), theirs (incoming), merged (working tree with conflict markers) 의 3 영역 diff.
- **fast-forward merge**: HEAD 가 target 의 ancestor 인 경우의 머지 (단순 pointer 이동).
- **octopus merge**: parents >= 3 인 머지 (v1.0.0 best-effort).
- **stash**: working tree 변경분을 임시 저장한 stack.

---

작성 완료: 2026-04-25
