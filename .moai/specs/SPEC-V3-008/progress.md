# SPEC-V3-008 Progress

**Started**: 2026-04-25
**Branch**: feature/SPEC-V3-005-file-explorer (shared branch with V3-006/DIST-001)
**SPEC status**: ms1-implemented
**Completion date**: N/A (MS-2/MS-3 not yet started)

## Planning Phase

- 2026-04-25 `8e1d3e9` PR #9: SPEC backlog created — included V3-008 scope alongside V3-004/V3-005~011

## Implementation Timeline

- 2026-04-27 `82a11b2` PR #60: feat(viewer,git): SPEC-V3-008 MS-1 — moai-git crate expansion (6 new modules)

## Milestone Status

- [x] MS-1: moai-git crate API expansion (status, diff, commit, branch, log, stash modules) — PR #60
- [ ] MS-2: Diff Viewer + Branch Switcher UI (not started)
- [ ] MS-3: Log Graph + Merge Conflict Resolver + Stash Management UI (not started)

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

- `crates/moai-studio-ui/src/git/mod.rs`
- `crates/moai-studio-ui/src/git/status_panel.rs`
- `crates/moai-studio-ui/src/git/diff_viewer.rs`
- `crates/moai-studio-ui/src/git/commit_composer.rs`
- `crates/moai-studio-ui/src/git/branch_switcher.rs`
- `crates/moai-studio-ui/src/git/log_view.rs`
- `crates/moai-studio-ui/src/git/merge_resolver.rs`
- `crates/moai-studio-ui/src/git/stash_panel.rs`

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-A-1 | PARTIAL | moai-git status_map exists but UI GitStatusPanel not created |
| AC-A-2 | PARTIAL | stage/unstage API exists but UI toggle not created |
| AC-A-3 | PARTIAL | commit API exists but UI composer not created |
| AC-A-4 | NOT STARTED | Commit button disabled state (UI scope) |
| AC-A-5 | NOT STARTED | Non-git directory handling (UI scope) |
| AC-A-6 | NOT STARTED | Diff Viewer UI |
| AC-A-7 | NOT STARTED | Syntax highlight fallback |
| AC-A-8 | NOT STARTED | Branch switcher UI |
| AC-A-9 | NOT STARTED | New branch creation UI |
| AC-A-10 | NOT STARTED | Log graph UI |
| AC-A-11 | NOT STARTED | Commit row click → diff |
| AC-A-12 | NOT STARTED | Merge conflict resolver UI |
| AC-A-13 | NOT STARTED | Stash push/pop UI |

## Test Coverage

- moai-git crate: inline unit tests in branch.rs, commit.rs, diff.rs, log.rs, stash.rs
- Integration tests in `crates/moai-git/tests/`
- @MX tags: ANCHOR on high fan_in functions (GitRepo::open, status_map)

## Known Limitations

- MS-1 is API-only (no GPUI UI components created yet)
- stash.rs only implements stash_push(); stash_apply(), stash_drop(), stash_list() are stubs
- diff_workdir() is a stub (placeholder implementation)
- stash_apply/drop/list deferred to MS-3
- No UI files created for any git management widget
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
