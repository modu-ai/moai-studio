---
id: SPEC-V3-015
version: 1.0.0
status: draft
created: 2026-04-26
updated: 2026-04-26
author: manager-spec
priority: high
issue_number: TBD
depends_on:
  - SPEC-V3-009
  - SPEC-V3-004
labels:
  - phase-3
  - ui
  - gpui
  - root-view
  - integration
milestones:
  - id: MS-1
    name: SpecPanelView 통합 + RootView 배선
    priority: high
    acceptance:
      - AC-RV-1
      - AC-RV-2
      - AC-RV-3
      - AC-RV-4
      - AC-RV-5
    gates:
      - USER-DECISION-SU-RV-A
---

# SPEC-V3-015 — RootView 통합: SpecListView/KanbanBoardView/SprintContractPanel 진입점 등록

## HISTORY

- 2026-04-26: Initial draft. SPEC-V3-009 N6 carry 해소 — V3-009 산출 컴포넌트 3종을 RootView 에 배선. (manager-spec)

---

## 1. 개요

SPEC-V3-009 (PR #30/#31/#32) 가 status `implemented` 로 머지되었지만, 산출된 3개의 신규 spec_ui 컴포넌트가 **RootView 에 mount 되어 있지 않아 사용자가 접근할 수 없는 상태**다 (V3-009 N6 carry).

본 SPEC 은 V3-009 의 컴포넌트들을 묶는 단일 컨테이너 `SpecPanelView` 를 신규 작성하고, 이를 RootView 에 배선하여 사용자가 단축키 한 번으로 SPEC 워크플로 UI 전체에 접근할 수 있도록 한다.

### 산출 대상 (V3-009 결과물)

- `spec_ui::SpecListView` — `.moai/specs/` 스캔 기반 SPEC 카드 목록
- `spec_ui::KanbanBoardView` — 4 lane Kanban 보드 + sidecar 영속화
- `spec_ui::SprintContractPanel` — Sprint Contract revision timeline
- `spec_ui::command_client::MoaiCommandClient` — `moai` CLI subprocess + stream-json (백엔드, 본 SPEC 에서는 직접 wiring 없음)

### 통합 위치 후보

- `crates/moai-studio-ui/src/lib.rs` (RootView slot 등록 + Render impl)
- `crates/moai-studio-ui/src/spec_ui/` (SpecPanelView 신규 모듈)

---

## 2. 배경 및 동기

### 현재 문제 (Carry from V3-009 N6)

1. V3-009 의 3개 컴포넌트는 **단위 테스트 + ad-hoc story** 수준에서만 검증되었고, 실제 RootView 통합 경로가 비어 있음.
2. 사용자는 컴포넌트의 존재를 인지할 방법이 없고, runtime 에서 호출 진입점이 부재.
3. V3-009 의 AC-SU-1~5 는 컴포넌트 단위로 통과했으나, **end-to-end UX 흐름** (SPEC 목록 진입 → Kanban 전환 → Sprint Contract 확인) 은 미검증.

### 본 SPEC 의 가치

- V3-009 투자분의 **사용자 가시성 확보** (carry 부채 해소)
- SPEC 워크플로 UI 의 **통일된 진입점** 제공 (3 view mode 한 곳에서 전환)
- 향후 V3-006 / V3-016 등 SPEC 관련 SPEC 들이 본 컨테이너를 재사용할 수 있는 토대 마련

---

## 3. Goals / Non-Goals

### Goals

- **G1**: `SpecPanelView` 라는 단일 컨테이너 컴포넌트를 신규 작성. 내부에서 List / Kanban / Sprint 3개 view mode 를 전환할 수 있어야 한다.
- **G2**: `Cmd+Shift+S` (macOS) / `Ctrl+Shift+S` (Linux/Windows) 단축키로 SpecPanelView 를 mount/dismiss 할 수 있어야 한다.
- **G3**: V3-009 의 3개 컴포넌트가 **RootView 경유로 모두 접근 가능** 해야 한다 (현재는 0건).
- **G4**: SpecPanelView 의 위치는 USER-DECISION-SU-RV-A 결정에 따라 overlay / dedicated tab / right-sidebar 중 하나로 확정.

### Non-Goals

- **NG1**: V3-009 의 SpecListView/KanbanBoardView/SprintContractPanel **자체 동작 변경**. 본 SPEC 은 wiring + container 만 다룬다 (정상화 작업이 필요하면 별도 SPEC).
- **NG2**: `MoaiCommandClient` 의 신규 호출 경로 추가. V3-009 가 정의한 인터페이스를 그대로 유지.
- **NG3**: 신규 design token / 색상 / 타이포 정의. 기존 V3-009 스타일을 그대로 사용.
- **NG4**: Mouse drag-and-drop 기반 view mode 전환. 키보드 (Tab/Number key) + 명시적 mode selector 만 지원.
- **NG5**: SPEC 데이터 영속화 / Kanban sidecar 형식 변경 (V3-009 가 이미 정의함).
- **NG6**: 단축키 사용자 커스터마이즈. `Cmd+Shift+S` 고정.

---

## 4. 가정 및 제약

### 가정 (Assumptions)

- **A1**: V3-009 의 3개 컴포넌트는 별도 인자 없이 `cx.new(|cx| SpecListView::new(cx))` 패턴으로 인스턴스화 가능 (V3-009 패턴 준수, 미확인 시 MS-1 spike 단계에서 재검증).
- **A2**: 기존 RootView 의 overlay 패턴 (banner_stack, palette, settings_modal) 이 신규 overlay 추가에 충분.
- **A3**: GPUI key dispatch 는 `Cmd+Shift+S` 를 다른 단축키와 충돌 없이 캡처할 수 있다 (사전 grep 으로 확인 필요).

### 제약 (Constraints)

- **C1**: terminal/panes/tabs 의 핵심 동작 변경 금지 (RG-P-7 carry from V3-001~003). lib.rs 의 slot 추가 + Render mount 만 허용.
- **C2**: 신규 외부 crate 의존성 추가 금지.
- **C3**: V3-004 (RootView shell) 의 기존 layout 비율 / sidebar 폭 변경 금지.
- **C4**: 최소 `cargo test -p moai-studio-ui` GREEN 유지.

---

## 5. 요구사항 (EARS)

### 5.1 Ubiquitous Requirements

- **REQ-RV-1** (Ubiquitous): The system **shall** expose `spec_ui::SpecPanelView` as a unified container that hosts SpecListView, KanbanBoardView, and SprintContractPanel sub-views.
- **REQ-RV-2** (Ubiquitous): RootView **shall** maintain a `spec_panel: Option<Entity<SpecPanelView>>` slot, initialized in `RootView::new()`.

### 5.2 Event-Driven Requirements

- **REQ-RV-3** (Event-Driven): **When** the user presses `Cmd+Shift+S` (macOS) or `Ctrl+Shift+S` (other platforms), the system **shall** toggle SpecPanelView mount state (mount if dismissed, dismiss if mounted).
- **REQ-RV-4** (Event-Driven): **When** SpecPanelView is mounted, the system **shall** default to `ViewMode::List` and render the SpecListView sub-component.
- **REQ-RV-5** (Event-Driven): **When** the user activates a different view mode (Tab key cycle or explicit mode selector click), the system **shall** swap the active sub-view to the corresponding component (List / Kanban / Sprint).

### 5.3 State-Driven Requirements

- **REQ-RV-6** (State-Driven): **While** SpecPanelView is mounted, terminal/panes/tabs core behavior **shall** remain unchanged (no focus stealing beyond standard overlay semantics).

### 5.4 Unwanted Behavior

- **REQ-RV-7** (Unwanted): **If** SpecPanelView mount is requested while another modal overlay (settings_modal, palette) is already active, **then** the system **shall** dismiss the previous overlay before mounting SpecPanelView (single-overlay invariant).

---

## 6. Acceptance Criteria

### AC-RV-1: RootView slot 등록

**Given** `crates/moai-studio-ui/src/lib.rs` 의 `RootView` 구조체,
**When** RootView 가 `RootView::new(cx)` 로 초기화되면,
**Then** `spec_panel: Option<Entity<SpecPanelView>>` 필드가 `Some(cx.new(...))` 로 초기화되어 있어야 한다.

**Verification**:
- `grep -n "spec_panel" crates/moai-studio-ui/src/lib.rs` 결과 ≥ 2건 (선언 + 초기화)
- `cargo test -p moai-studio-ui --test root_view_smoke -- spec_panel_slot_initialized` GREEN

### AC-RV-2: 단축키로 mount/dismiss toggle

**Given** RootView 가 GPUI window 에 mount 된 상태,
**When** 사용자가 `Cmd+Shift+S` (macOS) 를 누르면,
**Then** SpecPanelView 의 `is_mounted` 플래그가 toggle 되어야 하며, 두 번째 입력 시 dismiss 되어야 한다.

**Verification**:
- 단위 테스트 `tests/spec_panel_toggle.rs` — 동일 키 입력 2회 → mount/dismiss state machine 검증
- USER-DECISION-SU-RV-A 의 위치 결정에 따라 visual verification 절차 추가

### AC-RV-3: View mode 전환

**Given** SpecPanelView 가 mount 된 상태에서 `ViewMode::List` 가 active,
**When** 사용자가 `Tab` 키 (cycle) 또는 모드 셀렉터의 `Kanban` 항목을 활성화하면,
**Then** active view mode 가 `ViewMode::Kanban` 으로 전환되고, KanbanBoardView 가 렌더링되어야 한다.

**Verification**:
- 단위 테스트 `tests/spec_panel_view_mode.rs` — List → Kanban → Sprint → List 순환 검증
- 각 모드 전환 시 직전 mode 의 entity 가 unmount 또는 적절히 hidden 처리되는지 확인

### AC-RV-4: 3 view mode 의 V3-009 컴포넌트 invoke

**Given** SpecPanelView 의 ViewMode enum 정의,
**When** 각 mode 가 active 일 때 `Render::render` 가 호출되면,
**Then** 다음과 같이 V3-009 컴포넌트를 invoke 해야 한다:
- `ViewMode::List` → `SpecListView` entity 렌더링
- `ViewMode::Kanban` → `KanbanBoardView` entity 렌더링
- `ViewMode::Sprint` → `SprintContractPanel` entity 렌더링

**Verification**:
- `grep -n "SpecListView\|KanbanBoardView\|SprintContractPanel" crates/moai-studio-ui/src/spec_ui/spec_panel_view.rs` 결과 ≥ 3건
- 단위 테스트 — 각 ViewMode 에서 expected child entity 타입 검증

### AC-RV-5: terminal/panes/tabs core 무변경

**Given** SPEC-V3-001/002/003/004 의 terminal/panes/tabs core 모듈,
**When** SPEC-V3-015 의 변경 diff 를 검토하면,
**Then** 다음 파일들의 변경은 0 LOC 이어야 한다:
- `crates/moai-studio-terminal/**` (모든 파일)
- `crates/moai-studio-ui/src/panes/**` (모든 파일)
- `crates/moai-studio-ui/src/tabs/**` (모든 파일)

**예외 허용**: `crates/moai-studio-ui/src/lib.rs` 는 RootView slot 등록 + Render mount 변경 허용 (slot 1개 + impl 분기 1곳).

**Verification**:
- `git diff develop -- crates/moai-studio-terminal crates/moai-studio-ui/src/panes crates/moai-studio-ui/src/tabs` 결과 empty
- `git diff develop -- crates/moai-studio-ui/src/lib.rs` 결과: slot 추가 + Render mount 분기 외 변경 없음

---

## 7. 비기능 요구사항

- **NFR-1** (성능): 단축키 입력 → SpecPanelView mount 까지 < 50ms (체감 instant).
- **NFR-2** (메모리): SpecPanelView 가 dismiss 상태일 때 child entity 들도 unmount 되어야 함 (lazy mount). 단, 첫 mount 이후 재mount 시 V3-009 컴포넌트의 자체 캐시 정책 준수.
- **NFR-3** (회귀 안전): `cargo test --workspace` GREEN. 신규 테스트 ≥ 3건 추가.

---

## 8. 의존성

- **SPEC-V3-009** (implemented): SpecListView, KanbanBoardView, SprintContractPanel, MoaiCommandClient 의 산출물 사용
- **SPEC-V3-004** (implemented): RootView shell + sidebar/content_area/overlay layout

---

## 9. 위험 및 완화

### R1: GPUI key dispatch 충돌 (Medium)

- **위험**: `Cmd+Shift+S` 가 기존 단축키 (terminal save, tab save 등) 와 충돌할 가능성.
- **완화**: 구현 첫 단계에서 `grep -rn "Cmd+Shift\|cmd-shift" crates/` 로 충돌 검사. 충돌 시 alternate (예: `Cmd+Shift+M`) 후보 제시.

### R2: V3-009 컴포넌트의 `new()` 시그니처 미스매치 (Low-Medium)

- **위험**: V3-009 컴포넌트 일부가 외부 의존성 (예: `MoaiCommandClient` instance) 을 생성자 인자로 요구할 수 있음.
- **완화**: MS-1 진입 직후 V3-009 spec.md / 실제 코드 1차 확인. 의존성 주입이 필요하면 SpecPanelView 에서 lazy-init 또는 Arc 공유.

---

## 10. USER-DECISION 게이트

### USER-DECISION-SU-RV-A: SpecPanelView 의 RootView 내 위치

V3-009 컴포넌트들을 묶은 SpecPanelView 를 어디에 mount 할지 결정해야 한다. 위치에 따라 layout / lifecycle / 단축키 dispatch 경로가 달라진다.

#### Option (a) — Overlay (권장)

`Cmd+Shift+S` 로 mount/dismiss 하는 modal-style overlay. banner_stack / palette / settings_modal 과 동일 패턴.

- **장점**: 기존 overlay 패턴 일관성, scoped lifecycle (mount/dismiss 명확), 단축키 toggle 자연스러움, RootView slot 1개 추가만으로 완결.
- **단점**: SPEC 작업 중 terminal/panes 가 가려짐 (단, dismiss 빠름).
- **추정 LOC**: ~250 LOC (SpecPanelView ~180 + lib.rs slot/render ~30 + tests ~40).

#### Option (b) — Dedicated tab

V3-003 TabContainer 에 신규 `TabKind::SpecPanel` 추가. 사용자가 새 탭을 열어 SPEC 워크플로 진입.

- **장점**: 다른 작업과 병행 가능 (tab 전환), persistent state 유지 자연스러움.
- **단점**: V3-003 TabContainer 의 TabKind enum 확장 필요 → RG-P-7 (terminal/panes/tabs core 무변경) 와 긴장. C1 제약 위반 소지. 단축키 동작 의미가 ambiguous (탭 새로 열기? 기존 탭 활성화?).
- **추정 LOC**: ~400 LOC (TabKind 확장 + TabContainer mount path + SpecPanelView + tests).

#### Option (c) — Right sidebar

좌측 file_explorer 와 대칭으로 우측에 신규 SPEC sidebar 신설.

- **장점**: 항상 visible, content_area 와 병렬 사용 가능.
- **단점**: V3-004 의 layout 비율 변경 필수 → C3 제약 위반. content_area 폭 축소 → terminal 가독성 저하 risk. 신규 sidebar primitive 가 없음 (file_explorer 와 다른 구조 필요).
- **추정 LOC**: ~500 LOC (right sidebar primitive + layout 재배치 + SpecPanelView + tests).

#### 권장 결정

**Option (a) Overlay** — 제약 (C1/C3) 위반 없음, 기존 패턴 재사용, 최소 LOC, V3-009 carry 해소라는 본 SPEC 의 단일 목적에 가장 부합.

#### 결정 시점

MS-1 착수 전. orchestrator 의 AskUserQuestion 으로 사용자 확정 필요.

---

## 11. Milestones

### MS-1: SpecPanelView 통합 + RootView 배선 (Priority: High)

**Scope**: 본 SPEC 의 모든 AC (AC-RV-1 ~ AC-RV-5) 를 단일 milestone 에서 처리. 작은 범위이므로 분할하지 않음.

**선결조건**:
- USER-DECISION-SU-RV-A 결정 완료 (overlay / tab / sidebar 중 1택)
- V3-009 컴포넌트 `new()` 시그니처 1차 확인 (R2 완화)
- `Cmd+Shift+S` 단축키 충돌 검사 통과 (R1 완화)

**산출물**:
- 신규: `crates/moai-studio-ui/src/spec_ui/spec_panel_view.rs` (~180 LOC)
- 수정: `crates/moai-studio-ui/src/lib.rs` (slot 1 + render branch ~30 LOC)
- 수정: `crates/moai-studio-ui/src/spec_ui/mod.rs` (`pub mod spec_panel_view; pub use spec_panel_view::SpecPanelView;`)
- 신규 테스트: `crates/moai-studio-ui/tests/spec_panel_*.rs` (~3 파일, ~120 LOC 합산)

**완료 기준 (DoD)**:
- AC-RV-1 ~ AC-RV-5 전부 GREEN
- `cargo test -p moai-studio-ui` GREEN
- `cargo test --workspace` GREEN
- `cargo clippy -p moai-studio-ui -- -D warnings` clean
- 수동 검증: GPUI app 실행 → `Cmd+Shift+S` → 3 mode 순회 → dismiss

**추정 총 LOC**: ~330 LOC (구현 ~210 + 테스트 ~120)

---

## 12. Out-of-Scope (재확인)

본 SPEC 은 V3-009 carry 해소만 다룬다. 아래 항목은 별도 SPEC 으로 분리:

- SpecPanelView 내 검색 / 필터 / 정렬 기능 (→ 후속 V3-016 후보)
- MoaiCommandClient 의 신규 명령 추가 (→ V3-006 또는 후속)
- SpecPanelView 의 contextual command palette 통합
- SPEC 생성 / 편집 wizard

---

## 13. 참조

- `.moai/specs/SPEC-V3-009/spec.md` (산출 컴포넌트 정의)
- `.moai/specs/SPEC-V3-004/spec.md` (RootView shell)
- `crates/moai-studio-ui/src/lib.rs:86-150` (기존 RootView slot 패턴)
- `crates/moai-studio-ui/src/spec_ui/` (V3-009 산출 모듈)
