# SPEC-V0-3-0-WORKSPACE-COLOR-001 — Progress

## Status

- 2026-05-04: 초안 작성 + MS-1 구현 완료. 8 AC ALL ✅. ui crate 1320 → 1326 (+6), workspace crate 10 → 12 (+2). clippy/fmt clean.

## Milestone Tracker

### MS-1 — palette + ColorPickerModal logic (✅ DONE)

| AC | Status | Note |
|----|--------|------|
| AC-WC-1 | ✅ | WORKSPACE_COLOR_PALETTE 12 distinct entries |
| AC-WC-2 | ✅ | next_color round-robin (count % 12) |
| AC-WC-3 | ✅ | WorkspacesStore::set_color 성공 + save persistence |
| AC-WC-4 | ✅ | set_color → NotFound on missing id |
| AC-WC-5 | ✅ | WorkspaceMenuAction::all() returns 5 distinct variants |
| AC-WC-6 | ✅ | ChangeColor.is_destructive() == false |
| AC-WC-7 | ✅ | ColorPickerModal::open/select/selected_color/target |
| AC-WC-8 | ✅ | cargo build/clippy/fmt + 1326 ui tests / 12 workspace tests PASS |

### Implementation summary

- 신규 module `crates/moai-studio-ui/src/workspace_color.rs` (~80 LOC + 2 tests):
  - `WORKSPACE_COLOR_PALETTE: [u32; 12]` const (Radix Colors 600 step, hue 분산)
  - `next_color(existing_count: usize) -> u32` — round-robin helper
- 신규 API `WorkspacesStore::set_color(&mut self, id, color)` (workspace crate, +2 tests)
- enum 확장 `WorkspaceMenuAction::ChangeColor` (5번째 variant):
  - `all()` → `[; 5]` (Rename / ChangeColor / Delete / MoveUp / MoveDown)
  - `label()` → "Change Color"
  - `is_destructive()` → false
  - `WorkspaceMenu::items()` → `[; 5]`
- 신규 struct `ColorPickerModal` (workspace_menu module 내 ~30 LOC + 1 test):
  - fields: `target_id: String`, `selected: u32`
  - methods: `open(target, current)`, `select(color)`, `selected_color()`, `target()`
- `WorkspaceMenuOutcome::OpenColorPicker { ws_id, current_color }` variant + 1 test
- `dispatch_workspace_menu_action` 의 ChangeColor arm 추가
- `RootView::color_picker_modal: Option<ColorPickerModal>` 신규 필드 (default None)
- `RootView::next_color_for_new_workspace(&self) -> u32` helper (REQ-WC-009 logic-level)
- `handle_workspace_menu_action_logic` 의 OpenColorPicker arm 추가 (mount modal)
- 기존 tests 의 4 → 5 갱신 (workspace_menu_action_all*, workspace_menu_action_destructive)

총 +8 신규 tests (ui +6 / workspace +2).

## Carry-Forward (별 SPEC)

- ColorPickerModal GPUI render 측 (overlay UI, swatch grid, hover state) — 차후 SPEC.
- workspace_menu 우클릭 dropdown 의 ChangeColor 항목 GPUI 표시 + dispatch 트리거 — 차후 SPEC.
- `handle_add_workspace` 시점 wizard build flow 의 round-robin color 적용 — wizard 통합 SPEC.

## Notes

- Lightweight SPEC fast-track 9번째 적용 (이전 8번째 = SPEC-V0-3-0-MENU-WIRE-001).
- main 세션 직접 fallback 유지 (sub-agent 1M context 회피).
- v0.3.0 cycle Sprint 1 세 번째 PR (PR #99 clippy, #100 menu-wire, this).
- audit Top 16 진척: D-5 (Workspace color tags) — palette + API + Modal logic GA, GPUI render carry.
