# SPEC-V3-PALETTE-001 — Fuzzy Search Wiring to File Source

| Field | Value |
|-------|-------|
| **ID** | SPEC-V3-PALETTE-001 |
| **Title** | F-1 Fuzzy search wired to file source (Cmd+K) |
| **Status** | implemented |
| **Priority** | High |
| **Dependencies** | SPEC-V3-012 (Palette Surface core), SPEC-V3-005 (File Explorer) |
| **Covers** | F-1 fuzzy search wiring — connecting CmdPalette to real workspace file scanning |

## 1. Problem Statement

SPEC-V3-012 implemented the Palette Surface (CmdPalette, CommandPalette, SlashBar) with fuzzy match and Scrim UI. However, the CmdPalette operated on mock data — it did not scan actual workspace files. This SPEC wires the CmdPalette to real file sources using recursive directory scanning with fuzzy filtering.

## 2. Scope

| In Scope | Out of Scope |
|----------|-------------|
| CmdPalette::from_workspace_dir() recursive file scan | CmdPalette UI rendering (GPUI Entity render) |
| Fuzzy filter via CmdPalette.set_query() | Slash command injection |
| Cmd+K / Cmd+P toggle + dismiss | Palette overlay visual polish |
| Up/Down navigation, Enter selection, Esc dismiss | Multi-source palette (commands + files) |
| Query reset on dismiss | Workspace file cache optimization |

## 3. Acceptance Criteria

- AC-PA-1: Cmd+K opens CmdPalette with workspace files
- AC-PA-2: Query input triggers fuzzy filter
- AC-PA-3: Up/Down navigation through filtered results
- AC-PA-4: Enter returns selected file path + dismisses palette
- AC-PA-5: Esc dismisses palette and resets query
- AC-PA-6: Missing workspace dir falls back to mock data
- AC-PA-7: Recursive scan respects max 2000 files, depth 8, hidden/artefact exclusion

---

Version: 1.0.0
Created: 2026-04-27
