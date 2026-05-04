# SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001 — Progress

| Field | Value |
|-------|-------|
| Status | merged-pending (impl + tests done, PR + admin merge pending) |
| Branch | feature/SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001 |
| Base | main 3338f35 |

## Milestone Tracker

### MS-1 — workspace_dot_color helper + workspace_row dot 갱신

| AC | Status | Note |
|----|--------|------|
| AC-WDC-1 | ✅ | inactive returns (ws_color, None) |
| AC-WDC-2 | ✅ | active returns (ws_color, Some(ACCENT)) |
| AC-WDC-3 | ✅ | 12×2 palette table |
| AC-WDC-4 | ✅ | workspace_row active smoke |
| AC-WDC-5 | ✅ | workspace_row inactive smoke |
| AC-WDC-6 | ✅ | cargo build/clippy/fmt + ui tests PASS |

## Implementation summary

- `workspace_dot_color(ws_color: u32, is_active: bool) -> (u32, Option<u32>)` helper 신규 — pub(crate). inner == ws_color, outer = is_active 시 Some(tok::ACCENT).
- `workspace_row` 의 dot 영역이 inner = ws.color, active 시 outer 1px ring tok::ACCENT 추가.
- 5 신규 tests: inactive/active 단순 케이스 + palette 12×2 table + active/inactive smoke.
- 1350 → **1355 PASS** (+5), clippy/fmt clean, 회귀 0.

## Notes

- Lightweight SPEC fast-track 13번째.
- D-5 carry tail 의 마지막 결선 — sidebar 시각 가시화.
- main-direct 구현.
