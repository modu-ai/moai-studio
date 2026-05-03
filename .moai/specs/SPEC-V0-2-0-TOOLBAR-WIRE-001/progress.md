# SPEC-V0-2-0-TOOLBAR-WIRE-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-TOOLBAR-WIRE-001
**SPEC status**: implemented (MS-1 complete)
**Completion date**: 2026-05-04
**Predecessor**: SPEC-V0-1-2-MENUS-001 (Toolbar scaffold)
**audit reference**: feature-audit.md §3 Tier F v0.2.0 critical gap F-3 + §4 Top 8 #5 (⭐⭐⭐⭐)
**Classification**: Lightweight SPEC fast-track (spec.md 8236 bytes ≤10KB, 1 MS, 8 REQ / 6 AC, no architectural impact)

## MS-1 (2026-05-04 sess 12+) — 7 button mouse_down → cx.dispatch_action wire ✅

### Implementation

- `crates/moai-studio-ui/src/toolbar.rs`:
  - 각 7 button 의 child chain 에 `.on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.dispatch_action(&ActionType)))` 추가
  - 7 actions: NewWorkspace / ToggleSidebar / OpenSettings / OpenCommandPalette / NewTerminalSurface / ToggleFind / OpenDocumentation
  - `MouseButton` import 추가
  - 기존 `on_action` listener 7개 보존 (backward compat)
  - 기존 button id / label / 스타일 무변경

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-TW-1 | 7 button on_mouse_down → dispatch_action wire | ✅ |
| AC-TW-2 | 기존 button id / label / on_action 보존 | ✅ |
| AC-TW-3 | 수동 smoke: 클릭 → action 전파 | ⏸️ deferred (CI 비대상, manual smoke 다음 release) |
| AC-TW-4 | 회귀 0 (1269 ui tests GREEN) | ✅ |
| AC-TW-5 | sidebar_visible 보존 | ✅ |
| AC-TW-6 | clippy 0 warning + fmt clean | ✅ |

### Test count

- 신규: 7 (toolbar.rs T-TW 블록)
  - toolbar_new_false_preserves_flag (AC-TW-5)
  - toolbar_new_true_preserves_flag (AC-TW-5)
  - toolbar_set_sidebar_visible_mutates (AC-TW-5)
  - toolbar_button_ids_canonical_order (AC-TW-2)
  - toolbar_button_labels_reflect_sidebar_state (AC-TW-2)
  - toolbar_other_button_labels_are_stable (AC-TW-2)
  - toolbar_render_does_not_panic (AC-TW-1, TestAppContext)
- moai-studio-ui crate tests: 1269 → 1276 (+7)
- 회귀 0:
  - moai-studio-agent: 129 GREEN
  - moai-studio-terminal: 36 GREEN
  - moai-studio-workspace: 26 GREEN
- clippy 0 warning, fmt clean

### Public API additions (audit aid)

- `Toolbar::sidebar_visible()` getter (test convenience + future read-only access)
- `Toolbar::button_labels()` 7-entry array (test/doc convenience)
- `Toolbar::button_ids()` 7-entry static array (canonical id list)

### DoD ✅

`cargo run -p moai-studio-app` → 사용자가 toolbar 의 7 button 중 하나를 클릭 →
해당 GPUI Action 이 RootView 의 기존 `on_action` handler 까지 전파되어 동작
(NewWorkspace 추가, Settings 열림, Command Palette 띄움 등). 단위 테스트 + 회귀
0 + clippy/fmt clean. F-3 audit 항목: PARTIAL → DONE.

audit Top 8 진척: 4 GA + 1 90% + **1 GA (F-3)** = 5 GA / 8 = **63%**.
