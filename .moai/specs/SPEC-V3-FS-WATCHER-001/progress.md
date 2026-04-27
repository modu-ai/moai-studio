# SPEC-V3-FS-WATCHER-001 Progress

**Started**: 2026-04-27
**Branch**: feature branches (squash merged into main)
**SPEC status**: implemented
**Completion date**: 2026-04-27

## Implementation Timeline

- 2026-04-27 SPEC v1.1.0 created — 2 USER-DECISION gates resolved — PR #43 (`0711947`)
- 2026-04-27 Polling-based deterministic pattern implemented — PR #48 (`02e4828`)

## USER-DECISION Gates (All Resolved)

- **FW-A**: A3 polling with bounded retry (5s deadline + 50~100ms polling, ~10 LOC, zero new deps)
- **FW-B**: B1 cargo test name filter (tmux-test job substring filter + file watcher isolated step)
- **FW-C**: OMITTED (research §4 recommendation — notify v7 kept as default)

## Task Status

- [x] SPEC v1.1.0 authoring with resolved USER-DECISION gates
- [x] test_detect_file_creation: fixed sleep → polling-with-bounded-retry (AC-FW-1, AC-FW-2)
- [x] test_unwatch_stops_events: fixed sleep → polling-with-bounded-retry (AC-FW-3)
- [x] macOS /private/var ↔ /var symlink workaround: path comparison via file_name()
- [x] CI isolation: tmux-test job cargo test substring filter 'tmux' (REQ-FW-010, AC-FW-5)
- [x] File watcher tests isolated in separate CI step (continue-on-error: true) (REQ-FW-011, AC-FW-6)

## Key Files Changed

- `crates/moai-fs/src/lib.rs`: Test functions rewritten — fixed sleep replaced with polling-with-bounded-retry (~10 LOC changed per test)
- `.github/workflows/ci-v3-pane.yml`: tmux-test job substring filter + file watcher isolated step

## Test Coverage

- test_detect_file_creation: 5s deterministic upper bound + 100ms polling slice, stress 30/30 PASS
- test_unwatch_stops_events: same polling pattern, typical case <= 1s
- AC coverage: AC-FW-1, AC-FW-2, AC-FW-3, AC-FW-5, AC-FW-6

## Known Limitations

- Public API unchanged (REQ-FW-005) — only test internals modified
- No new dependencies added (REQ-FW-006)
- notify v7 retained as default watcher backend (FW-C omitted per research recommendation)
