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

- 실제 rename modal — gpui Entity + text input + confirm 버튼
- 실제 delete confirmation modal — destructive 액션 가드
- 실제 reorder dispatch — `WorkspacesStore::move_position` 추가 + UI 갱신
- workspace_row 우클릭 핸들러 와이어링 (RootView 통합)
- Quick switcher (⌘/Ctrl+,) — audit 후속 carry
- D-4 Global search / D-5 Color tags / D-6 Drag-and-drop add — audit 명시 v0.2.0 deferred
