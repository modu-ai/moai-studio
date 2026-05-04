# SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001 — Sidebar dot reflects workspace color tag

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001 |
| **Title** | Sidebar workspace_row dot uses ws.color (D-5 visible closure) |
| **Status** | in_progress |
| **Priority** | Low (P3 polish) |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-3-0-WORKSPACE-COLOR-001 (PR #101), SPEC-V0-3-0-COLOR-PICKER-RENDER-001 (PR #105) |
| **Cycle** | v0.3.0 Sprint 2 #3 (audit Top 16 #14 D-5 visible closure) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안. D-5 carry-forward 마지막 결선 — color tag 가 sidebar 에 시각적으로 표시되도록 dot 색상을 `ws.color` 로 적용. Lightweight 한도 (≤10KB / ≤8 AC / 1 MS) 충족.

## 1. Purpose

D-5 audit 항목 "Workspace 별 사용자 선택 색상" 은 PR #101 (palette + ColorPickerModal logic) + PR #105 (GPUI render + commit-to-store) 로 사용자 선택 → 저장 경로까지 작동하지만, **sidebar 의 workspace_row dot 이 여전히 `tok::ACCENT` / `tok::BORDER_STRONG` 만 사용하여 ws.color 가 시각적으로 보이지 않는다**. 본 SPEC 은 dot 색상을 `ws.color` 로 적용하고 active 강조는 별도 outer ring 으로 분리하여 D-5 의 사용자 경험을 가시적으로 닫는다.

## 2. Goals

- `workspace_dot_color(ws_color: u32, is_active: bool) -> (u32, Option<u32>)` helper 분리 — (inner_color, outer_ring_color)
- `workspace_row` 의 dot 영역이 inner = ws.color 사용. active 시 outer ring (`tok::ACCENT`, 1px) 추가
- TRUST 5 gates ALL PASS, 회귀 0

## 3. Non-Goals / Exclusions

- context menu 의 ChangeColor row 옆에 현재 color swatch hint — 별 SPEC carry (UX nice-to-have, render 시그니처 변경 필요)
- color tag 시각화 외 다른 sidebar 시각 변경 — 무관 (banner / sidebar layout 등)
- color picker overlay 자체 변경 (PR #105 완료) — non-goal
- workspace_dot_color 함수의 outer_ring 두께/위치 GPUI element 트리 검증 — element 비교 인프라 부재 (rename modal과 동일 정책: helper level + manual 시각 검증)

## 4. Requirements

- REQ-WDC-001: 신규 helper `pub(crate) fn workspace_dot_color(ws_color: u32, is_active: bool) -> (u32, Option<u32>)` 가 `crates/moai-studio-ui/src/lib.rs` 또는 `workspace_color.rs` 에 추가. inner = ws_color 그대로, outer = active 시 `Some(tok::ACCENT)`, inactive 시 `None`.
- REQ-WDC-002: `workspace_row(ws, is_active)` 가 `workspace_dot_color(ws.color, is_active)` 결과를 사용하여 dot 영역을 그림. inner dot 8px + (active 시) outer ring 1px solid border `tok::ACCENT`.
- REQ-WDC-003: ChangeColor commit (`commit_color_picker_modal`) 후 `cx.notify()` 호출 → sidebar 재렌더 시 새 ws.color 반영. (현재 listener 가 이미 cx.notify 호출 — 회귀 없음만 확인.)
- REQ-WDC-004: 기존 dot 의 무지개색 폴백 (`v0.1.0 에서 모든 row orange-red`) 주석 갱신.
- REQ-WDC-005: cargo build / clippy / fmt + ui crate test ALL PASS, 회귀 0.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-WDC-1 | ws_color = 0xff0000, is_active = false | `workspace_dot_color` | (0xff0000, None) | unit test |
| AC-WDC-2 | ws_color = 0x00ff00, is_active = true | `workspace_dot_color` | (0x00ff00, Some(tok::ACCENT)) | unit test |
| AC-WDC-3 | palette 12 colors × {active, inactive} 24 조합 | `workspace_dot_color` | inner == 입력 ws_color 모두 일치, outer 는 active 일 때만 Some | unit test (table-driven) |
| AC-WDC-4 | workspace_row(ws, true) | 호출 | panic 없이 Stateful<Div> 반환 | smoke test |
| AC-WDC-5 | workspace_row(ws, false) | 호출 | panic 없이 Stateful<Div> 반환 | smoke test |
| AC-WDC-6 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 1350 + 신규 tests, 회귀 0 | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | workspace_dot_color helper 추가 + workspace_row dot 갱신 + tests |
| `.moai/specs/SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-WORKSPACE-DOT-COLOR-001/progress.md` | created | 구현 후 갱신 |

FROZEN:
- `Workspace.color: u32` 필드 시그니처
- `WORKSPACE_COLOR_PALETTE` const

## 7. Test Strategy

신규 unit tests (lib.rs `tests` module):
1. `workspace_dot_color_inactive_returns_ws_color_only` (AC-WDC-1)
2. `workspace_dot_color_active_returns_ws_color_plus_accent_ring` (AC-WDC-2)
3. `workspace_dot_color_table_for_all_palette_entries` (AC-WDC-3, 12×2 조합)
4. `workspace_row_active_smoke` (AC-WDC-4)
5. `workspace_row_inactive_smoke` (AC-WDC-5)

기존 1350 ui tests 회귀 0.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 2 #3
Parent: SPEC-V0-3-0-WORKSPACE-COLOR-001 + SPEC-V0-3-0-COLOR-PICKER-RENDER-001 (D-5 carry tail)
Carry-to: context menu ChangeColor row swatch hint — 별 SPEC nice-to-have
