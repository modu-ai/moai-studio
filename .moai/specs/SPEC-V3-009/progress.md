# SPEC-V3-009 Progress

**Started**: 2026-04-26
**Branch**: feature/SPEC-V3-005-file-explorer (shared branch)
**SPEC status**: implemented
**Completion date**: 2026-04-26

## Implementation Timeline

- 2026-04-25 `8e1d3e9` PR #9: SPEC-V3-009 spec/plan/research documents created (planning phase)
- 2026-04-26 `d6ff25c` PR #30: feat(spec-ui): SPEC-V3-009 MS-1 Parser + AC tracker + List/Detail view (96 tests, AC-SU-1~5)
- 2026-04-26 `3be4ebd` docs(spec): SPEC-V3-009 status draft → ms1-implemented
- 2026-04-26 `4f6a598` PR #31: feat(spec-ui): SPEC-V3-009 MS-2 Kanban Board + sidecar persist (24 new tests, AC-SU-6/7)
- 2026-04-26 `f2ba875` PR #32: feat(spec-ui): SPEC-V3-009 MS-3 CLI integration + branch parser + Sprint Contract panel (30 new tests, AC-SU-8~12)
- 2026-04-26 `76af27a` docs(spec): SPEC-V3-009 status ms1-implemented → implemented (all MS complete)

## Milestone Status

- [x] MS-1: Parser + AC tracker + List/Detail view — PR #30 (96 tests)
- [x] MS-2: Kanban Board + sidecar persist — PR #31 (24 new tests)
- [x] MS-3: CLI integration + branch parser + Sprint Contract panel — PR #32 (30 new tests)
- [x] MS-4a: Terminal `TerminalClickEvent::OpenSpec` wires `SpecPanelView::select_spec` (B-4 detection, v0.1.2 Task 8 sub-PR)

### MS-4a Acceptance Criteria

| AC ID | Given | When | Then |
|-------|-------|------|------|
| AC-SU-13 | RootView with `spec_panel = None`, no palette/settings overlay active, `storage_path/.moai/specs/<id>/` exists | TerminalSurface emits `TerminalClickEvent::OpenSpec("SPEC-V3-007")` | spec_panel mounts (`Some(SpecPanelView)`); subsequent `select_spec(SpecId("SPEC-V3-007"))` populates `panel.list.selected_id` and `panel.sprint` |
| AC-SU-14 | RootView with `spec_panel = Some(_)` already mounted | TerminalSurface emits OpenSpec for a different known SPEC | spec_panel stays mounted; `select_spec` updates `selected_id` and rebinds the sprint panel |
| AC-SU-15 | RootView with palette overlay active OR settings modal active | TerminalSurface emits OpenSpec | spec_panel state unchanged (overlay invariant), tracing log emitted, no panic |
| AC-SU-16 | RootView, OpenSpec for an unknown spec_id | Event arrives | spec_panel mounts (or stays); `select_spec` is documented graceful no-op when the id is missing from the index, so `selected_id` keeps its prior value |

## Key Files Changed

### New Crate: moai-studio-spec

- `crates/moai-studio-spec/Cargo.toml`: New crate definition
- `crates/moai-studio-spec/src/lib.rs`: Public API exports
- `crates/moai-studio-spec/src/parser/mod.rs`: SPEC document parser orchestrator
- `crates/moai-studio-spec/src/parser/frontmatter.rs`: YAML frontmatter extraction
- `crates/moai-studio-spec/src/parser/ears.rs`: EARS requirement table parser (RG/REQ extraction)
- `crates/moai-studio-spec/src/parser/ac.rs`: AC table parser with status extraction
- `crates/moai-studio-spec/src/parser/sprint_contract.rs`: Sprint Contract Revision heading extraction
- `crates/moai-studio-spec/src/state/mod.rs`: State module
- `crates/moai-studio-spec/src/state/ac_state.rs`: AcState enum (Full/Partial/Deferred/Fail/Pending) + color mapping
- `crates/moai-studio-spec/src/state/spec_index.rs`: SpecIndex — scan, parse, watch all SPEC directories
- `crates/moai-studio-spec/src/state/spec_record.rs`: SpecRecord — per-SPEC metadata + files + requirements
- `crates/moai-studio-spec/src/state/kanban.rs`: KanbanStage enum
- `crates/moai-studio-spec/src/state/kanban_persist.rs`: Sidecar .kanban-stage file persistence
- `crates/moai-studio-spec/src/branch.rs`: Git branch parser — feature/SPEC-XXX-slug regex + active branch detection

### New UI Module: spec_ui

- `crates/moai-studio-ui/src/spec_ui/mod.rs`: Module registration
- `crates/moai-studio-ui/src/spec_ui/list_view.rs`: SpecListView GPUI Entity + Render (SPEC card list)
- `crates/moai-studio-ui/src/spec_ui/detail_view.rs`: SpecDetailView GPUI Entity + Render (EARS/AC metadata overlay)
- `crates/moai-studio-ui/src/spec_ui/kanban_view.rs`: KanbanBoardView — 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) + keyboard navigation
- `crates/moai-studio-ui/src/spec_ui/command_client.rs`: MoaiCommandClient — subprocess spawn + stream-json decode
- `crates/moai-studio-ui/src/spec_ui/sprint_panel.rs`: SprintContractPanel — revision timeline extraction

### Integration Tests

- `crates/moai-studio-spec/tests/integration_test.rs`: 7 integration tests

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-SU-1 | PASS | SPEC-V3-009 directory appears as card in SpecListView |
| AC-SU-2 | PASS | EARS table parsed: RG + REQ IDs extracted correctly |
| AC-SU-3 | PASS | AC state classified into 5 colors (Full/Partial/Deferred/Fail/Pending) |
| AC-SU-4 | PASS | External spec.md change → 100ms auto-refresh via watcher |
| AC-SU-5 | PASS | Missing acceptance.md → graceful placeholder, no panic |
| AC-SU-6 | PASS | KanbanBoardView 4 lanes with all SPECs distributed |
| AC-SU-7 | PASS | Keyboard navigation (up/down + Enter) + sidecar persist + reload restore |
| AC-SU-8 | PASS | Git branch parser: active feature branch detected + no-branch hint |
| AC-SU-9 | PASS | "Run" button → moai run subprocess spawn + stream-json decode |
| AC-SU-10 | PASS | Subprocess exit code + last status recorded in card badge |
| AC-SU-11 | PASS | Sprint Contract Revision headings extracted to timeline panel |
| AC-SU-12 | PASS | terminal/panes/tabs core 0 byte change (RootView entry point registration excluded) |

## Test Coverage

- MS-1: 96 tests (67 moai-studio-spec lib + 7 integration + 22 spec_ui)
- MS-2: 24 new tests (kanban_view + kanban_persist)
- MS-3: 30 new tests (branch parser + command_client + sprint_panel)
- Total: 150 tests across all milestones
- clippy 0, fmt PASS at each milestone

## Known Limitations

- Mouse drag-and-drop Kanban not implemented (keyboard-only per USER-DECISION-SU-B option a)
- RootView entry point registration delegated to follow-up SPEC
- SPEC dependency graph visualization not in scope
- spec.md content editing not in scope (read-only + stage transition only)
- Multi-project workspace support not in scope

## USER-DECISION Resolutions

- USER-DECISION-SU-A (Markdown parser): pulldown-cmark v0.13 (option a, recommended)
- USER-DECISION-SU-B (Kanban DnD): keyboard-only first, mouse deferred (option a)
- USER-DECISION-SU-C (moai-adk integration): subprocess + stream-json (option a, recommended)
