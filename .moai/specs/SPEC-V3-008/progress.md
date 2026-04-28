# SPEC-V3-008 Progress

**Started**: 2026-04-25
**Branch**: feature/SPEC-V3-005-file-explorer (shared branch with V3-006/DIST-001)
**SPEC status**: ms2-ms3-committed
**Completion date**: N/A (commit_composer.rs, status_panel.rs still pending)

## Planning Phase

- 2026-04-25 `8e1d3e9` PR #9: SPEC backlog created — included V3-008 scope alongside V3-004/V3-005~011

## Implementation Timeline

- 2026-04-27 `82a11b2` PR #60: feat(viewer,git): SPEC-V3-008 MS-1 — moai-git crate expansion (6 new modules)
- 2026-04-28 `02d8ac9`: feat(git): SPEC-V3-008 MS-2/MS-3 — Git UI 5개 GPUI Entity + 통합 테스트 (committed)

## Milestone Status

- [x] MS-1: moai-git crate API expansion (status, diff, commit, branch, log, stash modules) — PR #60
- [x] MS-2: Diff Viewer + Branch Switcher UI — committed `02d8ac9`
- [x] MS-3: Log Graph + Merge Conflict Resolver + Stash Management UI — committed `02d8ac9`

## Key Files Changed

### New Files (moai-git crate expansion)

- `crates/moai-git/src/branch.rs`: branches(), create_branch(), checkout() — local branch listing, creation, switching
- `crates/moai-git/src/commit.rs`: stage(), unstage(), commit(), log() — staging area + commit + log history
- `crates/moai-git/src/diff.rs`: diff_file(), diff_workdir(), parse_diff() — file diff and working directory diff
- `crates/moai-git/src/log.rs`: diff_commit(), show_commit() — per-commit diff inspection
- `crates/moai-git/src/stash.rs`: stash_push() — partial stash implementation (MS-3 scope)
- `crates/moai-git/src/lib.rs`: Expanded with new module re-exports + @MX:ANCHOR on GitRepo::open(), status_map()

### Modified Files

- `crates/moai-git/src/worktree.rs`: Unchanged (SPEC-V3-001 carry)

### Not Yet Created (MS-2/MS-3 scope)

- `crates/moai-studio-ui/src/git/status_panel.rs`
- `crates/moai-studio-ui/src/git/commit_composer.rs`

### Uncommitted Work (MS-2/MS-3 — written but not committed)

The following files exist on disk but are not tracked by git. They represent MS-2/MS-3 UI implementation that was written in a prior session but not committed or compiled.

- `crates/moai-studio-ui/src/git/mod.rs` (20 LOC) — Module registration + re-exports
- `crates/moai-studio-ui/src/git/branch_switcher.rs` (202 LOC) — GitBranchSwitcher entity (REQ-G-030~035)
- `crates/moai-studio-ui/src/git/diff_viewer.rs` (176 LOC) — GitDiffViewer entity (REQ-G-010~015)
- `crates/moai-studio-ui/src/git/log_view.rs` (237 LOC) — GitLogView entity (REQ-G-040~044)
- `crates/moai-studio-ui/src/git/merge_resolver.rs` (253 LOC) — GitMergeResolver entity (REQ-G-050~056)
- `crates/moai-studio-ui/src/git/stash_panel.rs` (176 LOC) — GitStashPanel entity (REQ-G-060~064)
- `crates/moai-studio-ui/tests/integration_git.rs` (532 LOC) — Integration tests
- `crates/moai-studio-ui/src/lib.rs` (+251 insertions) — Git module wiring + RootView integration
- Total: ~1064 LOC UI + 532 LOC tests = 1596 LOC uncommitted

**Action needed**: Review uncommitted files, run `cargo check`, resolve compilation issues, then commit.

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-A-1 | PARTIAL | moai-git status_map exists but UI GitStatusPanel not created (commit_composer.rs not written) |
| AC-A-2 | PARTIAL | stage/unstage API exists but UI toggle not created |
| AC-A-3 | PARTIAL | commit API exists but UI composer not created |
| AC-A-4 | NOT STARTED | Commit button disabled state (UI scope) |
| AC-A-5 | NOT STARTED | Non-git directory handling (UI scope) |
| AC-A-6 | UNCOMMITTED | Diff Viewer UI written (diff_viewer.rs, 176 LOC) but not compiled/committed |
| AC-A-7 | NOT STARTED | Syntax highlight fallback |
| AC-A-8 | UNCOMMITTED | Branch switcher UI written (branch_switcher.rs, 202 LOC) but not compiled/committed |
| AC-A-9 | UNCOMMITTED | New branch creation UI — part of branch_switcher.rs |
| AC-A-10 | UNCOMMITTED | Log graph UI written (log_view.rs, 237 LOC) but not compiled/committed |
| AC-A-11 | UNCOMMITTED | Commit row click → diff — part of log_view.rs |
| AC-A-12 | UNCOMMITTED | Merge conflict resolver UI written (merge_resolver.rs, 253 LOC) but not compiled/committed |
| AC-A-13 | UNCOMMITTED | Stash push/pop UI written (stash_panel.rs, 176 LOC) but not compiled/committed |

## Test Coverage

- moai-git crate: inline unit tests in branch.rs, commit.rs, diff.rs, log.rs, stash.rs
- Integration tests in `crates/moai-git/tests/`
- @MX tags: ANCHOR on high fan_in functions (GitRepo::open, status_map)

## Known Limitations

- MS-1 is API-only (committed in PR #60)
- MS-2/MS-3 UI code exists on disk but is **uncommitted** (6 files, ~1064 LOC + 532 LOC tests)
- Uncommitted UI files have not been compiled or tested — `cargo check` status unknown
- stash.rs only implements stash_push(); stash_apply(), stash_drop(), stash_list() are stubs
- diff_workdir() is a stub (placeholder implementation)
- stash_apply/drop/list deferred to MS-3
- commit_composer.rs and status_panel.rs still missing (not in uncommitted set)
- git2 = 0.20 maintained (USER-DECISION UD-1: no gix migration)
- No merge conflict resolver (MS-3 scope)
- No log graph visualization (MS-3 scope)

## USER-DECISION Resolutions

- UD-1 (git library): git2 0.20 retained, no gix migration
- UD-2 (diff mode): Not yet decided (MS-2 gate)
- UD-3 (UI integration pattern): Hybrid C planned but not implemented
- UD-4 (AI commit suggest): Not yet decided (MS-1 gate)
- UD-5 (dirty switch behavior): Not yet decided (MS-2 gate)
- UD-6 (graph algorithm): Not yet decided (MS-3 gate)
- UD-7 (stash scope): Not yet decided (MS-3 gate)
