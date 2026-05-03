---
id: SPEC-V0-2-0-TOOLBAR-WIRE-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V0-1-2-MENUS-001]
milestones: [MS-1]
language: ko
labels: [v0.2.0, ui, toolbar, audit-top-8, lightweight]
revision: v1.0.0 (lightweight) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-TOOLBAR-WIRE-001: Toolbar 7 button dispatch wire (audit F-3)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 9 (audit Top 8 #5 ⭐⭐⭐⭐). 기존 toolbar.rs (132 LOC, V0-1-2-MENUS-001 F-3 scaffold) 의 7 button 이 `on_action` listener 만 가지고 있고 실 click→dispatch wire 가 빠져 있는 결함 해소. `cx.dispatch_action(&Action)` 패턴으로 RootView 의 기존 7 action handler 와 wire. Lightweight SPEC fast-track 적용. |

---

## 1. 목적

`crates/moai-studio-ui/src/toolbar.rs` 의 7 button (NewWorkspace / ToggleSidebar / OpenSettings / OpenCommandPalette / NewTerminalSurface / ToggleFind / OpenDocumentation) 이 사용자 클릭 시 실제로 dispatch 되도록 wire 한다. 현재 각 button 은 `on_action` listener 를 가지지만 — 이는 GPUI Action 이 dispatch 된 *후* 호출되는 callback 이지 click handler 가 아니다. 따라서 사용자가 toolbar button 을 클릭해도 아무 일도 일어나지 않는다.

본 SPEC 은 각 button 에 `on_mouse_down(MouseButton::Left, cx.listener(... cx.dispatch_action(&Action)))` 를 추가해 click → dispatch chain 을 완성한다. RootView 의 기존 7 action handler 가 이미 존재하므로 (lib.rs §1973~2044) 본 SPEC scope 는 toolbar 측 wire 만.

audit feature-audit.md §3 Tier F line 222: "F-3 Toolbar 7 primary actions wire — toolbar.rs 4798 bytes 토대만, 실 button wire 미완. F-1 dispatch 와 통합 가능." 의 정식 해소.

**Lightweight SPEC fast-track** 적격성:
- spec.md ≤ 10 KB ✅
- AC 6 (≤ 8) ✅
- milestones 1 (≤ 2) ✅
- no architectural impact (toolbar.rs 단일 파일 + GPUI dispatch_action API 기존 사용) ✅
- 단일 PR (~150 LOC) ✅

---

## 2. 목표 (Goals)

- G1. 7 toolbar button 이 mouse_down(Left) 이벤트 시 해당 GPUI Action 을 `App::dispatch_action(&dyn Action)` 으로 dispatch 한다.
- G2. RootView 의 기존 7 on_action handler (NewWorkspace / OpenSettings / ToggleSidebar / ToggleFind / NewTerminalSurface / OpenCommandPalette + App-level OpenDocumentation) 가 그대로 trigger 되어 동작한다.
- G3. 기존 button label / 스타일 / id / on_action listener 는 무변경 (R3 새 mouse_down listener 만 추가).
- G4. F-3 audit 항목 PARTIAL → DONE.

---

## 3. Non-Goals / Exclusions

- N1. **F-4 Status Bar 실 state binding** — git2 / LSP / agent runtime 의존성, 별 SPEC.
- N2. **Toolbar button icon 추가** — 현재 텍스트 라벨 그대로.
- N3. **Toolbar 항목 추가/제거** — 7 항목 그대로.
- N4. **Sidebar visible 실 toggle** — V0-1-2-MENUS-001 의 ToggleSidebar handler 가 "deferred" 로 로깅만. 본 SPEC 은 dispatch 까지만.
- N5. **Toolbar button keyboard navigation** — Arrow / Enter / Tab — 별 PR.
- N6. **Toolbar 우클릭 컨텍스트 메뉴** — 별 SPEC.

---

## 4. Requirements (EARS)

- **REQ-TW-001**: Toolbar 의 NewWorkspace button 이 mouse_down(Left) 시 `App::dispatch_action(&NewWorkspace)` 를 호출한다.
- **REQ-TW-002**: Toolbar 의 ToggleSidebar button 이 mouse_down(Left) 시 `App::dispatch_action(&ToggleSidebar)` 를 호출한다.
- **REQ-TW-003**: Toolbar 의 OpenSettings button 이 mouse_down(Left) 시 `App::dispatch_action(&OpenSettings)` 를 호출한다.
- **REQ-TW-004**: Toolbar 의 OpenCommandPalette button 이 mouse_down(Left) 시 `App::dispatch_action(&OpenCommandPalette)` 를 호출한다.
- **REQ-TW-005**: Toolbar 의 NewTerminalSurface button 이 mouse_down(Left) 시 `App::dispatch_action(&NewTerminalSurface)` 를 호출한다.
- **REQ-TW-006**: Toolbar 의 ToggleFind button 이 mouse_down(Left) 시 `App::dispatch_action(&ToggleFind)` 를 호출한다.
- **REQ-TW-007**: Toolbar 의 OpenDocumentation button 이 mouse_down(Left) 시 `App::dispatch_action(&OpenDocumentation)` 를 호출한다.
- **REQ-TW-008**: 기존 button id / label / 스타일 / on_action listener 는 본 SPEC 변경 후에도 동일하다 (R3 새 listener 만 추가).

---

## 5. Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-TW-1 | REQ-TW-001 ~ REQ-TW-007 | 본 SPEC 적용 후 toolbar.rs | 컴파일 | 7 button 모두 `.on_mouse_down(MouseButton::Left, cx.listener(...))` 호출을 보유, 각 listener 가 적절한 action 으로 `dispatch_action` 호출 | 단위 테스트 (toolbar 구조 검사) + cargo check |
| AC-TW-2 | REQ-TW-008 | 본 SPEC 적용 전/후 toolbar.rs | grep | 기존 button id 7개 (`toolbar-new-workspace`, `toolbar-toggle-sidebar`, `toolbar-settings`, `toolbar-command-palette`, `toolbar-new-terminal`, `toolbar-find`, `toolbar-documentation`) 모두 보존, 기존 `on_action` listener 7개 모두 보존 | 정적 검사 (테스트 + grep) |
| AC-TW-3 | REQ-TW-001 ~ REQ-TW-007 | 사용자가 `cargo run -p moai-studio-app` 실행 후 toolbar 의 NewWorkspace button 클릭 | RootView 의 NewWorkspace handler 호출 여부 | `handle_add_workspace(cx)` 가 호출되어 새 workspace 가 sidebar 에 추가됨 (수동 smoke) | 수동 smoke test (CI 비대상) |
| AC-TW-4 | G3 / REQ-TW-008 | 본 SPEC 적용 전/후 | `cargo test -p moai-studio-ui --lib` | 기존 ui 1269 tests 회귀 0 | CI gate |
| AC-TW-5 | G3 | Toolbar 의 sidebar_visible 상태 | mouse_down listener 호출 후 | sidebar_visible 필드 무변경 (button label 만 ToggleSidebar action 결과에 의해 RootView 에서 update 됨) | 단위 테스트 |
| AC-TW-6 | clippy + fmt | 본 SPEC 적용 후 | `cargo clippy / fmt --check` | 0 warning, clean | CI gate |

---

## 6. File Layout

### 6.1 수정

- `crates/moai-studio-ui/src/toolbar.rs`:
  - 각 7 button 의 child chain 에 `.on_mouse_down(MouseButton::Left, cx.listener(...))` 추가.
  - listener body: `cx.dispatch_action(&ActionType)` (NewWorkspace / ToggleSidebar / OpenSettings / OpenCommandPalette / NewTerminalSurface / ToggleFind / OpenDocumentation).
  - 기존 `on_action` listener 보존 (현재 빈 클로저 — 이는 RootView 가 dispatch 시 수신하는 callback. toolbar 측 listener 는 사실상 no-op 이지만 backward compat 를 위해 보존).
  - `MouseButton` import 추가.
  - 단위 테스트 ~5-7개 추가 (toolbar 구조 + sidebar_visible 보존 + dispatch 호출 시 panic 없음).

### 6.2 변경 금지 (FROZEN)

- `crates/moai-studio-ui/src/lib.rs` (RootView on_action 7 handlers 무변경 — 그대로 활용).
- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-workspace/**` 전체.
- 기존 button id / label / 스타일 (R3 새 listener 만 추가).

---

## 7. Test Strategy

- 신규 테스트 minimum **5개** (toolbar.rs 의 #[cfg(test)] mod tests):
  - `Toolbar::new(false)` / `Toolbar::new(true)` 생성 + sidebar_visible 보존
  - `Toolbar::set_sidebar_visible(true/false)` 동작
  - `Render` 호출 시 panic 없음 (TestAppContext 기반)
  - sidebar_visible=true → label "Hide Sidebar" / false → "Show Sidebar"
  - 7 button id 가 element tree 에 모두 존재 (logic-level 또는 cargo check 로 충분)
- AC-TW-3 (실 click → handler 호출) 은 GPUI integration test 가 어렵기 때문에 **수동 smoke test** 로 검증 (CI 비대상). progress.md 에 명시.
- 회귀: 기존 `cargo test -p moai-studio-ui --lib` 1269 tests 모두 GREEN 유지.

---

## 8. DoD

`cargo run -p moai-studio-app` → 사용자가 toolbar 의 7 button 중 하나를 클릭 → 해당 action 이 RootView 까지 전파되어 동작 (NewWorkspace 추가, Settings 열림, Command Palette 띄움 등). cargo test/clippy/fmt 모두 GREEN.

audit Top 8 #5 F-3 진척: PARTIAL → DONE (button wire 완성). F-4 (Status Bar real binding) 는 별 SPEC carry.

---

Version: 1.0.0 (lightweight) | Source: SPEC-V0-2-0-TOOLBAR-WIRE-001 | 2026-05-04
