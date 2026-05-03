---
id: SPEC-V3-004
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-003]
milestones: [MS-1, MS-2, MS-3, MS-4, MS-5, MS-6]
language: ko
labels: [phase-3, ui, gpui, render, escape-hatch]
revision: v1.0.0 (initial draft, SPEC-V3-003 carry-over AC-P-4/AC-P-5 승계)
---

# SPEC-V3-004: Render Layer Integration — TabContainer Entity + PaneTree GPUI rendering + divider drag e2e

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. SPEC-V3-003 종결 시점에 분리된 render layer escape hatch SPEC. AC-P-4 (TabContainer ↔ divider render integration) 와 AC-P-5 (gpui test-support feature 채택 재평가) 가 본 SPEC 의 AC-R-5 / AC-R-6 으로 명시 승계. RootView::pane_splitter → tab_container 필드 교체, impl Render for TabContainer 신규, render_pane_tree 재귀 변환, GPUI 키 dispatch 배선의 4 축으로 구성. |

---

## 1. 개요

### 1.1 목적

SPEC-V3-003 종결 시점에 logic-only 로 남은 `TabContainer` / `PaneTree` / `GpuiDivider` / `dispatch_tab_key` 를 GPUI 0.2.2 위에서 실제 화면에 렌더하고 사용자 입력에 반응하도록 배선한다. `crates/moai-studio-ui/src/lib.rs` 의 `RootView::pane_splitter: Option<Entity<TerminalSurface>>` 임시 필드를 `tab_container: Option<Entity<TabContainer>>` 로 교체하여, 단일 터미널만 렌더하던 RootView 가 다중 탭 + 다중 pane + divider 가 보이는 진짜 v3 셸이 되도록 한다.

본 SPEC 은 SPEC-V3-003 의 **escape hatch** 다 — pane/tab logic 은 이미 53 unit + 2 integration tests 로 검증되어 있고, 본 SPEC 은 그 검증된 logic 을 GPUI 가 화면에 그리도록 통합하는 마지막 단계다.

### 1.2 SPEC-V3-003 carry-over 와의 관계

SPEC-V3-003 contract.md §11.6 escalation protocol 은 두 carry-over 를 명시했다:

- **AC-P-4 (MS-2 carry-over → MS-3 결정)**: "TabContainer 가 GPUI render 시 divider 가 실제 layout 에 포함되는지 integration 검증. **render layer 도입 어려우면 logic-level assertion 으로 대체.**" — MS-3 종결 시 logic-level alternative 로 잠정 종결되었으나, 사용자 결정에 따라 별도 SPEC 으로 분리하여 render layer 직접 검증.
- **AC-P-5 (MS-1 → MS-3 carry-over → DEFERRED)**: "gpui test-support 재평가 — 채택 시 headless resize unit." — MS-3 시점 DEFER 결정. 본 SPEC 진입 시 USER-DECISION 게이트로 재평가.

