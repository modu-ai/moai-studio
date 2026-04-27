# SPEC-V3-010 Progress

**Started**: 2026-04-25
**Branch**: main (per-milestone feature branches: feature/SPEC-V3-010-dashboard-ms1, feature/SPEC-V3-010-ms2-cost-filter, feature/SPEC-V3-010-ms3)
**SPEC status**: implemented
**Completion date**: 2026-04-27

## Implementation Timeline

- 2026-04-25 `8e1d3e9` PR #9: SPEC-V3-010 spec/plan/research documents created (planning phase)
- 2026-04-25 `8be7d6f` PR #11: feat(agent): SPEC-V3-010 MS-1 — Agent Dashboard foundation (17 new tests, 464 workspace tests)
- 2026-04-26 `181c8d8` PR #35: feat(agent): SPEC-V3-010 MS-2 Cost tracker + event filter (AC-AD-4/5/6)
- 2026-04-27 `49c0c7e` PR #37: feat(agent): SPEC-V3-010 MS-3 Instructions graph + Control bar + Detail view (AC-AD-7~12)

## Milestone Status

- [x] MS-1: Stream parser + basic timeline (Ring buffer + SSE/stream ingest + TimelineView + DashboardView) — PR #11
- [x] MS-2: Cost tracker + event filter (CostPanel + FilterChips + daily/weekly aggregation) — PR #35
- [x] MS-3: Instructions graph + agent control + detail view (InstructionsGraph + ControlBar + EventDetail) — PR #37

## Key Files Changed

### New Crate: moai-studio-agent

- `crates/moai-studio-agent/Cargo.toml`: New crate definition
- `crates/moai-studio-agent/src/lib.rs`: Public API exports (AgentRunId, AgentRun, AgentRunStatus)
- `crates/moai-studio-agent/src/events.rs`: AgentEvent, AgentEventKind (Stream + Hook + Unknown variants)
- `crates/moai-studio-agent/src/ring_buffer.rs`: Ring buffer (1000 events cap, oldest eviction)
- `crates/moai-studio-agent/src/stream_ingest.rs`: Stream-json decode → AgentEvent conversion
- `crates/moai-studio-agent/src/sse_ingest.rs`: SSE hook event → AgentEvent conversion
- `crates/moai-studio-agent/src/view.rs`: AgentRun view model
- `crates/moai-studio-agent/src/cost.rs`: CostSnapshot, CostAggregation (session/daily/weekly USD)
- `crates/moai-studio-agent/src/filter.rs`: EventKind filter (stream + hook groups, dim/full toggle)
- `crates/moai-studio-agent/src/instructions.rs`: InstructionNode tree (6-layer scanner + hook rebuild)
- `crates/moai-studio-agent/src/control.rs`: stdin envelope writer (MOAI-CTRL: {json})

### New UI Module: agent

- `crates/moai-studio-ui/src/agent/mod.rs`: Module registration
- `crates/moai-studio-ui/src/agent/dashboard_view.rs`: AgentDashboardView — 5-pane split layout container
- `crates/moai-studio-ui/src/agent/timeline_view.rs`: EventTimelineView — reverse chronological event list + filter chips
- `crates/moai-studio-ui/src/agent/cost_panel_view.rs`: CostPanelView — session/today/week USD metrics
- `crates/moai-studio-ui/src/agent/control_bar.rs`: AgentControlBar — pause/resume/kill + confirm modal + status matrix
- `crates/moai-studio-ui/src/agent/detail_view.rs`: EventDetailView — JSON pretty-print + collapse + Copy as JSON

### Modified Files

- `crates/moai-studio-ui/Cargo.toml`: Added moai-studio-agent dependency
- `crates/moai-studio-ui/src/lib.rs`: agent module registration
- `crates/moai-studio-spec/src/branch.rs`: Minor adjustments for shared type compatibility

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-AD-1 | PASS | stream-json 1 line decode → AgentEvent |
| AC-AD-2 | PASS | hook-http SSE event → AgentEvent |
| AC-AD-3 | PASS | EventTimelineView 1000 events rendering |
| AC-AD-4 | PASS | Event filter chip dim/full toggle |
| AC-AD-5 | PASS | CostPanel session cumulative USD display |
| AC-AD-6 | PASS | CostPanel daily/weekly aggregation correct |
| AC-AD-7 | PASS | InstructionsGraph 6-layer tree display |
| AC-AD-8 | PASS | InstructionsGraph SessionStart/PreCompact rebuild |
| AC-AD-9 | PASS | AgentControlBar pause envelope delivery |
| AC-AD-10 | PASS | AgentControlBar kill confirm dialog + envelope |
| AC-AD-11 | PASS | EventDetailView JSON pretty-print + collapse |
| AC-AD-12 | PASS | terminal/panes/tabs core git diff = 0 |

## Test Coverage

- MS-1: 17 new tests (999 LOC added)
- MS-2: Additional cost + filter tests (877 LOC added)
- MS-3: Domain 14 + UI 23 = 37 new tests
- Total: 37 tests across MS-3, cumulative 54+ tests
- clippy clean, fmt clean at each milestone

## Known Limitations

- REQ-AD-032 (Markdown payload rendering via SPEC-V3-006): JSON-only default, markdown deferred
- Event persistence: in-memory ring buffer only (USER-DECISION-AD-A = A1, SQLite deferred)
- Multi-agent concurrent dashboard: not supported (v1.0.0 shows active agent 1 only)
- moai-supervisor crate: not created (minimal AgentHandle trait stub only)
- Real SSE ingestion depends on moai-hook-http server availability
- Real stream-json ingestion depends on Claude Code subprocess output format

## USER-DECISION Resolutions

- USER-DECISION-AD-A (Persistence): A1 — in-memory ring buffer, 1000 events (default)
- USER-DECISION-AD-B (Cost source): B1 — agent self-report via stream-json usage (default)
- USER-DECISION-AD-C (Control IPC): C1 — stdin envelope MOAI-CTRL: {json} (default)
- USER-DECISION-AD-D (Hook ingestion): D2 — HTTP SSE (default)
