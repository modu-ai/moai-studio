---
spec_id: SPEC-V0-3-0-STATUS-BAR-WIRE-002
status: green
methodology: tdd
created_at: 2026-05-06
updated_at: 2026-05-06
---

# SPEC-V0-3-0-STATUS-BAR-WIRE-002 Progress

## Phase Log

### RED (2026-05-06)

5 fixture-based tests added to `crates/moai-studio-ui/src/lib.rs::tests` (T-SBW2 block):
- `derive_status_git_label_clean_repo_returns_branch_no_dirty` (AC-SBW-7)
- `derive_status_git_label_dirty_repo_returns_dirty_true` (AC-SBW-8)
- `derive_status_git_label_non_git_directory_returns_none` (AC-SBW-9)
- `derive_status_git_label_detached_head_returns_fixed_label` (AC-SBW-10)
- `derive_status_git_label_missing_path_returns_none` (AC-SBW-11)

Compilation failed (E0308 signature mismatch) before function body update — confirmed RED.

### GREEN (2026-05-06)

Changes applied:

1. `crates/moai-studio-ui/Cargo.toml`: added `git2 = "0.20"` to `[dev-dependencies]` for fixture creation (`Repository::init`, `set_head_detached`). Production code uses the existing `moai-git` wrapper at `[dependencies]` line 69.
2. `crates/moai-studio-ui/src/lib.rs`:
   - Added `use moai_git::{GitError, GitRepo};` after the existing `moai_studio_workspace` import.
   - Replaced `derive_status_git_label_from_workspace` body (placeholder echo → `GitRepo::open` + `current_branch` + `is_dirty`). Signature changed from `&str` to `&std::path::Path` per REQ-SBW-008.
   - Replaced `refresh_status_git_label` to look up the active workspace by id against the in-memory `self.workspaces` Vec and pass `&ws.project_path` to the derive helper. Lookup miss → `clear_git_branch` + `cx.notify()`.
   - Removed the two placeholder unit tests; added the five fixture-based tests above plus a `make_committed_repo` helper.

First test run: 1376 passed / 1 failed (`derive_status_git_label_detached_head_returns_fixed_label`).

### Spec Deviation (documented)

Failure cause: `moai_git::GitRepo::current_branch()` returns `Ok("HEAD")` for a detached HEAD (libgit2's `Reference::shorthand` of the HEAD ref itself is the literal `"HEAD"`), not `Err(GitError::DetachedHead)` as REQ-SBW-010 had assumed. The spec text predicted the wrapper would surface a typed error; in practice the wrapper passes the libgit2 default through.

Resolution: Extended the `match repo.current_branch()` arm to also map `Ok(name) if name == "HEAD"` to the canonical `"detached"` label. This preserves the spec's user-visible contract (status bar shows `"detached"` when HEAD is detached) without modifying the `moai-git` crate (FROZEN per spec.md §3). The `Err(GitError::DetachedHead)` arm is retained for forward compatibility in case the wrapper is later changed to return the typed error.

Inline rationale comment added to the match arm (English, per CLAUDE.local.md §9). No changes to spec.md required — the public AC remains satisfied.

### Verification (2026-05-06)

```
cargo build -p moai-studio-ui --tests        → OK (55.60s)
cargo test  -p moai-studio-ui --lib          → 1377 passed; 0 failed; 0 ignored (0.61s)
cargo clippy -p moai-studio-ui --all-targets -- -D warnings → 0 warnings
cargo fmt --all -- --check                   → clean (1 trivial drift auto-applied)
```

Test count delta: 1374 (prior baseline) − 2 (placeholder tests removed) + 5 (fixture-based tests added) = 1377. Matches spec.md §8 prediction exactly.

## Acceptance Criteria

| AC | Status | Test |
|----|--------|------|
| AC-SBW-7 (clean repo) | PASS | `derive_status_git_label_clean_repo_returns_branch_no_dirty` |
| AC-SBW-8 (dirty repo) | PASS | `derive_status_git_label_dirty_repo_returns_dirty_true` |
| AC-SBW-9 (non-git dir) | PASS | `derive_status_git_label_non_git_directory_returns_none` |
| AC-SBW-10 (detached HEAD) | PASS | `derive_status_git_label_detached_head_returns_fixed_label` |
| AC-SBW-11 (missing path) | PASS | `derive_status_git_label_missing_path_returns_none` |
| AC-SBW-12 (cargo gate) | PASS | clippy 0 warn / fmt clean / test 1377 PASS |

6/6 AC met.

## OQ Resolution

- **OQ-1 (default branch matcher)**: tests use `branch == "main" || branch == "master"` OR matcher (verified working on macOS host with `init.defaultBranch=main`).
- **OQ-2 (workspace dep vs path dep)**: not applicable — `moai-git` was already a `path` dependency at `crates/moai-studio-ui/Cargo.toml:69`. No new entry needed in `[workspace.dependencies]`. Promotion to workspace dep is a separate refactor SPEC.
- **OQ-3 (active workspace lookup cost)**: `refresh_status_git_label` now uses `self.workspaces.iter().find(...)` (in-memory Vec, O(n)) instead of re-loading `WorkspacesStore`. Lower cost than originally feared in spec.md §9.

## Files Touched

- `crates/moai-studio-ui/Cargo.toml` (+5 lines: git2 dev-dep)
- `crates/moai-studio-ui/src/lib.rs` (~+115 lines net: import, body replacement, helper rewrite, tests block)
- `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-002/spec.md` (already created in plan phase)
- `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-002/progress.md` (this file)

FROZEN files honoured: `crates/moai-git/**`, `crates/moai-studio-workspace/**`, `crates/moai-studio-terminal/**`, `crates/moai-studio-ui/src/status_bar.rs`, `crates/moai-studio-ui/src/palette/registry.rs` — all untouched.