본 SPEC 의 AC-R-5 / AC-R-6 가 각각 위 두 carry-over 의 해소 책임을 진다.

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-004/research.md` — 코드베이스 분석, GPUI Render trait 패턴, AC carry-over 정의, 위험 요약.
- `.moai/specs/SPEC-V3-003/spec.md` §9.2 — `pane_splitter` → `tab_container` 필드 rename 의 사전 정의.
- `.moai/specs/SPEC-V3-003/contract.md` §11.6, §11.7 — carry-over 결정 원본.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 원칙 (RG-P-7 carry).
- `crates/moai-studio-ui/src/terminal/mod.rs:262-306` — 기존 `impl Render for TerminalSurface` reference.
- `crates/moai-studio-ui/src/lib.rs:170-202` — 기존 `Render for RootView` 패턴.

---

## 2. 배경 및 동기

본 섹션의 상세 분석은 `.moai/specs/SPEC-V3-004/research.md` §1 ~ §6 참조. SPEC 독자가 요구사항 진입 전에 알아야 할 최소 맥락만 요약한다.

- **Logic-render 격차** (research §1.2): SPEC-V3-003 가 `PaneTree`, `TabContainer`, `TabBar`, `GpuiDivider`, `dispatch_tab_key` 를 모두 logic-only 로 완성했다. 이들은 `Render` trait 을 구현하지 않으며, GPUI Entity 도 아니다. RootView 와의 배선이 missing link 다.
- **AC-P-4 의 공식 승계** (research §1.1): SPEC-V3-003 종결 시 AC-P-4 는 "render-layer 작업이 별도 SPEC 으로 이관된다" 는 정책으로 deferred-closed 되었다. 본 SPEC 의 AC-R-5 가 그 책임을 이어받는다.
- **escape hatch 정의** (research §1.3): 본 SPEC 이 PASS 한 시점에 사용자가 `cargo run -p moai-studio-app` 으로 (1) 첫 탭 자동 표시, (2) Cmd/Ctrl+T 새 탭 생성 가시, (3) Cmd/Ctrl+1~9 탭 전환 가시, (4) Cmd/Ctrl+\\ 분할 가시, (5) divider drag 으로 비율 변경 + boundary rejection 가시 — 5 가지를 직접 관찰할 수 있어야 한다.
- **GPUI 0.2.2 의 안정성** (research §2.4): Zed 가 main 브랜치에서 Render API 를 변경 중이지만 0.2.2 는 SPEC-V3-001 부터 pin 된 상태. 본 SPEC 은 pin 변경 없이 진행한다.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `TabContainer` 가 GPUI `Entity` 로 생성 가능하며 `impl Render for TabContainer` 가 활성 탭의 `PaneTree` + 탭 바를 그린다.
- G2. `PaneTree<L>` 가 GPUI HStack/VStack 자식 트리로 변환되며 split 노드마다 정확히 1 개의 divider element 가 등장한다.
- G3. 사용자가 마우스로 divider 를 drag 할 때 GPUI 이벤트가 `GpuiDivider::on_drag` → `PaneTree::set_ratio` 로 전달되어 화면 비율이 바뀐다.
- G4. 사용자가 입력한 키스트로크가 `dispatch_tab_key` 를 거쳐 `TabContainer::new_tab/switch_tab/close_tab` 또는 `PaneTree::split_*` 으로 전파된다.
- G5. SPEC-V3-003 AC-P-4 (TabContainer ↔ divider render integration) 가 render-layer 검증으로 해소된다.
- G6. SPEC-V3-002 / SPEC-V3-003 의 logic 공개 API 는 변경하지 않는다 (RG-R-6 carry).
- G7. macOS 14+ / Ubuntu 22.04+ 양쪽에서 동일한 render 동작을 보장한다.

### 3.2 비목표 (Non-Goals)

- N1. **실제 PTY spawn per pane** — 본 SPEC 의 leaf payload 는 placeholder 또는 기존 단일 `TerminalSurface` 재사용. Per-pane PtyWorker spawn 은 별도 SPEC.
- N2. **Tab close UI element** (× 버튼) — 키 바인딩만 본 SPEC, 마우스 close 는 별도 SPEC.
- N3. **Tab reordering / detach / rename** — SPEC-V3-003 §3.2 의 N6/N7/N9 와 동일 입장.
- N4. **Persistence 변경** — SPEC-V3-003 MS-3 산출 (panes-{ws-id}.json) 그대로 유지. 본 SPEC 은 read 만, 새 schema 도입 없음.
- N5. **Pane zoom / named layout / broadcast** — SPEC-V3-003 N1/N2/N8 그대로.
- N6. **Windows 빌드** — SPEC-V3-002/003 N10 와 동일.
- N7. **새 design token 추가** — SPEC-V3-003 의 `toolbar.tab.active.background` 등 기존 토큰 재사용.

---

## 4. 사용자 스토리

- **US-R1**: 개발자가 앱을 실행하면 자동으로 첫 탭이 열려 있고 본문에 단일 placeholder pane 이 보인다 → RootView 가 `Entity<TabContainer>` 를 보유하며 활성 탭의 단일 leaf 가 렌더된다.
- **US-R2**: 개발자가 Cmd+T (macOS) / Ctrl+T (Linux) 를 누르면 탭 바에 새 탭이 추가되어 즉시 가시된다 → keystroke → `dispatch_tab_key` → `TabContainer::new_tab` → `cx.notify()` → re-render.
- **US-R3**: 개발자가 Cmd/Ctrl+\\ 를 누르면 활성 leaf 가 좌우 분할되어 두 placeholder pane 과 그 사이의 수직 divider 가 화면에 등장한다 → keystroke → `TabCommand::SplitVertical` → `PaneTree::split_horizontal(active_leaf)` (※ "vertical split"=수직 divider=Horizontal direction enum, SPEC-V3-003 §15 용어 정의 그대로 상속).
- **US-R4**: 개발자가 마우스로 divider 를 드래그하면 양쪽 pane 의 비율이 바뀌고, 한쪽이 너무 작아지려 하면 ratio 가 clamp 되어 더 이상 줄어들지 않는다 → mouse drag → `GpuiDivider::on_drag(delta_px, total_px)` → `PaneTree::set_ratio` → boundary rejection 시 ratio 유지 + tracing warn.
- **US-R5**: 개발자가 Cmd/Ctrl+1~9 로 탭을 전환하면 본문 영역이 해당 탭의 pane tree 로 즉시 교체된다 → `dispatch_tab_key` → `TabContainer::switch_tab` → render 재호출 → 새 탭의 PaneTree 렌더.

---

## 5. 기능 요구사항 (EARS)

### RG-R-1 — TabContainer Entity & Render

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-001 | Ubiquitous | 시스템은 `TabContainer` 가 GPUI `cx.new(\|cx\| TabContainer::new())` 호출로 `Entity<TabContainer>` 로 생성될 수 있도록 한다. | The system **shall** allow `TabContainer` to be instantiated as `Entity<TabContainer>` via `cx.new`. |
| REQ-R-002 | Ubiquitous | 시스템은 `TabContainer` 에 대해 `impl Render` 트레잇 구현을 제공한다. 해당 구현은 (a) 탭 바와 (b) 활성 탭의 `PaneTree` 렌더 결과를 세로로 쌓아 `IntoElement` 로 반환한다. | The system **shall** implement `Render for TabContainer` returning a vertical stack of (a) tab bar and (b) the active tab's `PaneTree` rendering. |
| REQ-R-003 | Event-Driven | `TabContainer::new_tab`, `switch_tab`, `close_tab` 중 하나가 호출되어 내부 상태가 변경되면, 시스템은 GPUI re-render 를 트리거하기 위해 `cx.notify()` 가 호출되도록 한다. | When `new_tab`/`switch_tab`/`close_tab` mutates state, the system **shall** ensure `cx.notify()` is invoked to schedule re-render. |
| REQ-R-004 | State-Driven | `TabContainer.tabs.len() >= 2` 이고 `active_tab_idx` 가 유효한 동안, 시스템은 활성 탭의 `last_focused_pane` 이 가리키는 leaf 를 보이는 영역의 일부로 렌더한다. | While `tabs.len() >= 2` and `active_tab_idx` is valid, the system **shall** render the leaf identified by the active tab's `last_focused_pane` as part of the visible area. |
| REQ-R-005 | Unwanted | 시스템은 `TabContainer.tabs.is_empty()` 가 `true` 인 상태에서 `Render::render` 가 panic 하지 않도록 한다. 빈 상태에서는 SPEC-V3-001 의 Empty State CTA 가 그대로 표시된다. | The system **shall not** panic in `Render::render` when `tabs.is_empty()`; it must fall through to SPEC-V3-001 Empty State CTA. |

### RG-R-2 — PaneTree → GPUI layout 변환

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-010 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/panes/render.rs` (신규) 에 재귀 함수 `render_pane_tree<L>(tree: &PaneTree<L>, ctx: ...) -> impl IntoElement` 를 제공한다. `L: Render + 'static` 제약을 만족해야 한다. | The system **shall** provide a recursive `render_pane_tree<L: Render + 'static>` function in `panes/render.rs`. |
| REQ-R-011 | Event-Driven | `PaneTree::Leaf(leaf)` 노드가 변환 대상일 때, 시스템은 leaf payload 의 GPUI `Entity` 를 자식 element 로 그대로 마운트한다. | When the node is `Leaf`, the system **shall** mount the leaf payload's GPUI `Entity` as a child element. |
| REQ-R-012 | Event-Driven | `PaneTree::Split { direction: Horizontal, ratio, first, second }` 가 변환 대상일 때, 시스템은 `flex_row` 컨테이너 안에 `first` (좌) / 수직 divider / `second` (우) 를 자식으로 배치한다. ratio 는 양 자식의 flex 또는 width 에 비례 반영된다. | When the node is `Split { Horizontal }`, the system **shall** lay out `first` (left), a vertical divider, and `second` (right) inside a `flex_row` container with width proportional to `ratio`. |
| REQ-R-013 | Event-Driven | `PaneTree::Split { direction: Vertical, ... }` 가 변환 대상일 때, 시스템은 `flex_col` 컨테이너 안에 `first` (상) / 수평 divider / `second` (하) 를 자식으로 배치한다. ratio 는 양 자식의 height 에 비례 반영된다. | When the node is `Split { Vertical }`, the system **shall** lay out `first` (top), a horizontal divider, and `second` (bottom) inside `flex_col` with height proportional to `ratio`. |
| REQ-R-014 | Ubiquitous | 시스템은 모든 `Split` 노드 당 정확히 1 개의 divider element 를 생성한다. divider element 는 SPEC-V3-003 의 `GpuiDivider::orientation` 과 일치하는 방향(수직/수평)을 가진다. | The system **shall** emit exactly one divider element per `Split` node, with orientation matching `GpuiDivider::orientation`. |
| REQ-R-015 | State-Driven | 윈도우 가용 영역이 한 leaf 의 최소 크기 (40 cols × 10 rows) 미만이 되는 동안, 시스템은 `PaneTree` 구조를 유지하면서 가장 깊은 leaf 부터 시각적으로 hidden(`display: none` 등 GPUI 동등물) 처리한다. | While the available area is below `MIN_COLS × MIN_ROWS` for a leaf, the system **shall** preserve the `PaneTree` structure but visually hide the deepest leaf first. |

### RG-R-3 — Divider drag GPUI 이벤트 → PaneTree.set_ratio

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-020 | Event-Driven | 사용자가 divider element 위에서 마우스 좌클릭 후 drag 를 시작하면, 시스템은 drag 시작 좌표를 저장하고 cursor 를 `ew-resize` (수직 divider) 또는 `ns-resize` (수평 divider) 로 변경한다. | When the user mouse-down + drag-starts on a divider, the system **shall** record the start coordinate and change cursor to `ew-resize`/`ns-resize`. |
| REQ-R-021 | Event-Driven | drag 중 마우스 이동 시, 시스템은 `delta_px = current - start` 와 split 의 `total_px` 를 계산하여 `GpuiDivider::on_drag(delta_px, total_px)` 를 호출하고 결과 ratio 를 `PaneTree::set_ratio(node_id, new_ratio)` 로 전파한다. | During drag, the system **shall** compute `delta_px` and call `GpuiDivider::on_drag` then `PaneTree::set_ratio` with the result. |
| REQ-R-022 | Unwanted | 시스템은 한 sibling 이 `MIN_COLS` (Horizontal) 또는 `MIN_ROWS` (Vertical) 미만이 되도록 ratio 를 변경하지 않는다. 그러한 시도는 clamp 된 값으로 이어지고 `tracing::debug!` 로그 1 건이 기록된다. | The system **shall not** allow ratio changes that would make a sibling smaller than `MIN_COLS`/`MIN_ROWS`; such attempts must be clamped with a `tracing::debug!` log entry. |
| REQ-R-023 | Event-Driven | 마우스 좌클릭이 해제되면, 시스템은 drag 상태를 종료하고 cursor 를 기본값으로 복원하며 `cx.notify()` 호출로 최종 비율 변경을 GPUI 에 반영한다. | When mouse-up occurs, the system **shall** end drag state, restore cursor, and call `cx.notify()`. |
| REQ-R-024 | State-Driven | drag 가 진행 중인 동안, 시스템은 다른 키 이벤트와 마우스 클릭 (탭 바, leaf 본문) 을 일시적으로 무시한다. drag 종료 후 정상 dispatch 가 재개된다. | While drag is active, the system **shall** suppress other key events and mouse clicks; normal dispatch resumes after drag ends. |

