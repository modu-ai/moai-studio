# SPEC-V0-1-2-MENUS-001 — View/Pane/Surface/Go Menus

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-1-2-MENUS-001 |
| **Title** | View / Pane / Surface / Go menus with keybindings |
| **Status** | implemented |
| **Priority** | High |
| **Dependencies** | SPEC-V3-001 (app shell) |
| **Covers** | v0.1.2 menu bar gap — user feedback "View/Pane/Surface menus empty" |

## 1. Problem Statement

After v0.1.0, the macOS menu bar showed 6 menus (App/File/Edit/View/Window/Help) but View, Pane, and Surface menus were empty placeholders. User screenshot feedback confirmed this gap. Additional menus (Go) and expanded Help were needed for feature discoverability.

## 2. Scope

| In Scope | Out of Scope |
|----------|-------------|
| View menu: Toggle Sidebar, Toggle Banner, Find, Reload, Toggle Theme | Full action handler wiring (stubs only) |
| Pane menu: Split Right/Down, Close, Focus Next/Prev | Pane splitting implementation |
| Surface menu: New Terminal, New Markdown, New Code Viewer | Surface creation backend |
| Go menu: Command Palette, SPEC Panel | Palette UI rendering |
| Help menu expansion: Documentation, Report Issue, About | Documentation content |

## 3. Acceptance Criteria

- AC-MN-1: View menu has 5 items with keybindings (Cmd+B, Cmd+F, Cmd+R, Cmd+T)
- AC-MN-2: Pane menu has 4 items with keybindings (Cmd+\, Cmd+Shift+\, Cmd+W, Cmd+]/[)
- AC-MN-3: Surface menu has 3 items (New Terminal/Markdown/Code Viewer)
- AC-MN-4: Go menu has 2 items (Cmd+K, Cmd+Shift+P)
- AC-MN-5: Help menu expanded with Documentation + Report Issue + About
- AC-MN-6: cargo build/clippy/fmt PASS

### 3.1 MS-2 Acceptance Criteria (action handler wiring polish)

MS-2 (2026-04-30, v0.1.2 Task 7) replaces 4 stub `info!("... deferred")` handlers with functional behavior. Remaining stubs (ToggleSidebar, ReloadWorkspace, NewTerminalSurface, NewMarkdownSurface, NewCodeViewerSurface, FocusNextPane, FocusPrevPane, ClosePane) stay deferred to follow-up SPECs. The four wired actions are:

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-MN-7 | RootView with palette dismissed | OpenCommandPalette action dispatched | `palette.active_variant == Some(CmdPalette)` and `cmd_palette.is_some()` | unit test (`open_command_palette_action_toggles_cmd_palette`) |
| AC-MN-8 | RootView with no spec_panel | OpenSpecPanel action dispatched | `spec_panel` is `Some(SpecPanelView)` mounted from `storage_path/.moai/specs`; second dispatch dismisses | unit tests (`open_spec_panel_action_mounts_when_dismissed`, `open_spec_panel_action_dismisses_when_visible`) |
| AC-MN-9 | RootView with TabContainer (single leaf) and focused pane id | SplitRight action dispatched | active_tab.pane_tree becomes `Split { direction: Horizontal, .. }`; focused leaf still resolvable | unit test (`split_right_action_horizontal_splits_focused_leaf`) |
| AC-MN-10 | RootView with TabContainer (single leaf) and focused pane id | SplitDown action dispatched | active_tab.pane_tree becomes `Split { direction: Vertical, .. }` | unit test (`split_down_action_vertical_splits_focused_leaf`) |
| AC-MN-11 | RootView with no TabContainer or no focused pane | Any of the above 4 actions dispatched | No panic, no state mutation outside the no-op branch | unit tests (early-return paths) |

Constraints:
- REQ-MN-MS2-1: SPEC-V3-002 terminal crate untouched (FROZEN).
- REQ-MN-MS2-2: SPEC-V3-003 panes/tabs public API unchanged — only call existing `split_horizontal` / `split_vertical`.
- REQ-MN-MS2-3: Existing app-level handlers (Quit/About/ReportIssue/OpenDocumentation/OpenAbout) remain in place; no duplicate wiring.

## 4. Keybindings

| Menu | Item | Shortcut |
|------|------|----------|
| View | Toggle Sidebar | Cmd+B |
| View | Toggle Banner | (none) |
| View | Find | Cmd+F |
| View | Reload | Cmd+R |
| View | Toggle Theme | Cmd+T |
| Pane | Split Right | Cmd+\ |
| Pane | Split Down | Cmd+Shift+\ |
| Pane | Close | Cmd+W |
| Pane | Focus Next | Cmd+] |
| Pane | Focus Prev | Cmd+[ |
| Surface | New Terminal | (none) |
| Surface | New Markdown | (none) |
| Surface | New Code Viewer | (none) |
| Go | Command Palette | Cmd+K |
| Go | SPEC Panel | Cmd+Shift+P |

---

Version: 1.0.0
Created: 2026-04-27
