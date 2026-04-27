# SPEC-V3-005 Progress

**Started**: 2026-04-25
**Branch**: feature/SPEC-V3-005-file-explorer (squash merged into main)
**SPEC status**: implemented (all 3 milestones complete)
**Completion date**: 2026-04-26

## USER-DECISION Gates (All Resolved)

- **USER-DECISION-A** (moai-fs API shape): Option (a) WorkspaceWatcher helper added at `crates/moai-fs/src/workspace_watcher.rs`
- **USER-DECISION-B** (gpui test-support): Option (a) gpui test-support added; spike 0 build success (9.12s)
- **USER-DECISION-C** (delete trash policy): Option (a) trash crate v5 for OS trash; spike 1 build success (0.97s)

## Implementation Timeline

- 2026-04-25 MS-1: File Explorer foundation — PR #10 (`5764ee5`) — 447 workspace tests, 14 new
- 2026-04-25 MS-2: Watch + debounce + delta apply — PR #14 (`3ed8d35`) — 522 tests, 12 new
- 2026-04-25 E2E wiring: Explorer click → viewer mount — PR #16 (`0293f20`) — 545 tests, 4 new integration
- 2026-04-26 MS-3 final: git status + context menu + DnD + search — PR #34 (`e2a4c13`) — AC-FE-8~12 PASS

## Milestone Status

- [x] Spike 0 + Spike 1 completed
- [x] MS-1: FsNode + ChildState + path::normalize + WorkspaceWatcher + FileExplorer Entity + RootView field — PR #10
- [x] MS-2: WorkspaceWatcher + debounce + diff apply + rename matching — PR #14
- [x] E2E wiring: Explorer click → viewer mount (shared with SPEC-V3-006) — PR #16
- [x] MS-3: git status + context menu + DnD + search — PR #34

## Key Files Changed

- `crates/moai-fs/src/lib.rs`: WorkspaceWatcher public API
- `crates/moai-fs/src/workspace_watcher.rs`: File system watcher helper (shared with SPEC-V3-008)
- `crates/moai-studio-ui/src/explorer/mod.rs`: Module entry point
- `crates/moai-studio-ui/src/explorer/path.rs`: path::normalize utility
- `crates/moai-studio-ui/src/explorer/tree.rs`: FsNode + ChildState + lazy load tree model
- `crates/moai-studio-ui/src/explorer/view.rs`: FileExplorer GPUI Entity + RootView field + click handler
- `crates/moai-studio-ui/src/explorer/config.rs`: Explorer configuration (FS watcher settings)
- `crates/moai-studio-ui/src/explorer/watch.rs`: Watch + debounce + delta apply engine
- `crates/moai-studio-ui/src/explorer/git_status.rs`: Git status overlay for file nodes
- `crates/moai-studio-ui/src/explorer/context_menu.rs`: Right-click context menu (AC-FE-8)
- `crates/moai-studio-ui/src/explorer/dnd.rs`: Drag and drop file operations (AC-FE-10)
- `crates/moai-studio-ui/src/explorer/search.rs`: File search/filter functionality (AC-FE-11)
- `crates/moai-studio-ui/src/lib.rs`: Explorer module registration + RootView integration
- `crates/moai-studio-ui/tests/integration_explorer_viewer.rs`: E2E integration tests (explorer click → viewer)
- `.moai/config/sections/fs.yaml`: File system watcher configuration

## Test Coverage

- 447 tests at MS-1 completion (14 new)
- 522 tests at MS-2 completion (12 new)
- 545 tests at E2E wiring completion (4 new integration)
- MS-3 added AC-FE-8~12 coverage (git status, context menu, DnD, search)
- Integration test: explorer click → viewer mount (end-to-end with SPEC-V3-006)

## Known Limitations

- DnD is file-system level only (no cross-pane DnD yet)
- Search is file-name based (no content search)
- Context menu actions partially wired (some handlers are stubs)
- Git status uses polling (not git watch hook)
