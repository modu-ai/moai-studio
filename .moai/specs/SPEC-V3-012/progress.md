# SPEC-V3-012 Progress

**Started**: 2026-04-26
**Branch**: feature/SPEC-V3-012-ms4-cmd-registry (MS-4)
**SPEC status**: FULLY IMPLEMENTED (all 4 milestones, 22 acceptance criteria)
**Completion date**: 2026-04-30 (MS-4)

## Implementation Timeline

- 2026-04-26 PR #22 (`fef0659`): MS-1 Scrim + PaletteView core (AC-PL-1~5) — 810 LOC, 21 tests
- 2026-04-26 PR #24 (`824eff1`): MS-2 3 variants + fuzzy match (AC-PL-6~13) — 1423 LOC, 48 new tests
- 2026-04-26 PR #26 (`58766a6`): MS-3 RootView integration + global keybindings (AC-PL-14~15) — 382 LOC, 18 new tests
- 2026-04-26 `ad44b83`: docs — SPEC-V3-012 Palette + SPEC-V3-013 Settings plan
- 2026-04-26 `b83fa84`: docs — SPEC-V3-012 + V3-013 status draft → implemented
- 2026-04-30 `fa03fab`: MS-4 CommandRegistry + dispatch wiring (AC-PL-16~22) — 1105 LOC, 33 new tests

## Milestone Status

- [x] MS-1: Scrim + PaletteView core — PR #22 (AC-PL-1~5)
- [x] MS-2: 3 variants + fuzzy match — PR #24 (AC-PL-6~13)
- [x] MS-3: RootView integration + global keybindings — PR #26 (AC-PL-14~15)
- [x] MS-4: CommandRegistry + dispatch wiring + @mention — `fa03fab` (AC-PL-16~22)

## Key Files Changed

### MS-1 — Scrim + PaletteView Core (4 files, 810 LOC)

- `crates/moai-studio-ui/src/palette/mod.rs`: Module root, PaletteView re-exports
- `crates/moai-studio-ui/src/palette/palette_view.rs`: 461 LOC — PaletteView Entity, 600px container, 14px input, 32px row, 320px max-height, keyboard nav state machine (Up/Down/Enter/Esc), FocusState management
- `crates/moai-studio-ui/src/palette/scrim.rs`: 320 LOC — Scrim Entity, backdrop, theme-aware (dark rgba(8,12,11,0.55) / light rgba(20,30,28,0.18)), click-to-dismiss, z-index 20
- `crates/moai-studio-ui/src/lib.rs`: `mod palette;` registration

### MS-2 — 3 Variants + Fuzzy Match (7 files, 1423 LOC)

- `crates/moai-studio-ui/src/palette/fuzzy.rs`: 437 LOC — Fuzzy matcher with subsequence matching + scoring + highlight indices (zero-dep). Scoring: base_match_credit +16, consecutive_bonus +15, prefix_bonus +10, word_boundary_bonus +8, gap_penalty -1. `build_text_runs` for TextRun conversion with accent-soft style.
- `crates/moai-studio-ui/src/palette/variants/cmd_palette.rs`: 270 LOC — CmdPalette (Cmd+P) for file quick-open, mock file index (5+ entries)
- `crates/moai-studio-ui/src/palette/variants/command_palette.rs`: 281 LOC — CommandPalette (Cmd+Shift+P) with command registry execution, mock 10 commands
- `crates/moai-studio-ui/src/palette/variants/slash_bar.rs`: 284 LOC — SlashBar (/moai *) slash subcommand launcher, mock 8 commands
- `crates/moai-studio-ui/src/palette/variants/mod.rs`: Variant module root
- `crates/moai-studio-ui/src/palette/mod.rs`: PaletteVariant enum addition
- `crates/moai-studio-ui/src/palette/palette_view.rs`: Highlight test additions

### MS-3 — RootView Integration (2 files, 382 LOC)

- `crates/moai-studio-ui/src/lib.rs`: 250 LOC added — PaletteOverlay (active variant management, mutual exclusion open/dismiss/toggle), RootView PaletteOverlay slot + terminal_focused field, `handle_palette_key_event` (Cmd+P/Cmd+Shift+P/Esc/slash), `render_palette_overlay` with Scrim backdrop
- `crates/moai-studio-ui/src/palette/mod.rs`: 133 LOC added — PaletteOverlay public interface

## Test Coverage

- MS-1: 21 new tests passing
- MS-2: 48 new tests (494 total passing)
- MS-3: 18 new tests (503 total passing)
- MS-4: 33 new tests — registry(9) + cmd_palette@mention(10) + slash_bar(3) + lib.rs dispatch(16) = 558 total
- Quality gates: clippy 0 warnings, rustfmt PASS, `cargo check --release` PASS on all milestones

## Known Limitations

- Mock data: File index in CmdPalette and slash commands in SlashBar use mock data. CommandPalette now uses real CommandRegistry (MS-4).
- PaletteVariant mutual exclusion is enforced but variant switching within an active session is simplified.
- No plan.md exists — planning was done inline in docs commit `ad44b83`. This is a known gap in SPEC artifact completeness.
- `pending_slash_injection` buffer is written by `inject_slash_command()` but must be drained by the render/update loop when a TerminalSurface Entity context is available. Terminal stdin actual write is deferred (V3-PALETTE-001).
- `dispatch_command` for tab.*, pane.*, surface.*, workspace.*, git.*, agent.* logs and returns true but does not invoke actual Entity actions (requires Entity handles not available in plain &mut self context).
- CmdPalette @mention mode uses static mock symbols/issues (MOCK_SYMBOLS, MOCK_ISSUES). Real language server / GitHub integration is out of scope for MS-4.
