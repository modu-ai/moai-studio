# SPEC-V0-3-0-STATUS-BAR-WIRE-001 — Progress

| Field | Value |
|-------|-------|
| **plan_complete_at** | 2026-05-05 |
| **plan_status** | audit-ready |
| **harness_level** | minimal (lightweight SPEC, ≤6 ACs, MS-1 단일) |
| **methodology** | TDD (RED-GREEN-REFACTOR via manager-cycle) |
| **base_commit** | 4a95529 (main, post-#106 WORKSPACE-DOT-COLOR-001) |
| **worktree** | feature/SPEC-V0-3-0-STATUS-BAR-WIRE-001 |
| **branch** | feature/SPEC-V0-3-0-STATUS-BAR-WIRE-001 |
| **run_complete_at** | 2026-05-05 |
| **run_status** | complete |

## Milestones

- [x] MS-1: 3 cx-bound RootView helper + 2 cx-free helper (`route_status_command_to_kind` / `derive_status_git_label_from_workspace`) + `StatusCommand` enum + `dispatch_command` 의 `status.*` 분기 + `handle_activate_workspace` git label refresh hook + 5 unit tests (T-SBW 블록). cargo test/clippy/fmt 3-gate PASS.

## Tasks (run 단계 진입 후 RED → GREEN → REFACTOR)

| Task ID | Phase | Description | Status |
|---------|-------|-------------|--------|
| T-SBW-1 | RED | Add 5 failing unit tests in lib.rs::tests covering AC-SBW-1~5 (`route_status_command_to_kind_set_agent_mode`, `route_status_command_to_kind_clear_and_refresh`, `route_status_command_to_kind_unknown_returns_none`, `derive_status_git_label_returns_workspace_id`, `derive_status_git_label_empty_id_returns_none`). Confirm compile fails (helpers/enum undefined). | done |
| T-SBW-2 | GREEN | Add `StatusCommand` enum (SetAgentMode / ClearAgentMode / RefreshGit) + cx-free `route_status_command_to_kind(&str) -> Option<StatusCommand>` covering 3 branches. | done |
| T-SBW-3 | GREEN | Add cx-free `derive_status_git_label_from_workspace(workspace_id: &str) -> Option<(String, bool)>` (empty → None, non-empty → Some((id, false))). | done |
| T-SBW-4 | GREEN | Add 3 cx-bound `RootView` helpers: `set_status_agent_mode`, `clear_status_agent_mode`, `refresh_status_git_label`. Each calls the corresponding `StatusBarState` setter and triggers `cx.notify()`. | done |
| T-SBW-5 | GREEN | Wire `dispatch_command` with `status.` prefix branch using `route_status_command_to_kind`. SetAgentMode → call helper with placeholder `"Plan"`; ClearAgentMode/RefreshGit → call respective helpers. Unknown `status.*` → return `false`. | done |
| T-SBW-6 | GREEN | Hook `refresh_status_git_label` invocation into `handle_activate_workspace` (after `store.touch` succeeds). Confirm 5 unit tests pass; ui crate test count = baseline + 5. | done |
| T-SBW-7 | REFACTOR | `cargo fmt --all`, `cargo clippy -p moai-studio-ui --all-targets -- -D warnings`, `cargo test -p moai-studio-ui --lib`. All 3 gates GREEN. LSP errors = 0. | done |

## Iteration Log

### Iteration 1 (2026-05-05)

**RED**: Added 5 unit tests in lib.rs::tests (T-SBW block). Compile failed with 11 errors (undefined `route_status_command_to_kind`, `StatusCommand`, `derive_status_git_label_from_workspace`). RED state confirmed.

**GREEN**: Added `StatusCommand` enum + `route_status_command_to_kind` + `derive_status_git_label_from_workspace` (cx-free helpers). Added 3 cx-bound RootView helpers (`set_status_agent_mode`, `clear_status_agent_mode`, `refresh_status_git_label`). Wired `dispatch_command` `status.*` branch (4 arms: SetAgentMode/ClearAgentMode/RefreshGit/None→false). Hooked `refresh_status_git_label` into `handle_activate_workspace` after store.touch. `cargo test -p moai-studio-ui --lib` → 1374 passed (baseline 1369 + 5 new).

**REFACTOR**: `cargo fmt --all` applied. `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` → clean. `cargo fmt --all -- --check` → clean. `cargo test -p moai-studio-ui --lib` → 1374 passed.

### AC Completion Summary

| AC | Status |
|----|--------|
| AC-SBW-1 | done (T-SBW-1, T-SBW-2) |
| AC-SBW-2 | done (T-SBW-1, T-SBW-2) |
| AC-SBW-3 | done (T-SBW-1, T-SBW-2) |
| AC-SBW-4 | done (T-SBW-1, T-SBW-3) |
| AC-SBW-5 | done (T-SBW-1, T-SBW-3) |
| AC-SBW-6 | done (T-SBW-7 — 3-gate: 1374 PASS, clippy clean, fmt clean) |
