# SPEC-V0-1-2-MENUS-001 Progress

**Started**: 2026-04-27
**Branch**: feature/SPEC-V0-1-2-MENUS-001 (squash merged into main)
**SPEC status**: implemented
**Completion date**: 2026-04-27

## Implementation Timeline

- 2026-04-27 v0.1.1 hotfix: macOS menu bar 6 menus (App/File/Edit/View/Window/Help) + items wire — `7a6004f` (PR #53)
- 2026-04-27 View/Pane/Surface/Go menus + keybindings — PR #58 (`24fd7fe`)

## Context

The v0.1.1 hotfix (PR #53, commit `7a6004f`) established the initial macOS menu bar with 6 menus and basic wiring. However, View/Pane/Surface menus were still empty placeholders. SPEC-V0-1-2-MENUS-001 was created to fill these gaps.

## Milestone Status

- [x] v0.1.1 hotfix: Basic menu bar with 6 menus (App/File/Edit/View/Window/Help) — PR #53
- [x] View menu: Toggle Sidebar (Cmd+B), Toggle Banner, Find (Cmd+F), Reload (Cmd+R), Toggle Theme (Cmd+T)
- [x] Pane menu: Split Right (Cmd+\), Split Down (Cmd+Shift+\), Close (Cmd+W), Focus Next/Prev (Cmd+]/[)
- [x] Surface menu: New Terminal, New Markdown, New Code Viewer
- [x] Go menu: Command Palette (Cmd+K), SPEC Panel (Cmd+Shift+P)
- [x] Help menu: Documentation, Report Issue, About expanded

## Key Files Changed

- `crates/moai-studio-ui/src/lib.rs`: 93 insertions — all 4 new menus (View/Pane/Surface/Go) + Help expansion + keybindings

## Test Coverage

- cargo build/clippy/fmt PASS
- No dedicated unit tests for menu items (GPUI menu rendering is visual)

## Known Limitations

- Action handlers are stubs only — menu items call empty handler functions
- Actual behavior wiring (pane splitting, surface creation, theme toggle) is carry-over to future SPECs
- Toggle Theme (Cmd+T) conflicts with some terminal emulators' "new tab" shortcut
- Menu state synchronization (enable/disable based on app state) not implemented
