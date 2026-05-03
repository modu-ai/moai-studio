# SPEC-V3-004 Progress

**Started**: 2026-04-25
**Branch**: feature/SPEC-V3-004-render
**SPEC status**: implemented
**Completion date**: 2026-04-25

## Implementation Timeline

- 2026-04-25 `8e1d3e9` PR #9: feat(render): SPEC-V3-004 Render Layer + V3-005~011 SPEC backlog + v0.1.0 release plan (MS-1/2/3, all 8 AC GREEN)
- 2026-04-25 `57adab8` feat(ui): Phase 1+2 toolbar wizard image implementation complete (integration commit, not V3-004-specific)
- 2026-04-25 `af5e25e` merge(develop): Phase 1+2 + SPEC-V3-004 integration (integration merge, not V3-004-specific)

## Milestone Status

- [x] MS-1: TabContainer Entity + impl Render for TabContainer + RootView field replacement — PR #9
- [x] MS-2: PaneTree render_pane_tree recursive conversion + key dispatch (Cmd+T, Cmd+1~9, Cmd+\) — PR #9
- [x] MS-3: Divider drag e2e + boundary rejection + AC-P-4 carry-over resolution — PR #9

## Key Files Changed

- `crates/moai-studio-ui/src/lib.rs`: RootView `pane_splitter` replaced with `tab_container: Option<Entity<TabContainer>>`, key dispatch handler registration, `Render for RootView` updated
- `crates/moai-studio-ui/src/tabs/container.rs`: `impl Render for TabContainer` added, tab bar + active tab PaneTree rendering
- `crates/moai-studio-ui/src/panes/render.rs`: Recursive `render_pane_tree<L: Render + 'static>` function, HStack/VStack layout conversion, divider element per Split node
- `crates/moai-studio-ui/src/tabs/keys.rs`: `keystroke_to_tab_key` conversion function added
- `crates/moai-studio-ui/src/panes/divider.rs`: GPUI element conversion helper added

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-R-1 | PASS | RootView.tab_container Entity creation + Render invocation |
| AC-R-2 | PASS | flex_row container + leaf elements + divider per Split node |
| AC-R-3 | PASS | Cmd+T → new tab visible |
| AC-R-4 | PASS | Cmd+\ → horizontal split + divider visible |
| AC-R-5 | PASS | Divider drag boundary rejection (MIN_COLS clamp) — AC-P-4 carry-over resolved |
| AC-R-6 | PASS | USER-DECISION gpui-test-support resolved (logic-level verification) |
| AC-R-7 | PASS | 3-level split → 3 dividers with correct orientation |
| AC-R-8 | PASS | 433 workspace tests, 0 regression on terminal/panes/tabs |

## Test Coverage

- 433 workspace tests at completion, 0 fail, 8 ignored
- clippy 0 warnings, fmt PASS, bench PASS
- Logic-level unit tests for render_pane_tree, key dispatch, divider drag

## Known Limitations

- Leaf payload was placeholder (PTY spawn per pane deferred to separate SPEC)
- Tab close UI element (X button) not in scope
- KaTeX/Mermaid rendering deferred to SPEC-V3-006
- USER-DECISION gpui-test-support: logic-level verification approach used

## USER-DECISION Resolutions

- gpui-test-support-adoption-v3-004: Logic-level fallback (option b equivalent) — integration tests via logic assertions

## Carry-Over Resolution

- SPEC-V3-003 AC-P-4 (TabContainer divider render integration): **RESOLVED** in AC-R-5
- SPEC-V3-003 AC-P-5 (gpui test-support re-evaluation): **RESOLVED** in AC-R-6

---

## MS-4 (2026-05-01 sess 8) — D-2 Workspace switcher polish skeleton (audit D-2)

Branch: feature/SPEC-V3-004-ms4-workspace-switcher-polish

### Implementation

- `crates/moai-studio-ui/src/workspace_menu.rs` (신규) — `WorkspaceMenuAction` enum (Rename / Delete / MoveUp / MoveDown) + `WorkspaceMenu` struct (target id + position) + mutation API (`open_for` / `close` / `is_visible_for` / `visible_target` / `visible_position` / `is_open` / `items`). `MenuPosition` Copy + PartialEq.
- `crates/moai-studio-ui/src/lib.rs` — `pub mod workspace_menu` 등록.
- spec.md §8 에 MS-4 milestone + REQ-D2-MS4-1~3 [frozen-zone] 추가, §10 에 AC-D2-1~5 추가.

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-D2-1 | WorkspaceMenuAction 4 variant 노출 + label 매핑 비어있지 않음 | ✅ |
| AC-D2-2 | `WorkspaceMenu::default()` 가 closed 상태 | ✅ |
| AC-D2-3 | `open_for("ws-1", x, y)` → is_visible_for("ws-1") true, 다른 ws false, position 저장 | ✅ |
| AC-D2-4 | 두 번째 open_for 가 prior target 교체 (single-menu invariant) | ✅ |
| AC-D2-5 | `close()` 가 menu invisible + target/position None | ✅ |