### RG-R-4 — 앱 레벨 키 dispatch → TabContainer

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-030 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/tabs/keys.rs` 에 `keystroke_to_tab_key(ks: &gpui::Keystroke) -> Option<(KeyModifiers, TabKeyCode)>` 변환 함수를 제공한다. | The system **shall** provide `keystroke_to_tab_key` in `tabs/keys.rs`. |
| REQ-R-031 | Event-Driven | `RootView` 가 GPUI `Window::on_key_down` 핸들러로 키 이벤트를 수신하면, 시스템은 `keystroke_to_tab_key` → `dispatch_tab_key` 순서로 변환하여 `Some(TabCommand)` 일 때만 활성 `TabContainer` entity 의 `update` 메서드를 호출한다. | When `RootView` receives a key event, the system **shall** convert via `keystroke_to_tab_key` → `dispatch_tab_key` and only call `tab_container.update(...)` when `Some(TabCommand)` is returned. |
| REQ-R-032 | Event-Driven | `TabCommand::NewTab` 수신 시, 시스템은 `TabContainer::new_tab(None)` 호출 후 `cx.notify()` 를 발화한다. | When `TabCommand::NewTab` arrives, the system **shall** call `new_tab(None)` and `cx.notify()`. |
| REQ-R-033 | Event-Driven | `TabCommand::SwitchToTab(idx)` 수신 시, 시스템은 `TabContainer::switch_tab(idx)` 를 호출한다. `Err(IndexOutOfBounds)` 는 무시하고 로그하지 않는다 (사용자 의도 추측 금지, AC-P-25 호환). | When `TabCommand::SwitchToTab(idx)` arrives, the system **shall** call `switch_tab(idx)` and silently ignore `IndexOutOfBounds`. |
| REQ-R-034 | Event-Driven | `TabCommand::SplitVertical` 수신 시, 시스템은 활성 탭의 `pane_tree` 의 현재 `last_focused_pane` leaf 를 대상으로 `split_horizontal` (좌우 분할) 을 호출한다. `TabCommand::SplitHorizontal` 도 동일 패턴으로 `split_vertical` (상하 분할) 호출. | When `SplitVertical`/`SplitHorizontal` arrives, the system **shall** call `split_horizontal`/`split_vertical` on the active tab's focused leaf. |
| REQ-R-035 | Unwanted | 시스템은 `TabCommand::Other` 또는 `dispatch_tab_key` 가 `None` 을 반환한 키 이벤트를 RootView 레벨에서 소비하지 않는다. 해당 이벤트는 활성 leaf 의 `TerminalSurface::handle_key_down` 에 전달된다 (REQ-P-033 carry). | The system **shall not** consume keystrokes returning `None`; they must be forwarded to the focused `TerminalSurface::handle_key_down`. |

### RG-R-5 — AC-P-4 carry-over 해소 (TabContainer divider render 통합)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-040 | Ubiquitous | 시스템은 `TabContainer` render 결과 element tree 가 활성 탭 `PaneTree` 의 split 노드마다 정확히 1 개의 divider element 를 포함하도록 한다 (REQ-R-014 의 RootView 통합 시점 보증). | The system **shall** ensure the rendered element tree contains exactly one divider per split node in the active tab's `PaneTree`. |
| REQ-R-041 | Event-Driven | 사용자가 divider drag 를 시도하면, 시스템은 SPEC-V3-003 RG-P-2 (REQ-P-011, REQ-P-012) 의 boundary rejection 동작을 RootView 통합 환경에서 그대로 보존한다. | When the user attempts to drag a divider, the system **shall** preserve SPEC-V3-003 RG-P-2 boundary rejection behavior in the integrated RootView context. |
| REQ-R-042 | State-Driven | 활성 탭이 split 된 PaneTree 를 가진 동안, 시스템은 divider element 의 hover/active 시각 상태 (cursor 변경, 색상 강조) 를 SPEC-V3-003 의 `panes::divider` 모듈 정책과 일치하도록 표현한다. | While the active tab has a split tree, the system **shall** render divider hover/active visuals consistent with `panes::divider`. |

### RG-R-6 — Terminal Core 무변경 (RG-P-7 carry)

| REQ ID | 패턴 | 요구사항 (한국어) |
|--------|------|-------------------|
| REQ-R-060 | Ubiquitous | 시스템은 `crates/moai-studio-terminal/**` 의 어떤 파일도 수정하지 않는다. SPEC-V3-002 의 13 tests 가 본 SPEC 모든 milestone 에서 regression 0 으로 유지된다. |
| REQ-R-061 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}` 의 공개 API 를 변경하지 않는다. 내부 helper 추가는 허용한다. |
| REQ-R-062 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` 의 공개 API 를 변경하지 않는다. `impl Render for TabContainer` 추가는 새 구현체이므로 공개 API 추가에 해당하나 기존 메서드 시그니처는 보존한다. |
| REQ-R-063 | Ubiquitous | 시스템은 `crates/moai-studio-workspace/src/persistence.rs` 의 SPEC-V3-003 MS-3 산출 schema (`moai-studio/panes-v1`) 를 변경하지 않는다. 본 SPEC 은 read 만 허용. |

---

## 6. 비기능 요구사항

### 6.1 성능

- NFR-R-1. 초기 윈도우 표시 시 첫 frame paint ≤ 200 ms (SPEC-V3-002 NFR carry, 동일 기준).
- NFR-R-2. divider drag 응답성: 마우스 이동 → 새 ratio 반영 → 화면 재그리기 까지 ≤ 16 ms (60 fps 단일 frame 예산). criterion bench 는 본 SPEC 범위 밖 (logic-only 인 SPEC-V3-003 AC-P-19 가 이미 50 cycle 평균 50ms 이하 보증).
- NFR-R-3. 9 leaf 까지 split 한 후 탭 전환 시 새 탭 첫 frame ≤ 50 ms (SPEC-V3-003 AC-P-19 carry).

### 6.2 안정성

- NFR-R-4. `Render::render` 는 어떤 RootView 상태에서도 panic 하지 않는다 (REQ-R-005 + 비어 있는 PaneTree 방어).
- NFR-R-5. `cargo run -p moai-studio-app` 5 분 idle 후 메모리 사용량 증가 ≤ 5 MB (메모리 누수 방지).

### 6.3 접근성

- NFR-R-6. divider element 는 키보드 접근 가능해야 한다 (Tab 으로 focus, 화살표 키로 ratio 조정). 본 SPEC v1.0.0 에서는 mouse 만 필수, 키보드 ratio 조정은 best-effort.
- NFR-R-7. 탭 바 element 는 GPUI 의 `tab` role 또는 등가 의미 구조를 사용한다 (SPEC-V3-003 §6.3 carry).

### 6.4 호환성

- NFR-R-8. macOS 14 + Ubuntu 22.04 양쪽에서 동일한 RootView render 결과 (탭 수, divider 수, 키 dispatch 결과) 를 보장한다.

---

## 7. 아키텍처

### 7.1 RootView 의 새 데이터 흐름

```
┌─────────────────────────────────────────────────────────────────┐
│  RootView (Entity<RootView>)                                    │
│  ├── workspaces: Vec<Workspace>                                 │
│  ├── active_id: Option<String>                                  │
│  ├── tab_container: Option<Entity<TabContainer>>   ← 신규 필드  │
│  └── (legacy) pane_splitter: REMOVED                            │
│                                                                  │
│  Render::render:                                                 │
│    ├── title_bar(self.title_label())                            │
│    ├── main_body(...)                                           │
│    │   ├── sidebar(...)                                         │
│    │   └── content_area(...)                                    │
│    │       ├── if let Some(tc) = &self.tab_container:           │
│    │       │     tc.clone().into_element()  ← TabContainer 렌더 │
│    │       └── else: Empty State CTA                            │
│    └── status_bar()                                             │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼ tc.update(cx, |tc, cx| ...)
┌─────────────────────────────────────────────────────────────────┐
│  TabContainer (Entity<TabContainer>)                            │
│  ├── tabs: Vec<Tab>                                             │
│  ├── active_tab_idx: usize                                      │
│  └── (Tab 내부) pane_tree: PaneTree<Entity<TerminalSurface>>    │
│                                                                  │
│  Render::render:                                                 │
│    ├── tab_bar_element(self.tabs, self.active_tab_idx)          │
│    └── render_pane_tree(&self.active_tab().pane_tree, ...)      │
│                                                                  │
│  ※ 본 SPEC 의 leaf payload 타입은 placeholder String (logic     │
│    호환) 또는 Entity<TerminalSurface> 중 일부 — MS-2 결정.       │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼ render_pane_tree (재귀)
┌─────────────────────────────────────────────────────────────────┐
│  panes::render::render_pane_tree<L: Render + 'static>           │
│    Leaf(L)             → leaf.into_element()                    │
│    Split{Horizontal}   → flex_row(first / divider_v / second)   │
│    Split{Vertical}     → flex_col(first / divider_h / second)   │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 키 이벤트 dispatch 경로

```
GPUI Window::on_key_down(Keystroke)
   │
   ▼ keystroke_to_tab_key(ks)        ← tabs/keys.rs (신규)
   │
   ▼ Some((mods, code))
   │
   ▼ dispatch_tab_key(mods, code)    ← tabs/keys.rs (기존, 무변경)
   │
   ▼ Some(TabCommand::*)
   │
   ▼ self.tab_container.update(cx, |tc, cx| { match cmd { ... } })
       │
       ├── NewTab          → tc.new_tab(None)
       ├── SwitchToTab(i)  → tc.switch_tab(i).ok()
       ├── SplitVertical   → tc.active_tab_mut().pane_tree.split_horizontal(focused)
       ├── SplitHorizontal → tc.active_tab_mut().pane_tree.split_vertical(focused)
       ├── PrevTab         → tc.switch_tab(idx - 1)
       └── NextTab         → tc.switch_tab(idx + 1)
       ▼
   cx.notify() → Render 재호출
```

`None` 반환 (REQ-R-035) 시 RootView 는 keystroke 를 소비하지 않고 활성 leaf entity 로 forward.

### 7.3 Divider drag 이벤트 경로

```
GPUI Stateful<Div>::on_mouse_down(MouseButton::Left)
   │
   ▼ self.drag_state = Some(DragState { node_id, start_xy, total_px })
   │
   ▼ on_mouse_move
   │
   ▼ delta_px = (current - start_xy).component(orientation)
   │
   ▼ GpuiDivider::on_drag(delta_px, total_px)  → new_ratio
   │
   ▼ self.tab_container.update(cx, |tc, cx| {
       tc.active_tab_mut().pane_tree.set_ratio(node_id, new_ratio).ok();
       cx.notify();
     })
   │
   ▼ on_mouse_up → self.drag_state = None
```

---

## 8. Milestone

본 SPEC 은 3 milestone 으로 분할한다. milestone 간 regression gate 는 SPEC-V3-003 정책 carry.

### MS-1: TabContainer Entity (REQ-R-001 ~ REQ-R-005, RG-R-1)

- **범위**: `impl Render for TabContainer` 신규 구현, `RootView::pane_splitter` → `tab_container: Option<Entity<TabContainer>>` 필드 교체, 빈 PaneTree (단일 leaf placeholder) 의 가시 렌더.
- **포함 요구사항**: REQ-R-001 ~ REQ-R-005, REQ-R-061, REQ-R-062.
- **시연 가능 상태**: `cargo run -p moai-studio-app` 실행 시 단일 placeholder pane (탭 바 1 개 탭 + 본문 1 leaf) 이 가시. 키 이벤트 처리 없음.

### MS-2: PaneTree render + key dispatch (REQ-R-010 ~ REQ-R-015, REQ-R-030 ~ REQ-R-035)

- **범위**: `panes::render::render_pane_tree` 신규, `tabs::keys::keystroke_to_tab_key` 신규, RootView `Window::on_key_down` 등록.
- **포함 요구사항**: RG-R-2, RG-R-4 전체.
- **시연 가능 상태**: Cmd/Ctrl+T 로 새 탭 가시, Cmd/Ctrl+1~9 로 탭 전환 가시, Cmd/Ctrl+\\ 와 Cmd/Ctrl+Shift+\\ 로 분할 가시 (divider 시각 등장). divider drag 은 MS-3.

### MS-3: Divider drag e2e + AC-R-5 carry-over 해소 (REQ-R-020 ~ REQ-R-024, REQ-R-040 ~ REQ-R-042)

- **범위**: divider element 의 mouse drag 핸들러 배선, `set_ratio` 호출, boundary rejection 통합 검증, SPEC-V3-003 AC-P-4 의 render-layer 해소.
- **포함 요구사항**: RG-R-3, RG-R-5 전체.
- **시연 가능 상태**: 사용자가 divider 를 드래그하여 비율을 바꾸고, 한쪽이 너무 작아지려 하면 멈춤. AC-R-5 / AC-R-7 PASS.

### MS-4: D-2 Workspace switcher polish (audit D-2)

후속 milestone (post-implementation, 2026-05-01 sess 8 추가). audit feature-audit.md D-2 의 PARTIAL 상태 ("Sidebar lists workspaces, active highlighting works. Missing: drag-to-reorder, context menu (rename/delete), quick switcher") 해소를 위한 skeleton 도입.

- **범위**: `crates/moai-studio-ui/src/workspace_menu.rs` 모듈 신규 — `WorkspaceMenuAction` enum (Rename / Delete / MoveUp / MoveDown) + `WorkspaceMenu` struct (target id + visible position) + mutation API (`open_for` / `close` / `is_visible_for`). `explorer/context_menu.rs` 패턴 재사용.
- **포함 요구사항** (frozen-zone):
  - **REQ-D2-MS4-1**: WorkspaceMenuAction 4 variant (Rename / Delete / MoveUp / MoveDown) 가 stable enum 으로 노출된다.
  - **REQ-D2-MS4-2**: 외부 호출자가 `WorkspaceMenu::open_for(workspace_id, x, y)` / `close()` / `is_visible_for(workspace_id)` 로 menu 상태를 조작할 수 있다. 동일 시점에 여러 workspace 의 menu 가 동시에 열리지 않는다.
  - **REQ-D2-MS4-3**: 본 milestone 은 skeleton 까지만 도입한다. 실제 rename modal / delete confirmation / reorder dispatch 와 RootView 통합은 후속 PR. 기존 workspace_row 호출 동작에 regression 0.
- **제외 (Deferred carry to v0.2.0)**:
  - D-4 (Global search across workspaces), D-5 (Workspace color tags), D-6 (Drag-and-drop workspace add) — audit 명시 "deferred to v0.2.0".
  - Quick switcher (⌘/Ctrl+,) — audit line 143 carry, 별도 PR.
  - Real rename / delete / reorder dispatch — RootView store mutation 과 결합 필요.
- **AC**: AC-D2-1 ~ AC-D2-5 (§10 표).
- **시연 가능 상태**: 외부 코드가 mutation API 호출 시 menu state 가 정확히 갱신. 단일 menu invariant 유지.

### MS-5: D-2 Workspace switcher real dispatch (audit D-2 follow-up, v0.2.0 cycle)

후속 milestone (MS-4 skeleton 의 실 동작 완성, 2026-05-04 sess 11 추가). MS-4 의 `WorkspaceMenuAction` 4 variant (Rename / Delete / MoveUp / MoveDown) 가 실 store mutation 까지 연결되도록 dispatch logic + RenameModal logic + DeleteConfirmation logic + WorkspacesStore API 확장 추가. RootView 우클릭 wire 는 GPUI render side carry (별 milestone 또는 별 SPEC).

- **범위**:
  - `crates/moai-studio-workspace/src/lib.rs`: `WorkspacesStore::rename(id, new_name) -> Result`, `move_up(id) -> Result`, `move_down(id) -> Result` 신규 API. 단위 테스트 포함.
  - `crates/moai-studio-ui/src/workspace_menu.rs`: `RenameModal` struct (target_id + buffer + open/commit/cancel API), `DeleteConfirmation` struct (target_id + confirm/cancel API), `dispatch_workspace_menu_action(action, ws_id, store) -> WorkspaceMenuOutcome` adapter. 단위 테스트 포함.
  - `crates/moai-studio-ui/src/lib.rs`: `RootView::handle_workspace_menu_action(action, ws_id, cx)` 메서드 — RenameModal/DeleteConfirmation toggle 또는 store mutation 직접 호출. `rename_modal` / `delete_confirmation` 옵셔널 필드 추가 (R3 새 필드만).
- **포함 요구사항** (frozen-zone):
  - **REQ-D2-MS5-1**: `WorkspacesStore::rename(id, new_name)` 가 (a) workspace 존재 확인 후 name 갱신 + save 호출, (b) name 가 빈 string trim 시 `WorkspaceError::EmptyName`, (c) workspace id 없을 시 `WorkspaceError::NotFound` 반환.
  - **REQ-D2-MS5-2**: `WorkspacesStore::move_up(id)` 가 list 에서 id 의 인덱스를 1 감소 (0 인덱스 no-op + Ok). `move_down(id)` 가 1 증가 (last 인덱스 no-op + Ok). 모두 save 호출. id 없을 시 `WorkspaceError::NotFound`.
  - **REQ-D2-MS5-3**: `RenameModal::open(ws_id, current_name)` → `set_buffer(s)` → `commit() -> Option<(ws_id, new_name)>` (cancel 시 `None`). 빈 trim buffer commit → `None`.
  - **REQ-D2-MS5-4**: `DeleteConfirmation::open(ws_id)` → `confirm() -> Option<ws_id>` (cancel 시 `None`).
  - **REQ-D2-MS5-5**: `dispatch_workspace_menu_action(action, ws_id, store)` 가 (a) `Rename` → `WorkspaceMenuOutcome::OpenRenameModal { ws_id, current_name }`, (b) `Delete` → `WorkspaceMenuOutcome::OpenDeleteConfirmation { ws_id }`, (c) `MoveUp` → `store.move_up(ws_id)` 호출 후 `WorkspaceMenuOutcome::Reordered`, (d) `MoveDown` → `store.move_down(ws_id)` 호출 후 `WorkspaceMenuOutcome::Reordered`. unknown ws_id → `WorkspaceMenuOutcome::Unknown`.
- **제외 (Deferred carry)**:
  - **RootView 우클릭 wire** (workspace_row right-click → `WorkspaceMenu::open_for(ws_id, x, y)`) — GPUI render side, 별 milestone 또는 별 SPEC.
  - **Quick switcher (⌘/Ctrl+,)** — 별 SPEC 또는 별 PR.
  - D-4 / D-5 / D-6 — v0.2.0 의 별 SPEC.
- **AC**: AC-D2-6 ~ AC-D2-10 (§10 표 — MS-5 추가 분).
- **시연 가능 상태**: `dispatch_workspace_menu_action` 호출 시 store 가 실 mutation (rename / move) 또는 modal outcome 반환. 단위 테스트 + integration 으로 검증. RootView 우클릭 wire 는 별 PR 진행 시 `handle_workspace_menu_action` 호출만으로 e2e 완성.

### MS-6: D-2 Workspace switcher GPUI overlay mount (audit D-2 fully closed, v0.2.0 cycle)

후속 milestone (MS-5 logic-level dispatch 의 GPUI render side 완성, 2026-05-04 sess 12 추가). MS-5 가 logic-level 까지 끝낸 뒤 carry 였던 (a) `workspace_row` 우클릭 → `WorkspaceMenu::open_for`, (b) ContextMenu overlay render, (c) RenameModal overlay render (text input + commit/cancel), (d) DeleteConfirmation overlay render (confirm/cancel) 4 축을 GPUI element tree 에 mount 한다. MS-5 의 `handle_workspace_menu_action(cx)` 가 이미 존재하므로 본 MS 는 render side wire 만 추가.

- **범위**:
  - `crates/moai-studio-ui/src/lib.rs`:
    - `RootView` 에 `workspace_menu: workspace_menu::WorkspaceMenu` 필드 신규 (R3 새 필드만 추가, 기존 필드 무변경).
    - `workspace_row` 우클릭 listener (`MouseButton::Right`) 추가 — `WorkspaceMenu::open_for(ws_id, x, y)` 호출 + `cx.notify()`.
    - `Render for RootView` 에 ContextMenu / RenameModal / DeleteConfirmation overlay branch 추가 (existing palette / settings_modal overlay 슬롯 패턴 재사용).
    - ContextMenu overlay: `WorkspaceMenu::is_open()` true 일 때 4 항목 menu (Rename / Delete / Move Up / Move Down) 를 visible_position 에 absolute mount, 항목 클릭 시 `handle_workspace_menu_action(action, ws_id, cx)` 호출 후 `WorkspaceMenu::close()`.
    - RenameModal overlay: `rename_modal.is_some()` true 일 때 modal box (text input + Commit / Cancel 버튼). Commit 시 `RenameModal::commit()` → `Some((ws_id, new_name))` 받아 `store.rename(&ws_id, &new_name)` 호출 + `rename_modal = None` + `cx.notify()`. Cancel 시 `rename_modal = None`.
    - DeleteConfirmation overlay: `delete_confirmation.is_some()` true 일 때 modal box (경고 텍스트 + Confirm / Cancel 버튼). Confirm 시 `DeleteConfirmation::confirm()` → `Some(ws_id)` 받아 `store.remove(&ws_id)` + `workspaces.retain(|w| w.id != ws_id)` + `delete_confirmation = None` + `cx.notify()`. Cancel 시 `delete_confirmation = None`.
- **포함 요구사항** (frozen-zone):
  - **REQ-D2-MS6-1**: `RootView` 가 `workspace_menu: WorkspaceMenu` 필드를 보유한다. `Default::default()` 로 초기화 (closed state).
  - **REQ-D2-MS6-2**: `workspace_row` 의 우클릭 (`MouseButton::Right`) 이벤트가 `WorkspaceMenu::open_for(ws_id, x, y)` 를 호출하고 `cx.notify()` 를 트리거한다. 좌클릭 동작 (activate workspace) regression 0.
  - **REQ-D2-MS6-3**: `Render for RootView` 가 `WorkspaceMenu::is_open()` 시 4 항목 ContextMenu overlay 를 visible_position 에 mount 한다. 항목 클릭은 `handle_workspace_menu_action(action, ws_id, cx)` → `WorkspaceMenu::close()` 호출. 단일 menu invariant (REQ-D2-MS4-2) 유지.
  - **REQ-D2-MS6-4**: `Render for RootView` 가 `rename_modal.is_some()` 시 RenameModal overlay 를 mount 하고 commit 시 `store.rename` 호출 + `workspaces` 의 해당 entry name 갱신 + `cx.notify()`. `delete_confirmation.is_some()` 시 DeleteConfirmation overlay 를 mount 하고 confirm 시 `store.remove` 호출 + `workspaces.retain` + `cx.notify()`.
- **제외 (Deferred carry)**:
  - keyboard navigation in ContextMenu (Arrow / Esc / Enter) — 별 PR.
  - Drag-to-reorder workspace — D-6 carry (별 SPEC).
  - Workspace color tag editing — D-5 carry (별 SPEC).
- **AC**: AC-D2-11 ~ AC-D2-14 (§10 표 — MS-6 추가 분).
- **시연 가능 상태**: `cargo run -p moai-studio-app` 시 사용자가 sidebar workspace_row 를 우클릭하면 4 항목 메뉴 표시, Rename 클릭 시 텍스트 입력 modal 등장, Commit 시 sidebar 의 workspace 이름이 즉시 갱신. Delete 클릭 시 confirmation modal 등장, Confirm 시 workspace 가 sidebar 에서 제거. 좌클릭 (workspace activate) regression 0. D-2 audit 항목 PARTIAL → DONE.

---

## 9. 파일 레이아웃 (canonical)

### 9.1 신규

- `crates/moai-studio-ui/src/panes/render.rs` — 재귀 렌더 함수 `render_pane_tree<L: Render + 'static>`.
- `crates/moai-studio-ui/src/panes/render/mod.rs` 또는 `panes/render.rs` 단일 파일 — MS-1 시점 결정.
- `crates/moai-studio-ui/tests/integration_render.rs` — TestAppContext 기반 통합 테스트 (USER-DECISION PASS 시).

### 9.2 수정

- `crates/moai-studio-ui/src/lib.rs:72-99` — `RootView` 정의: `pane_splitter` 필드 제거, `tab_container: Option<Entity<TabContainer>>` 신규.
- `crates/moai-studio-ui/src/lib.rs:170-202` — `Render for RootView`: tab_container 분기 + key handler 등록.
- `crates/moai-studio-ui/src/lib.rs:294-308` — `main_body` / `content_area` 시그니처를 `tab_container` 기반으로 교체.
- `crates/moai-studio-ui/src/tabs/container.rs` — `impl Render for TabContainer` 추가 (또는 `tabs/render.rs` 분리).
- `crates/moai-studio-ui/src/tabs/keys.rs` — `keystroke_to_tab_key` 신규 함수.
- `crates/moai-studio-ui/src/panes/divider.rs` — GPUI element 변환 helper 추가 (`fn divider_element(orientation, ...) -> impl IntoElement`), 기존 logic 무변경.
- `crates/moai-studio-ui/Cargo.toml` — `dev-dependencies` 의 `gpui` 에 `features = ["test-support"]` 추가 (USER-DECISION PASS 시).
- `crates/moai-studio-ui/src/panes/mod.rs` — `pub mod render;` 추가.

### 9.3 변경 금지 (FROZEN — REQ-R-060 ~ REQ-R-063)

- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-ui/src/terminal/**` 전체 (재사용만).
- `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs}` 의 공개 API.
- `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` 의 기존 메서드 시그니처. (`impl Render for TabContainer` 추가는 허용.)
- `crates/moai-studio-workspace/src/persistence.rs` 의 `moai-studio/panes-v1` schema.

---

## 10. Acceptance Criteria

| AC ID | Requirement Group | Milestone | Given | When | Then | 검증 수단 |
|-------|-------------------|-----------|-------|------|------|-----------|
| AC-R-1 | RG-R-1 (REQ-R-001, REQ-R-002) | MS-1 | 사용자가 `cargo run -p moai-studio-app` 으로 앱 실행. workspace 1 개 활성. | RootView 렌더 시점 | `RootView.tab_container` 가 `Some(Entity<TabContainer>)` 이고 `impl Render for TabContainer` 가 호출되어 탭 바 1 개와 본문 1 leaf 가 가시한다. panic 없음. | 수동 smoke test + cargo test (RootView 단위) |
| AC-R-2 | RG-R-2 (REQ-R-010 ~ REQ-R-014) | MS-2 | TabContainer 의 활성 탭 PaneTree 가 1 회 horizontal split 된 상태 (2 leaf + 1 split node) | RootView 렌더 시점 | render 결과 element tree 에 `flex_row` 컨테이너 1 개, leaf element 2 개, divider element 정확히 1 개가 존재한다. | cargo test (USER-DECISION 결과에 따라 TestAppContext or render_pane_tree unit) |
| AC-R-3 | RG-R-4 (REQ-R-030 ~ REQ-R-032) | MS-2 | RootView 활성, TabContainer.tabs.len() == 1 | 사용자가 Cmd+T (macOS) / Ctrl+T (Linux) 입력 | TabContainer.tabs.len() == 2, active_tab_idx == 1, RootView render 가 다시 호출되어 탭 바에 2 개 탭이 가시. | integration test (TestAppContext + simulate_keystroke) 또는 logic-level (handle_key 직접 호출) |
| AC-R-4 | RG-R-4 (REQ-R-033, REQ-R-034) | MS-2 | TabContainer 활성, 단일 leaf 활성 탭, focused leaf 가 root pane | 사용자가 Cmd/Ctrl+\\ 입력 | 활성 탭의 PaneTree 가 `Split { direction: Horizontal, ratio: 0.5, ... }` 로 교체되고, RootView render 결과에 새 leaf element 1 개와 divider element 1 개가 추가된다. | integration test 또는 logic + render_pane_tree unit |
| AC-R-5 | RG-R-3 (REQ-R-020 ~ REQ-R-022), RG-R-5 (REQ-R-040, REQ-R-041) — **SPEC-V3-003 AC-P-4 carry-over 직접 승계** | MS-3 | 활성 탭 PaneTree 가 horizontal split (좌 60% / 우 40%, 윈도우 폭 1600px → 좌 960px / 우 640px). | 사용자가 마우스로 divider 를 좌측으로 drag 하여 우측 sibling 이 `MIN_COLS = 40` 미만 (즉, 40 × `advance_width=8` = 320px 미만) 이 되도록 시도 | 새 ratio 가 clamp 되어 우측 sibling 이 정확히 320px (또는 그 직전 안전 경계) 로 멈춘다. PaneTree.get_ratio(node_id) 가 clamp 된 값 반환. tracing::debug 로그 1 건. | integration test (TestAppContext mouse simulation) 또는 logic-level: GpuiDivider::on_drag 결과를 RootView wire-up 의 mock cx 에서 검증 |
| AC-R-6 | RG-R-1 (REQ-R-005), USER-DECISION 게이트 직접 검증 | MS-1 | gpui crate 가 `features = ["test-support"]` 활성화 (USER-DECISION 결과 채택 시) 또는 비활성 (비채택 시 — logic-level fallback) | `cargo test -p moai-studio-ui --tests` 실행 | (a) 채택 시: integration_render.rs 테스트가 빌드 + 실행 GREEN. (b) 비채택 시: logic-level 대체 unit test 가 GREEN, README 또는 spec.md 의 USER-DECISION 항목에 비채택 사실 명시 + AC-R-2/3/4/5 가 logic-level 로 검증되었음을 progress.md 에 기록. | cargo test + progress.md 검토 |
| AC-R-7 | RG-R-5 (REQ-R-040 ~ REQ-R-042) — **SPEC-V3-003 AC-P-4 의 element-tree 측 검증** | MS-3 | TabContainer 의 활성 탭이 3 level split (1 horizontal + 2 vertical = 4 leaf) | RootView 렌더 시점 | render 결과 element tree 에 divider element 가 정확히 3 개 (split 노드 수와 일치) 존재한다. divider 의 orientation (수직/수평) 이 각 split 의 direction 과 일치. | unit test (render_pane_tree 결과 element tree 자식 수 검증) — TestAppContext 비채택 시 가능 |
| AC-R-8 | RG-R-6 (REQ-R-060 ~ REQ-R-063) | 전체 | 전체 milestone 완료 후 | `cargo test -p moai-studio-terminal` + `cargo test -p moai-studio-workspace` + `cargo test -p moai-studio-ui --lib panes::` + `cargo test -p moai-studio-ui --lib tabs::` 실행 | SPEC-V3-002/003 기존 tests 전원 GREEN. terminal crate, panes/tabs 의 기존 unit tests, workspace crate 의 persistence tests 모두 0 regression. | CI gate / cargo test |
| AC-D2-1 | REQ-D2-MS4-1 | MS-4 | `WorkspaceMenuAction::all()` 또는 enum exhaustive match | 4 variant 모두 (Rename / Delete / MoveUp / MoveDown) 노출, label 매핑 비어있지 않음 | unit test (`label()` 모든 variant 검증) |
| AC-D2-2 | REQ-D2-MS4-2 | MS-4 | `WorkspaceMenu::default()` 인스턴스 | `is_visible_for("ws-1")` 호출 | false 반환 (menu closed by default), `visible_target()` Some/None 분기 정상 | unit test |
| AC-D2-3 | REQ-D2-MS4-2 | MS-4 | `open_for("ws-1", 100.0, 200.0)` 호출 후 | `is_visible_for("ws-1")` + `is_visible_for("ws-2")` + `visible_target()` | "ws-1" → true, "ws-2" → false, target = Some("ws-1"), position = (100, 200) | unit test |
| AC-D2-4 | REQ-D2-MS4-2 (single-menu invariant) | MS-4 | menu 가 "ws-1" 으로 열려 있는 상태 | `open_for("ws-2", ...)` 호출 | menu target 이 "ws-2" 로 교체. "ws-1" → false, "ws-2" → true (single-menu invariant 유지) | unit test |
| AC-D2-5 | REQ-D2-MS4-2 | MS-4 | menu 가 어떤 workspace 로 열려 있는 상태 | `close()` 호출 | menu invisible, `visible_target()` = None, `is_visible_for(*)` = false | unit test |
| AC-D2-11 | REQ-D2-MS6-1 | MS-6 | `RootView::new(...)` 직후 인스턴스 | `workspace_menu` 필드 접근 | `WorkspaceMenu::default()` 와 동일 (closed, no target, no position) | unit test |
| AC-D2-12 | REQ-D2-MS6-2 | MS-6 | sidebar 에 workspace 1 개 표시, 사용자가 row 를 우클릭 | RootView 가 mouse-down(Right) 이벤트 처리 | `workspace_menu.is_visible_for(ws.id)` true, `visible_position()` = Some(MenuPosition), 좌클릭 처리 (`handle_activate_workspace`) regression 0 | logic test (RootView::open_workspace_menu_at helper) + 수동 smoke |
| AC-D2-13 | REQ-D2-MS6-3 | MS-6 | `workspace_menu` 가 "ws-1" 으로 open 상태 | 사용자가 ContextMenu 의 Rename 항목 클릭 | `rename_modal` Some (target_id="ws-1", buffer=current_name), `workspace_menu` closed (REQ-D2-MS4-2 invariant) | logic test (RootView::click_workspace_menu_item helper) |
| AC-D2-14 | REQ-D2-MS6-4 | MS-6 | `rename_modal` open (target="ws-1", buffer="NewName"), `delete_confirmation` open (target="ws-2"), store 에 두 workspace 등록 | Rename Commit + Delete Confirm 호출 | (a) "ws-1" name = "NewName" (store + workspaces vector 양쪽), (b) "ws-2" 가 store + workspaces vector 양쪽에서 제거, (c) rename_modal = None, delete_confirmation = None | logic test (RootView::commit_rename_modal + confirm_delete_modal helpers) |

---

## 11. 의존성 및 제약

### 11.1 외부 의존성

| Crate | 버전 / 상태 | 비고 |
|-------|-------------|------|
| `gpui` | 0.2.2 (SPEC-V3-001/002/003 와 동일) | 변경 없음 |
| `gpui` `test-support` feature | **USER-DECISION 게이트 (MS-1 진입 시)** | dev-dependencies 에 features 추가, 비채택 시 logic-level fallback (research §6 참조) |
| `serde` / `serde_json` | workspace 기본 | 본 SPEC 신규 직렬화 없음 (persistence read-only) |
| `arboard` | 3.0 (SPEC-V3-002 carry) | 변경 없음 |
| `tracing` | workspace | divider drag debug 로그 (REQ-R-022) |

### 11.2 USER-DECISION 게이트

- **[USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-004]** — MS-1 진입 직전 발동.
  - Option (a) **(권장)**: gpui crate 의 `test-support` feature 를 `dev-dependencies` 에 추가. 비용 1 줄, 가치: AC-R-2/3/4/5 의 integration test 가 실제 GPUI 환경에서 검증됨.
  - Option (b): 채택하지 않고 logic-level fallback 으로 진행. 비용: 우회 unit test 코드 약 100-150 LOC 추가, AC 검증 충실성 약간 감소.
  - Default: option (a). 비채택 시 progress.md 에 명시.

### 11.3 내부 의존성

- `crates/moai-studio-terminal` (SPEC-V3-002 완료) — 무변경 carry.
- `crates/moai-studio-ui::{panes, tabs, terminal}` (SPEC-V3-003 완료) — 공개 API 무변경, 내부 helper 추가 허용.
- `crates/moai-studio-workspace::persistence` (SPEC-V3-003 MS-3) — 무변경 carry.

### 11.4 시스템/도구 제약

- Rust stable 1.93+ (SPEC-V3-002 carry).
- macOS 14+ / Ubuntu 22.04+. Windows 는 본 SPEC 범위 밖.
- 기존 `mlugg/setup-zig@v2` CI 스텝 유지 (Terminal Core 링크).
- USER-DECISION 게이트 PASS 시: gpui test-support 가 macOS/Linux CI 양쪽에서 빌드 통과해야 함 — Spike 0 (research §6.2).

### 11.5 Git / Branch 제약

- 본 SPEC 구현은 `feature/SPEC-V3-004-render` 브랜치에서 진행.
- `main` 직접 커밋 금지 (CLAUDE.local.md §1).
- 각 MS 는 squash 머지를 위한 단일 또는 그룹 커밋으로 분리.

---

## 12. 위험 및 완화

상세 분석은 `.moai/specs/SPEC-V3-004/research.md` §10 참조.

| ID | 위험 | 영향 | 완화 전략 | research 참조 |
|----|------|------|-----------|---------------|
| R-R1 | gpui Entity / Render trait 학습 곡선 | MS-1 일정 지연, 잘못된 패턴 사용 | TerminalSurface impl Render (lib.rs:262) 와 RootView impl Render (lib.rs:170) 를 reference 로 활용. MS-1 시작 시 4h budget spike. | research §2 |
| R-R2 | gpui test-support feature CI 빌드 실패 (Linux) | USER-DECISION 채택 시 테스트 환경 분기 | Spike 0 (≤ 1h, MS-1 진입 직후): macOS + Linux 모두 `cargo test --features test-support` 빌드 통과 검증. 실패 시 자동 fallback option (b). | research §6.2 |
| R-R3 | divider drag GPUI API 가 0.2.2 에서 빈약 | AC-R-5 검증 어려움 | SPEC-V3-003 Spike 1 결과 (interactivity API 확인) 코드 패턴 재사용. `Stateful<Div>::on_mouse_down/move/up` 조합으로 충분. | research §3.3 |
| R-R4 | TabContainer.update 호출 시 PaneTree mutation 배선 복잡성 | 키 이벤트 → 상태 변경 누락 | RG-R-4 의 명확한 dispatch 경로 (§7.2) + per-command 단위 테스트 (AC-R-3, AC-R-4). | research §4.3 |
| R-R5 | render layer 도입이 SPEC-V3-003 logic test 를 깨뜨림 | regression | RG-R-6 (REQ-R-060 ~ REQ-R-063) 무변경 원칙, AC-R-8 의 CI gate. | research §8.1 |
| R-R6 | Cmd/Ctrl 키 바인딩 충돌 (다른 OS 단축키) | UX 이슈 | SPEC-V3-003 AC-P-9a/9b 정책 그대로 carry, 추가 OS-level 단축키 추가하지 않음. | research §10 |
| R-R7 | gpui-test-support 비채택 시 fallback 우회 코드 복잡성 | logic-level test 중복, 유지비 증가 | Default 권장이 option (a). 비채택 시 progress.md 에 명시 + 우회 코드량 100-150 LOC 상한. | research §6.2 |

---

## 13. 참조 문서

### 13.1 본 레포 내

- `.moai/specs/SPEC-V3-004/research.md` — 본 SPEC 의 코드베이스 분석.
- `.moai/specs/SPEC-V3-003/spec.md` §9.2 — `pane_splitter` → `tab_container` rename 의 사전 정의.
- `.moai/specs/SPEC-V3-003/contract.md` §11.6, §11.7 — AC-P-4 / AC-P-5 carry-over 결정 원본.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 원칙.
- `.moai/specs/SPEC-V3-001/spec.md` — RootView scaffold 전제.
- `crates/moai-studio-ui/src/lib.rs:170-202` — RootView impl Render reference.
- `crates/moai-studio-ui/src/terminal/mod.rs:262-306` — TerminalSurface impl Render reference.
- `crates/moai-studio-ui/src/panes/divider.rs:80-115` — GpuiDivider 구체 구현체.
- `crates/moai-studio-ui/src/panes/splitter_gpui_native.rs:58-100` — GpuiNativeSplitter generic factory 패턴.
- `crates/moai-studio-ui/src/tabs/container.rs:117-242` — TabContainer 공개 API.
- `crates/moai-studio-ui/src/tabs/keys.rs:92-113` — dispatch_tab_key 기존 정의.
- `.moai/design/v3/spec.md:420-438` — 플랫폼별 키 바인딩 (SPEC-V3-003 carry).

### 13.2 외부 참조

- [Zed gpui::Render 패턴](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/view.rs) — `impl Render` reference.
- [Zed Pane GPUI 통합](https://github.com/zed-industries/zed/blob/main/crates/workspace/src/pane.rs) — Pane render 패턴 reference.
- [Zed gpui TestAppContext](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app/test_context.rs) — test-support 활성화 시 사용 패턴.

---

## 14. Exclusions

본 SPEC 이 명시적으로 **다루지 않는** 항목 (별도 SPEC 으로 분리):

- E1. **실제 PTY worker per-pane spawn** — 본 SPEC 의 leaf payload 는 placeholder 또는 단일 TerminalSurface 재사용. 분할 시 새 PtyWorker 가 spawn 되는 동작은 별도 SPEC (예: SPEC-V3-005 Pane PTY Lifecycle).
- E2. **Tab close UI element / mouse close** — 키 바인딩 only.
- E3. **PaneTree drag-and-drop 재배치 / pane zoom / named layout** — SPEC-V3-003 §3.2 Non-Goal 그대로.
- E4. **Persistence schema 변경 / 새 필드 추가** — SPEC-V3-003 MS-3 산출 그대로.
- E5. **Terminal Core 변경 / 추가 FFI 호출** — RG-R-6 carry.
- E6. **다중 윈도우 / 탭 detach / 새 design token 추가** — 별도 SPEC.
- E7. **macOS / Linux 외 플랫폼 (Windows, BSD)** — SPEC-V3-002/003 carry.
- E8. **scrollback rendering / VT escape sequence 처리** — SPEC-V3-002 범위.
- E9. **GPUI 0.3+ 마이그레이션** — Phase 7+ 별도 SPEC.

---

## 15. 용어 정의

- **Render Layer**: `impl Render for X` + GPUI Entity 시스템을 통한 화면 렌더링 책임 계층.
- **TabContainer Entity**: `cx.new(|_cx| TabContainer::new())` 로 만든 `Entity<TabContainer>` 인스턴스. RootView 가 `Option<Entity<TabContainer>>` 로 보유.
- **render_pane_tree**: `panes::render` 모듈의 재귀 함수. PaneTree<L> → GPUI element tree 변환.
- **Horizontal split (좌/우 분할)**: `SplitDirection::Horizontal`, 시각적으로는 **수직 divider** 가 보임. SPEC-V3-003 §15 carry.
- **Vertical split (상/하 분할)**: `SplitDirection::Vertical`, 시각적으로는 **수평 divider** 가 보임. SPEC-V3-003 §15 carry.
- **escape hatch**: SPEC 분할 전략에서 carry-over 되었던 검증 항목을 다음 SPEC 이 책임지고 종결시키는 지점. 본 SPEC 은 SPEC-V3-003 의 escape hatch.
- **AC carry-over**: 한 SPEC 의 AC 가 다른 SPEC 으로 책임 이관되는 것. 본 SPEC 의 AC-R-5 / AC-R-7 가 SPEC-V3-003 AC-P-4 의 직접 승계자.
- **USER-DECISION 게이트**: 명확한 default 와 옵션이 있으나 사용자 명시 결정이 필요한 분기점. 본 SPEC 은 1 개 (gpui-test-support-adoption-v3-004).

---

## 16. 열린 결정 사항

| ID | 결정 사항 | Default / 권장 | 결정 시점 |
|----|----------|----------------|----------|
| OD-R1 | gpui `test-support` feature 채택 여부 | (a) 채택 (권장) | MS-1 진입 직전 ([USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-004]) |
| OD-R2 | `impl Render for TabContainer` 위치: `tabs/container.rs` 인라인 vs `tabs/render.rs` 분리 | 인라인 (container.rs) — 단일 책임 + 작은 코드량 | MS-1 시작 시 |
| OD-R3 | leaf payload 타입 결정: `String` placeholder vs `Entity<TerminalSurface>` 재사용 | placeholder String — 본 SPEC 의 실제 PTY spawn 은 Non-Goal (E1) | MS-2 시작 시 |
| OD-R4 | `keystroke_to_tab_key` 의 위치: `tabs/keys.rs` 인라인 vs 별도 모듈 | tabs/keys.rs 인라인 | MS-2 시작 시 |
| OD-R5 | divider drag 의 정밀한 GPUI API: `interactivity().on_drag` vs `Stateful<Div> + on_mouse_*` 조합 | SPEC-V3-003 Spike 1 결과 reference, 동일 패턴 | MS-3 시작 시 |

---

작성: 2026-04-25
브랜치: `feature/SPEC-V3-004-render`
다음: plan.md (Milestone × Task table)
