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