### Test count

- 신규: 11 (workspace_menu::tests — action variants/labels/destructive/default/open/replace/close/idempotent/items helper/position equality/reopen update)
- 전체 ui crate tests 1137 → 1148, clippy 0, fmt clean

### Deferred (carry to follow-up PR or v0.2.0)

- 실제 rename modal — gpui Entity + text input + confirm 버튼 (logic-level은 MS-5 RenameModal 완성, GPUI overlay render 만 carry)
- 실제 delete confirmation modal — destructive 액션 가드 (logic-level은 MS-5 DeleteConfirmation 완성, GPUI overlay render 만 carry)
- 실제 reorder dispatch — `WorkspacesStore::move_up/move_down` MS-5 완성 ✅
- workspace_row 우클릭 핸들러 와이어링 (RootView 통합) — MS-5 `handle_workspace_menu_action` 완성, render side wire 만 carry
- Quick switcher (⌘/Ctrl+,) — audit 후속 carry
- D-4 Global search ✅ DONE (PR #78~#81, SPEC-V0-2-0-GLOBAL-SEARCH-001 GA)
- D-5 Color tags / D-6 Drag-and-drop add — audit 명시 v0.2.0 deferred

---

## MS-5 (2026-05-04 sess 11) — D-2 Workspace switcher real dispatch (audit D-2 follow-up)

Branch: feature/SPEC-V3-004-ms5-d2-followup

### Implementation

- `crates/moai-studio-workspace/src/lib.rs`:
  - `WorkspaceError::EmptyName` variant 추가
  - `WorkspacesStore::rename(id, new_name)` — name 갱신 + save (REQ-D2-MS5-1)
  - `WorkspacesStore::move_up(id)` — 인덱스 1 감소 (0 인덱스 no-op + Ok) (REQ-D2-MS5-2)
  - `WorkspacesStore::move_down(id)` — 인덱스 1 증가 (last 인덱스 no-op + Ok) (REQ-D2-MS5-2)
- `crates/moai-studio-ui/src/workspace_menu.rs`:
  - `RenameModal` struct (target_id + buffer + open / set_buffer / commit / cancel / is_open / target_id / buffer) (REQ-D2-MS5-3)
  - `DeleteConfirmation` struct (target_id + open / confirm / cancel / is_open / target_id) (REQ-D2-MS5-4)
  - `WorkspaceMenuOutcome` enum (OpenRenameModal / OpenDeleteConfirmation / Reordered / Unknown)
  - `dispatch_workspace_menu_action(action, ws_id, store) -> WorkspaceMenuOutcome` adapter (REQ-D2-MS5-5)
- `crates/moai-studio-ui/src/lib.rs`:
  - RootView 에 `rename_modal: Option<RenameModal>` + `delete_confirmation: Option<DeleteConfirmation>` + `store: WorkspacesStore` 필드 추가 (R3 새 필드만)
  - `handle_workspace_menu_action_logic(action, ws_id) -> WorkspaceMenuOutcome` (logic-level test 가능)
  - `handle_workspace_menu_action(action, ws_id, cx)` (GPUI context-aware, cx.notify())

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| REQ-D2-MS5-1 | WorkspacesStore::rename + EmptyName + NotFound 에러 | ✅ |
| REQ-D2-MS5-2 | WorkspacesStore::move_up + move_down (경계 no-op + NotFound) | ✅ |
| REQ-D2-MS5-3 | RenameModal open/set_buffer/commit/cancel | ✅ |
| REQ-D2-MS5-4 | DeleteConfirmation open/confirm/cancel | ✅ |
| REQ-D2-MS5-5 | dispatch_workspace_menu_action + RootView wire | ✅ |

### Test count

- 신규: 26 (workspace 9 + workspace_menu 14 + RootView 3)
- moai-studio-workspace: 17 → 26 (+9)
- moai-studio-ui workspace_menu: 11 → 25 (+14)
- ui crate 전체: 1193 → 1210 (+17)
- clippy 0 warning, fmt clean, 워크스페이스 회귀 0

### Carry (다음 PR — RootView 우클릭 wire)

`WorkspaceMenu::open_for` 로 우클릭 이벤트 받은 뒤 호출:

```rust
root_view.update(cx, |view, cx| {
    view.handle_workspace_menu_action(action, &ws_id, cx);
});
```

`handle_workspace_menu_action` 가 outcome 에 따라 rename_modal/delete_confirmation 자동 설정 + Reordered 시 cx.notify() 호출. 별 PR 에서 GPUI render side mount 만 추가하면 e2e 완성.

---

## MS-6 (2026-05-04 sess 12) — D-2 Workspace switcher GPUI overlay mount (audit D-2 fully closed)

Branch: feature/SPEC-V3-004-ms6-overlay-mount

### Implementation

- `crates/moai-studio-ui/src/lib.rs`:
  - `RootView` 에 `workspace_menu: workspace_menu::WorkspaceMenu` 필드 신규 (REQ-D2-MS6-1, R3 새 필드만).
  - `RootView::new` 에 `workspace_menu: WorkspaceMenu::default()` 초기화 추가.
  - 새 helper 메서드 6개 (모두 `pub` + 1개 private `sync_workspaces_from_store`):
    - `open_workspace_menu_at(ws_id, x, y)` (REQ-D2-MS6-2 / AC-D2-12)
    - `click_workspace_menu_item(action)` (REQ-D2-MS6-3 / AC-D2-13) — visible_target → handle_workspace_menu_action_logic → close
    - `commit_rename_modal() -> Option<(String, String)>` (REQ-D2-MS6-4 / AC-D2-14 rename half) — store.rename + sync_workspaces_from_store + clear
    - `cancel_rename_modal()`
    - `confirm_delete_modal() -> Option<String>` (REQ-D2-MS6-4 / AC-D2-14 delete half) — store.remove + sync + active_id 재할당 + clear
    - `cancel_delete_modal()`
  - `sync_workspaces_from_store()` private @MX:ANCHOR helper — fan_in≥3 (Reordered branch + click_workspace_menu_item + commit_rename_modal + confirm_delete_modal).
  - **MS-5 fix-up**: `handle_workspace_menu_action_logic` 의 `Reordered` arm 이 기존에는 cx.notify() 만 트리거하고 self.workspaces vector 를 갱신하지 않아 sidebar 가 stale 표시되던 잠재 결함 → `sync_workspaces_from_store()` 호출 추가.
  - GPUI render side wire:
    - `workspace_row` 에 `MouseButton::Right` listener 추가 — `open_workspace_menu_at(ev.position.x, ev.position.y)` + cx.notify(). 좌클릭 (activate) 동작 무변경.
    - `Render for RootView` chain 끝에 3개 overlay branch 추가 (palette/settings_modal/spec_panel 패턴 mirror):
      - `workspace_menu.is_open()` → `render_workspace_context_menu_overlay` (absolute positioned 4 항목 menu)
      - `rename_modal.is_some()` → `render_rename_modal_overlay` (centered scrim + buffer display + Commit/Cancel)
      - `delete_confirmation.is_some()` → `render_delete_confirmation_overlay` (centered scrim + warning + Confirm/Cancel)
  - 새 helper 함수 3개 (lib.rs file scope, render_spec_panel_overlay 직후): `render_workspace_context_menu_overlay`, `render_rename_modal_overlay`, `render_delete_confirmation_overlay`.

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-D2-11 | RootView::new 가 workspace_menu 를 closed 로 초기화 | ✅ |
| AC-D2-12 | open_workspace_menu_at 가 target + position 기록 | ✅ |
| AC-D2-13 | click_workspace_menu_item 이 dispatch + 메뉴 close (single-menu invariant) | ✅ |
| AC-D2-14 | commit_rename_modal + confirm_delete_modal 이 store mutation + workspaces sync | ✅ |

### Test count

- 신규: 13 (T8 블록)
  - test_workspace_menu_default_closed_on_root_view_new (AC-D2-11)
  - test_open_workspace_menu_at_records_target_and_position (AC-D2-12)
  - test_click_workspace_menu_rename_opens_rename_modal_and_closes_menu (AC-D2-13)
  - test_click_workspace_menu_delete_opens_delete_confirmation_and_closes_menu (AC-D2-13)
  - test_click_workspace_menu_move_up_calls_store_and_syncs_workspaces (AC-D2-13 + Reordered fix)
  - test_commit_rename_modal_renames_in_store_and_syncs_workspaces (AC-D2-14 rename)
  - test_confirm_delete_modal_removes_from_store_and_syncs_workspaces (AC-D2-14 delete)
  - test_confirm_delete_modal_reassigns_active_when_active_deleted (active_id invariant)
  - test_cancel_rename_modal_clears_state
  - test_cancel_delete_modal_clears_state
  - test_click_workspace_menu_item_noop_when_closed
  - test_commit_rename_modal_returns_none_when_closed
  - test_confirm_delete_modal_returns_none_when_closed
- moai-studio-ui lib tests: 1218 → 1231 (+13)
- clippy 0 warning, fmt clean, 워크스페이스 회귀 0

### Done

- D-2 audit 항목: PARTIAL → DONE. 우클릭 → ContextMenu → Rename/Delete modal → store mutation → sidebar 재렌더 풀 e2e 완성 (logic-level 검증 + GPUI render mount).
- MS-5 carry "RootView 우클릭 wire (별 milestone)" 해소.
- MS-5 latent bug (Reordered no sync) 부수 해결.

### Carry (다음 SPEC 또는 별 PR)

- ContextMenu keyboard navigation (Arrow / Esc / Enter) — 별 PR.
- RenameModal text input 실 wire (per-keystroke set_buffer) — 별 PR (text input infrastructure 필요).
- D-5 Color tags / D-6 Drag-and-drop add — v0.2.0 별 SPEC 그대로 carry.
