# SPEC-V0-3-0-WORKSPACE-COLOR-001 — Workspace Color Tags (palette + picker logic)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-WORKSPACE-COLOR-001 |
| **Title** | Workspace 별 사용자 선택 색상 (palette + ColorPickerModal logic) |
| **Status** | merged (PR #101, 2026-05-04 main 47e1b47) — logic-only closure. GPUI render carry to SPEC-V0-3-0-COLOR-PICKER-RENDER-001 (sess 18 entry). |
| **Priority** | Low (P3 polish) |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V3-004 MS-5/6 (WorkspaceMenu skeleton) |
| **Cycle** | v0.3.0 (audit Top 16 #14 / D-5) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 1 #3 진입. audit feature-audit.md §4 #14 D-5 carry. Lightweight 한도 (≤10KB / ≤8 AC / 1 MS) 충족.

## 1. Purpose

`Workspace.color: u32` 필드는 v0.1.0 부터 존재하지만 모든 workspace 가 `brand::PRIMARY_DARK` 단일 색으로 하드코드되어 사이드바에서 시각적 구별이 불가능하다. 본 SPEC 은 12-preset color palette 와 `WorkspacesStore::set_color` API + `ColorPickerModal` (logic-level, GPUI render carry) 를 도입하고, 신규 workspace 추가 시 round-robin 색상 자동 부여로 UX 를 개선한다. GPUI render 측 modal UI 는 차후 SPEC carry.

## 2. Goals

- 12 preset color (`WORKSPACE_COLOR_PALETTE`) 정의 — 색상 충돌 회피, brand-aligned
- `WorkspacesStore::set_color(id, color)` API — id 로 workspace 찾아 색상 변경 + save
- `WorkspaceMenuAction::ChangeColor` variant — 우클릭 메뉴에서 dispatch 진입점
- `ColorPickerModal` struct (logic only) — open/select/commit/dismiss state machine
- `RootView::color_picker_modal: Option<ColorPickerModal>` slot
- 신규 workspace 추가 시 round-robin 색상 자동 부여 (기존 workspaces 수 % 12)
- TRUST 5 gates ALL PASS

## 3. Non-Goals / Exclusions

- ColorPickerModal 의 GPUI render 측 (overlay UI, swatch grid, hover state) — 차후 SPEC carry
- workspace_menu 우클릭 dropdown 의 ChangeColor 항목 GPUI 표시 — RootView 단 dispatch 만 wire
- color customization (사용자 hex 입력) — 12 preset 만
- color persistence migration (기존 workspaces.json 의 color 필드 무수정 — 새 workspace 만 적용)

## 4. Requirements

- REQ-WC-001: `crates/moai-studio-ui/src/workspace_color.rs` 모듈 신규 + `WORKSPACE_COLOR_PALETTE: [u32; 12]` const 노출.
- REQ-WC-002: 12 preset color 는 brand-aligned (hue 분산, 채도 통일) 으로 시각적 구별 가능.
- REQ-WC-003: `WORKSPACE_COLOR_PALETTE.next_color(existing_count: usize) -> u32` helper — round-robin (count % 12).
- REQ-WC-004: `WorkspacesStore::set_color(&mut self, id: &str, color: u32) -> Result<(), WorkspaceError>` — id 로 workspace 찾아 color 변경 후 save. NotFound 시 Err.
- REQ-WC-005: `WorkspaceMenuAction::ChangeColor` enum variant 추가 (no payload).
- REQ-WC-006: `WorkspaceMenuAction::all()` / `label()` / `is_destructive()` 갱신 (5 variant).
- REQ-WC-007: `ColorPickerModal` struct (workspace_menu module 내) — fields: `target_id: String`, `selected: u32`. methods: `open(target, current) -> Self`, `select(color)`, `selected_color() -> u32`, `target() -> &str`.
- REQ-WC-008: `RootView::color_picker_modal: Option<ColorPickerModal>` 필드 추가, `new()` 에 None 초기화.
- REQ-WC-009: `RootView::next_color_for_new_workspace(&self) -> u32` helper 메서드 — `workspace_color::next_color(self.workspaces.len())` 위임. 호출자(wizard build flow) 가 workspace.color 적용 — wizard 통합은 차후 SPEC carry.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-WC-1 | workspace_color 모듈 | 호출 | `WORKSPACE_COLOR_PALETTE.len() == 12`, 모두 distinct u32 | unit test |
| AC-WC-2 | next_color helper | count = 0, 1, 12, 13, 24 | 각각 palette[0], palette[1], palette[0], palette[1], palette[0] | unit test |
| AC-WC-3 | WorkspacesStore 에 ws-a 존재 | `set_color("ws-a", 0xff0000)` | ws-a.color == 0xff0000, save ✅ | unit test (tempfile) |
| AC-WC-4 | WorkspacesStore 에 ws-a 부재 | `set_color("ws-zzz", any)` | `Err(NotFound("ws-zzz"))` | unit test |
| AC-WC-5 | WorkspaceMenuAction::all() | 호출 | length == 5, 신규 ChangeColor 포함 | unit test |
| AC-WC-6 | WorkspaceMenuAction::ChangeColor.is_destructive() | 호출 | false | unit test |
| AC-WC-7 | ColorPickerModal::open("ws-a", 0xff0000) | 호출 | target() == "ws-a", selected_color() == 0xff0000. select(0x00ff00) → selected_color() == 0x00ff00 | unit test |
| AC-WC-8 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 회귀 0 | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/workspace_color.rs` | created | 신규 module — palette + next_color helper |
| `crates/moai-studio-ui/src/lib.rs` | modified | mod workspace_color, RootView field 추가, handle_add_workspace 갱신, tests |
| `crates/moai-studio-ui/src/workspace_menu.rs` | modified | ChangeColor variant 추가, all/label/is_destructive 갱신, ColorPickerModal struct, tests |
| `crates/moai-studio-workspace/src/lib.rs` | modified | `set_color` API + tests |
| `.moai/specs/SPEC-V0-3-0-WORKSPACE-COLOR-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-WORKSPACE-COLOR-001/progress.md` | created | 구현 후 갱신 |

FROZEN:
- `Workspace` struct 자체 (color 필드 시그니처 보존)
- 기존 `WorkspacesFile` JSON 스키마

## 7. Test Strategy

신규 unit tests:
- workspace_color.rs: 2 tests (palette 검증, next_color round-robin)
- workspace-crate lib.rs: 2 tests (set_color 성공/실패)
- workspace_menu.rs: 3 tests (all/label/is_destructive, ColorPickerModal)
- ui-crate lib.rs: 1 test (handle_add_workspace 가 next_color 적용)

GPUI 미가동 (cx 의존 X). 기존 1320 ui tests 회귀 0.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 1 #3
Carry-to: ColorPickerModal GPUI render 측 (overlay UI, swatch grid) — 차후 SPEC
