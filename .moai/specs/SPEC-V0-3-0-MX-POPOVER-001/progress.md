# SPEC-V0-3-0-MX-POPOVER-001 — Progress

## HISTORY

- 2026-05-04: spec.md 초안 작성, MS-1 진입. v0.3.0 Sprint 2 #1.
- 2026-05-04: MS-1 구현 완료. mx_gutter.rs 에 hit-test/MxHoverFsm/should_flip_left/render_popover_text + 12 단위 테스트 추가. ui 1332 → 1344 (+12). clippy + fmt clean.

## MS-1 (completed)

- [x] AC-MXP-1: `hit_test_gutter(viewport_y, line_height, num_lines) -> Option<usize>` 구현 (line band/negative/zero line_height 가드 포함). 2 단위 테스트.
- [x] AC-MXP-2: `MxHoverFsm` + `MxPopoverState::{Closed, Hovering, Open}` + 200ms `MX_HOVER_DEBOUNCE_MS` debounce. `tick()` 가 Hovering→Open 승격. 2 단위 테스트 (debounce 통과 + 줄 변경 시 timer 리셋).
- [x] AC-MXP-3: `render_popover_text(&MxPopoverData) -> String` 헬퍼 — icon + body + reason(opt) + spec_id(opt) 순서로 직렬화. 2 단위 테스트.
- [x] AC-MXP-4: `on_mouse_leave_all()` — popover 닫힘 + hover state 초기화. `set_mouse_in_popover(true)` 시 gutter 빈 줄로 이동해도 popover 유지 (사용자가 popover 영역으로 진입한 케이스). 2 단위 테스트.
- [x] AC-MXP-5: `on_escape()` — Hovering / Open 양쪽 상태에서 Closed 로 전이. 2 단위 테스트.
- [x] AC-MXP-6: `should_flip_left(anchor_x, popover_width, viewport_width) -> bool` — 우측 잔여 폭 < popover 폭일 때 true. 2 단위 테스트 (flip 케이스 + no-flip 케이스).
- [x] AC-MXP-7: 기존 mx_gutter 24 tests 회귀 0 (FROZEN 자료구조/Scanner 무수정).
- [x] AC-MXP-8: cargo build/clippy/fmt + ui 전체 tests ALL PASS (1332 → 1344, +12).

## Notes

- SPEC 의 +N 테스트 목표는 6 개였으나 실제 12 개 추가 (각 AC 당 일반 + 엣지 케이스 1 쌍씩). 추가 비용 무시할 수준이고 회귀 방어 강화.
- viewer 측 GPUI 결합 (실제 MouseMove → `on_gutter_hover` 호출, 실제 popover 요소 그리기, animation frame → `tick`) 은 carry-out — visual smoke 으로 별도 검증 필요. unit-testable 표면 (FSM + helper 함수) 만 본 SPEC 의 범위.
- audit-stale 사실 정정: `mx_gutter.rs` 에 `MXPopover` struct 는 존재하지 않음 (only `MxPopoverData`). audit feature-audit.md §4 #11 의 표현은 이번 PR 머지 후 갱신 권장 (별건).
