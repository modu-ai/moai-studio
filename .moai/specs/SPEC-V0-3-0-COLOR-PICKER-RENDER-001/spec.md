# SPEC-V0-3-0-COLOR-PICKER-RENDER-001 — ColorPickerModal GPUI render

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-COLOR-PICKER-RENDER-001 |
| **Title** | ColorPickerModal 4×3 swatch overlay + commit/cancel wire |
| **Status** | in_progress |
| **Priority** | Low (P3 polish) |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-3-0-WORKSPACE-COLOR-001 (logic merged PR #101) |
| **Cycle** | v0.3.0 Sprint 2 #2 (audit Top 16 #14 D-5 carry tail) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안. 부모 SPEC carry-forward #1 (ColorPickerModal GPUI render). Lightweight 한도 (≤10KB / ≤8 AC / 1 MS) 충족.

## 1. Purpose

`SPEC-V0-3-0-WORKSPACE-COLOR-001` 에서 `ColorPickerModal` 의 logic-only state machine + `RootView::color_picker_modal` slot 까지 구현되었으나, GPUI render 측 (overlay UI, swatch grid, hover state) 와 commit-to-store 결선이 carry-forward 로 남아있다. 본 SPEC 은 `render_rename_modal_overlay` 와 동일한 시각 idiom 으로 4×3 12-swatch grid overlay 를 추가하고, Commit 시 `WorkspacesStore::set_color` 를 호출 + sidebar sync 까지 결선한다.

## 2. Goals

- `RootView::commit_color_picker_modal()` — 모달의 `selected_color` 를 store 에 반영 + sidebar sync + 모달 dismiss
- `RootView::cancel_color_picker_modal()` — 변경 없이 모달 dismiss
- `RootView::color_picker_select(color)` — 모달의 in-progress 선택 갱신
- `render_color_picker_overlay(modal, cx)` — scrim + 카드 + 4×3 swatch grid + Cancel/Commit 버튼
- `render()` body 에 `color_picker_modal == Some` 분기 mount (rename_modal 과 동일 패턴)
- 선택된 swatch 는 강조 border 표시
- TRUST 5 gates ALL PASS, ui crate 회귀 0

## 3. Non-Goals / Exclusions

- workspace_menu 우클릭 dropdown 의 ChangeColor 항목 GPUI 표시 — 별 SPEC carry (부모 carry #2)
- color customization (사용자 hex 입력) — 12 preset 만
- 색상 변경 후 sidebar workspace dot 의 즉시 재렌더 검증 — sync_workspaces_from_store 위임으로 충분 (이미 다른 mutation 들이 동일 경로 사용)
- GPUI element 트리 구조 자체 검증 — 기존 modal 들과 동일하게 logic-level 만 unit test

## 4. Requirements

- REQ-CPR-001: `RootView::commit_color_picker_modal(&mut self) -> Option<(String, u32)>` — `color_picker_modal` 이 Some 일 때 `(target, selected_color)` 추출 후 `store.set_color(target, color)` 호출, 성공 시 `sync_workspaces_from_store()` + 모달 dismiss + `Some((target, color))` 반환. None 또는 store err 시 모달 dismiss + None 반환 (err 는 tracing::warn 로 기록).
- REQ-CPR-002: `RootView::cancel_color_picker_modal(&mut self)` — `color_picker_modal = None` 만 설정.
- REQ-CPR-003: `RootView::color_picker_select(&mut self, color: u32)` — `color_picker_modal` 이 Some 일 때 modal.select(color) 위임.
- REQ-CPR-004: `render_color_picker_overlay(modal: &ColorPickerModal, cx: &mut Context<RootView>) -> impl IntoElement` — scrim (`0x08_0c_0b_8c`) + 360px 카드 + "Pick Workspace Color" 타이틀 + 4×3 swatch grid (각 swatch 28px, gap 8px) + Cancel/Commit 버튼 row.
- REQ-CPR-005: 각 swatch `on_mouse_down` → `color_picker_select(palette[i])`. 현재 `selected_color()` 와 동일한 swatch 는 흰색 2px border 강조.
- REQ-CPR-006: `RootView::render()` 에서 `delete_confirmation` 직후 `color_picker_modal.as_ref().map(|m| render_color_picker_overlay(m, cx))` 분기 추가.
- REQ-CPR-007: cargo build / clippy / fmt + ui crate test ALL PASS, 회귀 0.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-CPR-1 | 모달 None 상태 | `commit_color_picker_modal` | 반환 None, store/sidebar 무변동 | unit test |
| AC-CPR-2 | 모달 open(target=ws-a, current=palette[0]), select(palette[5]) | `commit_color_picker_modal` | store ws-a.color == palette[5], sidebar workspaces[i].color == palette[5], modal None, 반환 Some(("ws-a", palette[5])) | unit test (tempfile) |
| AC-CPR-3 | 모달 open(target=ws-a, current=palette[0]) | `cancel_color_picker_modal` | modal None, store ws-a.color 무변동 (palette[0]) | unit test (tempfile) |
| AC-CPR-4 | 모달 open(target=ws-a, current=palette[0]) | `color_picker_select(palette[7])` | modal.selected_color() == palette[7], target 보존 | unit test |
| AC-CPR-5 | 모달 None | `color_picker_select(any)` | no-op (panic X) | unit test |
| AC-CPR-6 | 모달 open, store 에 target_id 부재 | `commit_color_picker_modal` | 모달 dismiss, 반환 None, tracing::warn | unit test (tempfile) |
| AC-CPR-7 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 1344 + 신규 5 = 1349+ tests, 회귀 0 | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | 3 신규 method (commit/cancel/select) + render_color_picker_overlay fn + render() mount + 5 tests |
| `.moai/specs/SPEC-V0-3-0-COLOR-PICKER-RENDER-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-COLOR-PICKER-RENDER-001/progress.md` | created | 구현 후 갱신 |

FROZEN:
- `ColorPickerModal` struct 시그니처 (workspace_menu.rs)
- `WORKSPACE_COLOR_PALETTE` const
- `WorkspacesStore::set_color` API

## 7. Test Strategy

신규 unit tests (ui-crate lib.rs, all in `tests` module):
1. `commit_color_picker_modal_persists_color_and_syncs_sidebar` — open → select → commit → store/sidebar/modal 검증 (tempfile)
2. `commit_color_picker_modal_returns_none_when_modal_closed` — None 모달에서 commit no-op
3. `cancel_color_picker_modal_dismisses_without_change` — open → cancel → store 무변동 (tempfile)
4. `color_picker_select_updates_selected_when_open` + `_no_ops_when_closed` — modal in-progress 선택 갱신
5. `commit_color_picker_modal_handles_missing_target_gracefully` — target 부재 시 graceful

GPUI render fn 자체는 logic-level 테스트 불가 (기존 rename modal render 와 동일 정책). render() body 의 mount path 는 컴파일 검증으로 충분.

기존 1344 ui tests 회귀 0.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 2 #2
Parent: SPEC-V0-3-0-WORKSPACE-COLOR-001 (carry-forward #1)
Carry-to: workspace_menu 우클릭 dropdown ChangeColor GPUI 표시 — 별 SPEC
