---
id: SPEC-V0-3-0-STATUS-BAR-WIRE-002
version: "1.0.0"
status: draft
created_at: 2026-05-06
updated_at: 2026-05-06
author: GOOS행님
priority: Medium
labels: [v0.3.0, sprint-2, lightweight, status-bar, git-integration]
issue_number: null
---

# SPEC-V0-3-0-STATUS-BAR-WIRE-002 — StatusBar git2 Integration (real branch name + dirty marker)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-STATUS-BAR-WIRE-002 |
| **Title** | StatusBar git label graduation — `derive_status_git_label_from_workspace` placeholder → `moai-git::GitRepo` 실제 호출 |
| **Status** | draft (Sprint 2 #8 carry-from #7) |
| **Priority** | Medium |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-3-0-STATUS-BAR-WIRE-001 (#109 머지, MS-1 — `derive_status_git_label_from_workspace` placeholder skeleton 도입), `moai-git` crate (이미 git2 0.20 wrapper 존재 — `GitRepo::open` / `current_branch` / `is_dirty` / `DetachedHead`) |
| **Cycle** | v0.3.0 Sprint 2 #8 (carry-from #7 STATUS-BAR-WIRE-001 §"Carry-to (c)") |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-06: 초안 작성. SPEC-V0-3-0-STATUS-BAR-WIRE-001 (PR #109 + spec.md 보완 PR #110) 의 carry-to 항목 (c) "실제 git2 crate 통합 (`derive_status_git_label_from_workspace` 본체 교체)" 를 단독으로 graduate. #109 의 MS-1 은 `derive_status_git_label_from_workspace(workspace_id: &str) -> Option<(String, bool)>` 를 placeholder 로 도입했고 (lib.rs:3853, "비어있지 않은 ID → `Some((id.to_string(), false))` / 빈 ID → `None`"), `handle_activate_workspace` (lib.rs:865~881) 가 workspace 전환 직후 이를 호출해 status bar 의 git 라벨을 갱신하는 구조까지 wire 했다. 본 SPEC 은 그 placeholder body 만 `moai-git::GitRepo` 호출로 교체하여 status bar 가 실제 branch 이름 + dirty marker 를 반영하도록 한다. AC 수 (≤8) / milestones (≤2) 모두 lightweight 한도 충족.

## 1. Purpose / 배경

PR #109 의 `derive_status_git_label_from_workspace` 는 workspace **식별자 (ID)** 만 받아 그 문자열 자체를 라벨로 echo back 하는 placeholder 였다. 사용자 관점에서는 status bar 가 실제 branch 이름 (`main`, `feature/SPEC-V0-3-0-STATUS-BAR-WIRE-002` 등) 도 보여주지 못하고, dirty 여부 (uncommitted changes) 도 표시하지 못한다.

`moai-git` crate 는 SPEC-V3-008 흐름에서 이미 git2 0.20 wrapper 를 완비했다 (`crates/moai-git/src/lib.rs:46~`):
- `GitRepo::open(path: &Path) -> Result<Self, GitError>` — 저장소 오픈
- `GitRepo::current_branch(&self) -> Result<String, GitError>` — HEAD 가 branch 면 short name, detached 이면 `GitError::DetachedHead`
- `GitRepo::is_dirty(&self) -> Result<bool, GitError>` — 워킹 트리 + 인덱스 종합 dirty 여부
- `GitError` enum — `Git(git2::Error)` / `DetachedHead` 두 variant

또한 `moai-studio-workspace::Workspace` 는 `project_path: PathBuf` 필드를 보유 (lib.rs:34) — 실제 git 저장소 경로로 그대로 사용 가능.

본 SPEC 은 다음 단일 graduation 만 수행한다:
1. `derive_status_git_label_from_workspace` 의 input 시그니처를 path 기반으로 확장 (`workspace_path: &Path`) 하고 body 를 `GitRepo` 호출로 교체
2. 호출 측 (`refresh_status_git_label`) 이 active workspace 의 `project_path` 를 전달하도록 갱신
3. error / non-git / detached-head 모두 graceful 매핑 (panic 금지)

신규 widget UI, async/cached refactor, multi-window 동기화, palette `status.refresh_git` 의 entry 추가는 모두 본 SPEC 범위 밖.

## 2. Goals / 목표

- `derive_status_git_label_from_workspace` 가 `&Path` 입력에서 `GitRepo::open` 호출로 실제 git 저장소를 탐지하고, branch 가 존재하면 `(branch_name, dirty_bool)` 반환
- non-git 디렉터리 (예: `~/Documents/notes/`) 입력 시 `None` 반환 (status bar 라벨 clear) — `git2` 의 RepoNotFound 등 에러는 `None` 으로 swallow
- detached HEAD 상태에서 `GitError::DetachedHead` 가 발생하면 fallback 라벨 (`"detached"` 7-byte 라벨 + `is_dirty` 결과) 반환, panic 금지
- `is_dirty` 자체 실패 시 dirty 를 `false` 로 가정하고 branch 만 표시 (graceful)
- `refresh_status_git_label` (lib.rs:470) 이 active workspace 의 `project_path` 를 인자로 전달 — 기존 `active_id` 기반 호출은 폐기
- `handle_activate_workspace` (lib.rs:865~881) 의 wire 경로 (workspace 전환 → refresh) 는 무수정 — 호출 인자만 path 로 변경
- TRUST 5 gates (clippy / fmt / cargo test) ALL PASS, 기존 ui crate 1374 tests 회귀 0 (additive only, +4 fixture-based integration tests)

## 3. Non-Goals / Exclusions

- 실시간 git 상태 폴링 (FROZEN — workspace 전환 시 1회만 호출, file watcher 통합은 별 SPEC)
- async / 캐싱 / 백그라운드 worker 도입 (REQ-SBW-008 가 명시 — sync 동기 호출 유지, 향후 프로파일링 결과로만 별 SPEC 진입)
- `palette/registry.rs` 에 `status.refresh_git` entry 추가 (FROZEN — #109 carry-to (a) 별 SPEC, 본 SPEC 은 helper 본체 교체만)
- Multi-window 의 status bar 동기화 (single-window 만 대상)
- `status_bar.rs` 모듈 내부 변경 (FROZEN — render / setter API 무수정)
- `agent_mode` 파이프라인 (`set_agent_mode` payload 메커니즘) — #109 carry-to (b), 별 SPEC
- LSP polling → `set_lsp_status` wire — #109 carry-to (d), 별 SPEC
- 신규 git 작업 (commit / push / branch switch UI) — `moai-git` crate 자체 SPEC 의 영역
- detached HEAD 시 short SHA 표시 (본 SPEC 은 고정 라벨 `"detached"` 만 — short SHA 진화는 별 SPEC carry-to)
- worktree (`.git` 가 file 인 경우) 또는 submodule 특수 처리 — `git2::Repository::open` 의 기본 동작 (`open_from_env` 미사용) 위임

FROZEN (touch 금지):
- `crates/moai-git/**` (read-only — `GitRepo::open` / `current_branch` / `is_dirty` / `GitError` 사용만)
- `crates/moai-studio-workspace/**` (read-only — `Workspace.project_path` 필드 read 만)
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-ui/src/status_bar.rs` (전체 read-only — `StatusBarState` API 호출만)
- `crates/moai-studio-ui/src/palette/registry.rs` (본 SPEC 무수정)
- 기존 `dispatch_command` 의 `status.*` 분기 (#109 routing 결과 — 무수정)
- `handle_activate_workspace` 의 wire 흐름 (인자만 변경, 흐름 자체 무수정)
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 4. Touchpoints / 현행 scaffolding 위치

- `crates/moai-studio-ui/src/lib.rs:3845~3859` — `derive_status_git_label_from_workspace(workspace_id: &str) -> Option<(String, bool)>` placeholder body (본 SPEC 교체 대상)
- `crates/moai-studio-ui/src/lib.rs:465~477` — `refresh_status_git_label(&mut self, cx: &mut Context<Self>)` cx-bound helper (인자 source 변경 대상 — `active_id` → active workspace 의 `project_path`)
- `crates/moai-studio-ui/src/lib.rs:865~881` — `handle_activate_workspace` (`refresh_status_git_label` 호출 지점, 무수정)
- `crates/moai-studio-ui/src/lib.rs:6937~6960` — 기존 unit tests `derive_status_git_label_returns_workspace_id` / `derive_status_git_label_empty_id_returns_none` (본 SPEC 에서 path-based fixture test 로 교체)
- `crates/moai-git/src/lib.rs:46~120` — `GitRepo` / `current_branch` / `is_dirty` / `GitError` (read-only 사용)
- `crates/moai-studio-workspace/src/lib.rs:31~38` — `Workspace.project_path: PathBuf` (active workspace lookup 후 전달)
- `crates/moai-studio-ui/Cargo.toml` — `moai-git` workspace dependency 추가 (현재 미존재)

## 5. Requirements

- REQ-SBW-008: `derive_status_git_label_from_workspace` 의 시그니처는 `pub fn derive_status_git_label_from_workspace(workspace_path: &std::path::Path) -> Option<(String, bool)>` 로 변경된다. 입력은 workspace ID 가 아닌 실제 디렉터리 경로다.
- REQ-SBW-009: 본체는 `moai_git::GitRepo::open(workspace_path)` 를 호출한다. 반환이 `Err(_)` (non-git 디렉터리, RepoNotFound, IO 실패 등) 이면 `None` 반환 (라벨 clear 신호) — panic 금지, log 는 `tracing::debug!` 레벨로만.
- REQ-SBW-010: `GitRepo::open` 성공 후 `current_branch()` 호출. `Ok(name)` 이면 라벨에 그대로 사용. `Err(GitError::DetachedHead)` 이면 라벨을 고정 문자열 `"detached"` (7-byte ASCII) 로 사용. 그 외 `Err(GitError::Git(_))` 는 `None` 반환 (graceful fallback, panic 금지).
- REQ-SBW-011: branch 라벨 결정 후 `is_dirty()` 호출. `Ok(b)` 이면 그대로 사용. `Err(_)` 는 `false` 로 가정 (graceful — 라벨은 보존, dirty marker 만 생략).
- REQ-SBW-012: 최종 반환은 `Some((branch_label, dirty_bool))`. dirty marker 의 시각화 (●, *, 등) 는 status_bar.rs 의 기존 `set_git_branch(label, dirty)` 시그니처가 dirty bool 을 받아 처리하므로 본 helper 는 bool 만 전달 (FROZEN — `status_bar.rs` 무수정).
- REQ-SBW-013: `RootView::refresh_status_git_label` 은 `self.active_id` 를 사용하여 `WorkspacesStore` (또는 in-memory cache) 에서 active workspace 를 lookup 하여 `Workspace.project_path` 를 추출한다. lookup 실패 (active_id None / store 미로드 / id 매치 실패) 시 `clear_git_branch` 호출 (라벨 clear) 후 early return — `cx.notify()` 는 호출 (재렌더 보장).
- REQ-SBW-014: `derive_status_git_label_from_workspace` 호출은 동기 (sync) 다 — workspace 전환 이벤트 자체가 이벤트 드라이븐이므로 일반적인 저장소 크기에서 동기 호출은 허용. 프로파일링 결과 perceptible blocking 이 관측되면 async / 캐시 refactor 는 별 SPEC.
- REQ-SBW-015: 기존 `derive_status_git_label_returns_workspace_id` / `derive_status_git_label_empty_id_returns_none` unit tests 는 path-based fixture tests 로 교체된다 (clean repo / dirty repo / non-git directory / detached HEAD 4 시나리오).

## 6. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-SBW-7 | tempfile 로 생성된 클린 git 저장소 (`git init` + `git commit --allow-empty -m initial`) | `derive_status_git_label_from_workspace(repo_path)` 호출 | `Some((branch_name, false))` 반환. branch_name 은 `"main"` 또는 `"master"` (git init default 에 의존) — 테스트는 두 값 모두 허용 | unit test (`derive_status_git_label_clean_repo_returns_branch_no_dirty`) |
| AC-SBW-8 | tempfile 로 생성된 git 저장소 + 한 파일을 워킹 트리에 unstaged 로 수정 | `derive_status_git_label_from_workspace(repo_path)` 호출 | `Some((branch_name, true))` 반환 (dirty=true) | unit test (`derive_status_git_label_dirty_repo_returns_dirty_true`) |
| AC-SBW-9 | tempfile 로 생성된 plain 디렉터리 (git init 없음) | `derive_status_git_label_from_workspace(non_git_path)` 호출 | `None` 반환 (status bar label clear 신호) — panic 없음 | unit test (`derive_status_git_label_non_git_directory_returns_none`) |
| AC-SBW-10 | tempfile 로 생성된 git 저장소 + commit 후 detached HEAD 진입 (`set_head_detached(commit_oid)`) | `derive_status_git_label_from_workspace(repo_path)` 호출 | `Some(("detached".to_string(), dirty_bool))` 반환. dirty_bool 은 `is_dirty()` 결과 (보통 false) — 라벨은 정확히 `"detached"` | unit test (`derive_status_git_label_detached_head_returns_fixed_label`) |
| AC-SBW-11 | 존재하지 않는 경로 (`/tmp/definitely-not-existing-path-{uuid}`) | `derive_status_git_label_from_workspace(missing_path)` 호출 | `None` 반환 — panic 없음, error log 만 | unit test (`derive_status_git_label_missing_path_returns_none`) (REQ-SBW-009 graceful path 검증, AC-SBW-9 와 별개로 IO 실패 경로 명시 검증) |
| AC-SBW-12 | cargo build / clippy / fmt + ui crate test | run | ALL PASS, 기존 ui crate 1374 tests 회귀 0 (기존 placeholder 검증 2 tests 는 본 SPEC 의 fixture-based tests 5개로 교체 → net +3 tests) | CI |

(AC 합계: 6. lightweight 한도 ≤8 충족. 모두 fixture 기반 integration test 로 검증.)

## 7. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | `derive_status_git_label_from_workspace` body 교체 (`&str` → `&Path`, placeholder echo → `GitRepo` 호출), `refresh_status_git_label` 의 lookup 경로 갱신 (`active_id` → workspace path resolution), 기존 unit tests 2개 제거 후 fixture-based tests 5개 추가 (T-SBW2 블록), `use moai_git::{GitRepo, GitError}` import 추가 |
| `crates/moai-studio-ui/Cargo.toml` | modified | `moai-git = { workspace = true }` 또는 path dep 신규 추가 (workspace dep 우선; manager-tdd 에서 workspace Cargo.toml 정합성 확인 후 적용). `tempfile` 은 dev-dependency 로 신규 추가 (이미 존재 시 무수정) |
| `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-002/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-002/progress.md` | created | run 진입 시 manager-tdd 가 갱신 stub |

추가 파일 없음. `crates/moai-git/**` 와 `crates/moai-studio-workspace/**` 는 read-only — 기존 API 호출 + 필드 read 만.

FROZEN (touch 금지):
- `crates/moai-git/**` (전체 read-only)
- `crates/moai-studio-workspace/**` (전체 read-only)
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-ui/src/status_bar.rs` (전체 read-only — `set_git_branch` / `clear_git_branch` API 호출만)
- `crates/moai-studio-ui/src/palette/registry.rs` (본 SPEC 무수정)
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 8. Test Strategy

ui crate `lib.rs::tests` 모듈에 fixture-based integration tests 5개 추가 (T-SBW2 블록). 기존 placeholder 검증 2 tests (`derive_status_git_label_returns_workspace_id` / `derive_status_git_label_empty_id_returns_none`) 는 본 SPEC 에서 의미가 사라지므로 제거 — net +3 tests.

테스트 fixture 패턴 (각 test 별 독립적 tempfile 사용):

- AC-SBW-7 (clean repo): `tempfile::TempDir::new()` → `git2::Repository::init(path)` → empty commit (signature, tree, parent[]) → `derive_status_git_label_from_workspace(path)` 호출 → `Some((branch, false))` 검증. branch 는 `"main"` 또는 `"master"` 둘 중 하나 (libgit2 default 가 환경에 따라 다름) — 테스트는 둘 다 허용하는 OR 매처 사용
- AC-SBW-8 (dirty repo): AC-SBW-7 의 fixture + tempfile 안에 신규 파일 write (commit 안 함) → `derive_status_git_label_from_workspace(path)` 호출 → `(branch, true)` 검증
- AC-SBW-9 (non-git dir): `tempfile::TempDir::new()` 만 (git init 없음) → 호출 → `None` 검증
- AC-SBW-10 (detached HEAD): AC-SBW-7 의 fixture + commit OID 추출 후 `repo.set_head_detached(oid)` → 호출 → `Some(("detached".to_string(), _))` 검증
- AC-SBW-11 (missing path): `Path::new("/tmp/definitely-not-existing-{uuid::Uuid::new_v4()}")` → 호출 → `None` 검증

`refresh_status_git_label` 의 cx-bound 래핑 부분 (active workspace lookup → path 추출 → derive 호출) 은 #109 의 정책 동일 (cx-free helper 단위 검증으로 분기 정확성 100% 확보, GPUI cx-bound wrapper 는 GPUI-level 검증 생략).

회귀 검증: 기존 ui crate 1374 tests 중 placeholder-bound 2 tests 만 의도적 제거, 나머지 1372 tests 무영향. 신규 +5 tests 후 합계 1377 tests.

본 SPEC run 단계에서 `cargo test -p moai-studio-ui --lib` + `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` + `cargo fmt --all -- --check` 3 gate 통과 필수.

추가 dev-dependency: `tempfile` (이미 다수 crate 가 사용 중이므로 workspace dep 가능), `git2` 직접 사용은 `moai-git` 통과로 wrapping 되지만 fixture 생성 (init / detached HEAD 진입) 에는 직접 호출 필요 → ui crate 의 dev-dependency 에 `git2 = "0.20"` 추가 검토 (manager-tdd 가 run 진입 시 결정 — workspace dep / 직접 dep 정합성 확인 후 적용).

## 9. Risks / Open Questions

- **OQ-1 (default branch name)**: `git init` 의 default branch 가 환경에 따라 `master` 또는 `main` (git 2.28+ `init.defaultBranch` 설정 의존). 테스트는 `branch == "main" || branch == "master"` OR 매처로 두 값 모두 허용 — 호스트 git 설정에 무관하게 통과해야 함. (manager-tdd 가 run 진입 시 정확한 git2 default 동작 재확인 후 매처 형태 결정)
- **OQ-2 (workspace dep vs path dep)**: `moai-git` 을 ui crate 의 dependency 에 추가할 때 workspace `Cargo.toml` 의 `[workspace.dependencies]` 에 `moai-git` 등록 여부 확인 필요. 미등록 상태면 path dep (`moai-git = { path = "../moai-git" }`) 사용 후 workspace dep 등록은 별 SPEC carry-to. (manager-tdd 가 결정)
- **OQ-3 (active workspace lookup 비용)**: `refresh_status_git_label` 가 매 workspace 전환마다 `WorkspacesStore::load(&self.storage_path)` 를 다시 호출하면 file IO 비용이 발생 (#109 의 `handle_activate_workspace` 패턴 동일). 단일 워크스페이스 전환 이벤트 1회당 1회 호출이라 typical 비용은 무시 가능 — in-memory cache 진화는 별 SPEC.
- **R-1 (libgit2 cross-platform 동작)**: macOS / Linux 에서 `git2::Repository::open` 의 동작 차이 (특히 worktree, symlink) 가능성. 본 SPEC 은 main project_path 만 대상 — worktree 진입은 사용자가 worktree 의 `.git` file 을 가리키는 path 를 workspace 로 등록한 경우만 발생, 그 케이스는 `Repository::open` 이 자동 resolution 수행. 추가 처리 없음.
- **R-2 (`is_dirty` 비용)**: `is_dirty` 는 인덱스 + 워킹 트리 status walk 를 수행하여 큰 저장소 (수만 파일) 에서 수십 ms 소요 가능. 동기 호출 정책 (REQ-SBW-014) 하에 일반 저장소는 무시 가능, 거대 저장소 사용자만 perceptible. 프로파일링 결과로만 별 SPEC 진입.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-06
Cycle: v0.3.0 Sprint 2 #8 (carry-from #7 STATUS-BAR-WIRE-001 §"Carry-to (c) 실제 git2 crate 통합")
Carry-from: SPEC-V0-3-0-STATUS-BAR-WIRE-001 (#109 — placeholder skeleton + handle_activate_workspace wire), `moai-git` crate (git2 wrapper API 완비)
Carry-to: (a) palette/registry.rs 에 `status.refresh_git` 신규 entry (#109 carry-to (a) 미해결), (b) `set_agent_mode` payload 메커니즘 (#109 carry-to (b) 미해결), (c) detached HEAD 시 short SHA 라벨 진화 (현재 고정 `"detached"`), (d) async / cached refresh — 프로파일링 trigger 시, (e) file watcher 통합으로 commit / status 변경 시 자동 refresh, (f) LSP polling → `set_lsp_status` wire (#109 carry-to (d) 미해결)
