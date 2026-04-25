# SPEC-V3-004 Research — Render Layer Integration

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render`
선행: SPEC-V3-001 (scaffold), SPEC-V3-002 (terminal core), SPEC-V3-003 (pane/tab logic) — 모두 완료 또는 carry-over 명시.
범위: GPUI 0.2.2 위에서 SPEC-V3-003 의 logic-only `TabContainer` / `PaneTree` 를 실제 화면에 그리는 render layer 통합.

---

## 1. 동기 — escape hatch 가 필요한 이유

### 1.1 SPEC-V3-003 의 사실상 완료와 두 가지 carry-over

SPEC-V3-003 contract.md §11.6 escalation protocol 와 §11.7 sprint exit criteria 는 다음을 명시한다:

- **AC-P-4 (carry-over from MS-2)**: "TabContainer 가 GPUI render 시 divider 가 실제 layout 에 포함되는지 integration 검증. **render layer 도입 어려우면 logic-level assertion 으로 대체.**"
- **AC-P-5 (carry-over from MS-1, deferred at MS-3)**: "gpui test-support 재평가 — 채택 시 headless resize unit"

진행 결과 (progress.md 기록 기준) MS-3 은 logic-level alternative 로 종결되었고, render layer 작업 자체는 **별도 SPEC** 으로 분리된다는 사용자 결정이 SPEC-V3-003 종결 시점에 내려졌다.

본 SPEC 은 그 분리된 render layer 작업의 캐리어다.

### 1.2 logic ↔ render 의 현재 격차

`crates/moai-studio-ui/src/lib.rs:83` 의 RootView 필드:

```
pub pane_splitter: Option<Entity<terminal::TerminalSurface>>,
```

이 필드는 `TabContainer` 가 도입된 SPEC-V3-003 MS-2 이후에도 단일 `TerminalSurface` 만 참조한다. 즉 코드베이스의 진실은:

| 레이어 | 상태 |
|--------|------|
| `panes::tree::PaneTree<L>` | ✅ Pure Rust enum + algorithms (29 unit + integration tests) |
| `tabs::container::TabContainer` | ✅ Pure Rust struct + lifecycle methods (5 AC unit tests) |
| `tabs::bar::TabBar::items` | ✅ Pure Rust 시각 상태 계산기 (AC-P-24/27 unit tests) |
| `panes::divider::GpuiDivider` | ✅ Pure Rust ratio clamp (AC-P-6 unit tests) |
| `panes::splitter_gpui_native::GpuiNativeSplitter<L>` | ⚠️ Generic factory 만 있음, prod 타입 미바인딩 |
| **`RootView` ↔ `TabContainer` 배선** | ❌ **미구현** — 본 SPEC 의 핵심 |
| **PaneTree → GPUI HStack/VStack 변환** | ❌ **미구현** |
| **Divider drag GPUI 이벤트 → set_ratio** | ❌ **미구현** (논리 함수만 존재) |
| **앱 레벨 키 dispatch → TabContainer** | ❌ **미구현** (`dispatch_tab_key` 호출자 없음) |

이 격차가 사용자 가시 동작 (탭 전환 보이기, 분할 보이기, divider 끌기) 을 막는 마지막 escape hatch 다.

### 1.3 사용자 가시 정의

본 SPEC 이 PASS 한 시점에 사용자가 `cargo run -p moai-studio-app` 으로 다음을 직접 관찰할 수 있어야 한다:

1. 첫 탭이 자동으로 열리고 단일 `TerminalSurface` (placeholder) 가 보인다.
2. Cmd/Ctrl+T 입력 시 새 탭이 생성되고 탭 바에 추가된다.
3. Cmd/Ctrl+1~9 입력 시 활성 탭이 바뀌고 본문 영역 (`content_area`) 의 `PaneTree` 가 교체된다.
4. Cmd/Ctrl+\\ 입력 시 활성 leaf 가 좌/우 분할되어 두 `TerminalSurface` 가 가로 배치되고 그 사이에 수직 divider 가 보인다.
5. Divider 를 마우스로 끌면 양쪽 pane 의 비율이 바뀌고, 한쪽이 `MIN_COLS` 미만이 되려 하면 더 이상 줄어들지 않는다 (AC-R-5 ↔ SPEC-V3-003 AC-P-4 carry-over).

---

## 2. GPUI 0.2.2 Render trait 패턴 분석

### 2.1 기존 prod 사례 — `TerminalSurface`

`crates/moai-studio-ui/src/terminal/mod.rs:262-306`:

```
impl Render for TerminalSurface {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut area = div().size_full().bg(rgb(0x1a1a1a)).flex().flex_col().p_2();
        // ... cursor_info, selection rect, cursor blink ...
        area.child(...).child(...)
    }
}
```

핵심 관찰:
- `Render::render(&mut self, window, cx) -> impl IntoElement` 시그니처
- `cx: &mut Context<Self>` 는 `cx.notify()`, `cx.listener(...)`, `cx.new(|cx| ...)` 를 제공
- `_window: &mut Window` 는 키 이벤트 루팅에 필요 (현 시점 사용처 없음)
- 반환 element 는 `div()` chain 으로 구성, `.child(...)` 로 자식 누적

### 2.2 RootView 의 listener 패턴

`crates/moai-studio-ui/src/lib.rs:170-202`:

```
impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let new_ws_btn = new_workspace_button().on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, cx| this.handle_add_workspace(cx)),
        );
        ...
    }
}
```

`cx.listener(closure)` 는 `Fn(&mut Self, &EvType, &mut Window, &mut Context<Self>)` 시그니처의 클로저를 받아 GPUI 가 인식하는 이벤트 핸들러로 변환한다. 본 SPEC 에서 divider drag, tab click, 키 dispatch 모두 이 패턴을 차용한다.

### 2.3 Entity 라이프사이클

`crates/moai-studio-ui/src/lib.rs:603-605`:

```
cx.open_window(options, move |_window, cx| {
    cx.new(|_cx| RootView::new(ws, path))
})
```

`cx.new(|cx| State::new())` 는 `Entity<State>` 를 만든다. 본 SPEC 에서 `TabContainer` 도 이 방식으로 생성하여 `Entity<TabContainer>` 를 RootView 가 보유한다.

부모 → 자식 entity 관계는 `cx.new` 로 만든 자식 Entity 를 부모 필드에 저장하면 된다 (`.clone()` 으로 cheap reference, GPUI 가 ref-count 관리).

### 2.4 GPUI Render API 의 안정성 위험

`panes::splitter_gpui_native.rs:110-114` 에 이미 명시된 `@MX:WARN` :
> GPUI 0.2.2 는 crates.io 공식판이나 Zed 팀이 main 브랜치에서 렌더 API 를 지속 변경하고 있다.

본 SPEC 도 동일 위험을 상속한다 (R-R1, §12).

---

## 3. PaneTree → GPUI Layout 변환 설계

### 3.1 PaneTree 의 자료구조

`panes::tree::PaneTree<L>` 는 enum:

```
PaneTree::Leaf(Leaf<L>)
PaneTree::Split { id, direction, ratio, first, second }
```

여기서 `direction` ∈ {Horizontal (좌/우), Vertical (상/하)}, `ratio: f32` ∈ (0.0, 1.0).

### 3.2 변환 규칙 (재귀)

```
fn render_pane_tree<L: Render>(tree: &PaneTree<L>, total_w: Pixels, total_h: Pixels) -> impl IntoElement {
    match tree {
        Leaf(L) => leaf_payload (Entity<TerminalSurface> 등),
        Split { direction: Horizontal, ratio, first, second } =>
            div().flex_row().w(total_w).h(total_h)
                .child(render_pane_tree(first, total_w * ratio, total_h))
                .child(divider_vertical())   // 수직 divider, drag → set_ratio
                .child(render_pane_tree(second, total_w * (1.0 - ratio), total_h)),
        Split { direction: Vertical, ratio, first, second } =>
            div().flex_col().w(total_w).h(total_h)
                .child(render_pane_tree(first, total_w, total_h * ratio))
                .child(divider_horizontal())
                .child(render_pane_tree(second, total_w, total_h * (1.0 - ratio))),
    }
}
```

핵심 결정 사항:
- **Total width/height 은 GPUI layout system 이 부모 div 에서 추론 가능** — `div().flex_row()` + 자식의 `flex_grow_X` 또는 `w(px)` 로 ratio 표현 가능. 명시적 픽셀 계산을 피하면 코드 단순화.
- **Leaf 의 `L` 타입**: prod 에서 `Entity<TerminalSurface>` 또는 SPEC-V3-004 시점에는 placeholder string 도 허용 (terminal spawn 통합은 별도 SPEC).
- **재귀 깊이**: SPEC-V3-003 AC-P-2 가 3-level split (8 leaf) 까지 검증 — 일반적으로 4-level 이내, GPUI 의 element tree 깊이 제한은 사실상 없음.

### 3.3 Divider 시각 + 인터랙션

수직 divider (Horizontal split 의 첫/둘째 사이):
- 폭 4-6px, 높이 100%, hover 시 색상 변경, cursor: ew-resize
- `on_mouse_down(MouseButton::Left)` → drag 시작 추적
- `on_mouse_move` → `GpuiDivider::on_drag(delta_px, total_px)` 호출 → 결과 ratio 로 `PaneTree::set_ratio(node_id, new_ratio)` 호출

수평 divider 동일하지만 높이 4-6px, cursor: ns-resize.

GPUI 의 mouse drag API 는 `interactivity().on_drag(...)` 또는 stateful div 의 `on_mouse_move` + 직접 좌표 추적이 필요. **본 SPEC 의 Spike 1** 이 정확한 API 표면을 결정한다.

---

## 4. 키 바인딩 GPUI dispatch 분석

### 4.1 `dispatch_tab_key` 의 caller 부재

`tabs::keys::dispatch_tab_key(modifiers, code) -> Option<TabCommand>` 는 SPEC-V3-003 T9 에서 작성되고 24 unit tests + 2 integration tests 가 있다. 그러나 실제 GPUI 키 이벤트로부터 호출하는 코드는 **존재하지 않는다**.

GPUI 0.2.2 의 키 이벤트:
- `Window::on_key_down` — 글로벌 키
- View 레벨 `key_context` + `Action` macro — Zed 패턴 (복잡, 현재 단계에선 과대)
- 단순 경로: RootView 가 `Window::on_key_down` 핸들러 등록 → `Keystroke` 를 `KeyModifiers` + `TabKeyCode` 로 변환 → `dispatch_tab_key` 호출

### 4.2 keystroke → TabKeyCode 변환

`Keystroke { modifiers: { control, command, shift, alt, function }, key: String, ... }` (`gpui::Keystroke`) 에서:

```
let mods = KeyModifiers {
    cmd: ks.modifiers.command,
    ctrl: ks.modifiers.control,
    shift: ks.modifiers.shift,
    alt: ks.modifiers.alt,
};
let code = match ks.key.as_str() {
    "t" => TabKeyCode::T,
    "1"..="9" => TabKeyCode::Digit(ks.key.parse().unwrap()),
    "\\" => TabKeyCode::Backslash,
    "{" => TabKeyCode::BraceOpen,
    "}" => TabKeyCode::BraceClose,
    _ => TabKeyCode::Other,
};
```

이 매핑 함수 자체가 본 SPEC 의 신규 코드 (`tabs::keys::keystroke_to_tab_key` 등).

### 4.3 TabCommand 실행

`dispatch_tab_key` 가 `Some(TabCommand::NewTab)` 등을 반환하면 RootView 의 핸들러가 `TabContainer` entity 에 `update` 메서드 호출:

```
self.tab_container.as_ref().map(|tc| tc.update(cx, |tc, cx| {
    match cmd {
        TabCommand::NewTab => { tc.new_tab(None); cx.notify(); }
        TabCommand::SwitchToTab(idx) => { let _ = tc.switch_tab(idx); cx.notify(); }
        TabCommand::SplitVertical => { /* PaneTree::split_vertical on active leaf */ }
        ...
    }
}));
```

`cx.notify()` 가 Render 재실행을 트리거.

---

## 5. AC-P-4 carry-over 의 render-layer 정의

SPEC-V3-003 AC-P-4 원문 (carry-over 시점 정의):
> "TabContainer 가 GPUI render 시 divider 가 실제 layout 에 포함되는지 integration 검증."

본 SPEC 에서의 구체화 (AC-R-5):
- (a) `TabContainer` 의 활성 탭이 split 된 `PaneTree` 를 가질 때, RootView 의 render 결과 element tree 에 `divider_vertical()` (또는 `_horizontal()`) 자식이 정확히 1 개 존재한다 (split 노드 당).
- (b) 사용자가 그 divider 를 마우스로 drag 하여 `delta_px` 가 ratio 변화로 이어진다.
- (c) `delta_px` 가 sibling 을 `MIN_COLS` 미만으로 만들려 할 때 ratio 는 clamp 되어 `set_ratio` 결과가 유지된다 (boundary rejection).

이 세 조건이 모두 GPUI integration 환경에서 검증되어야 carry-over 가 해소된다.

logic-level 대체 경로 (rerun fallback): GPUI test harness 가 미가용 시, `render_pane_tree` 함수의 element tree 구조를 unit test 로 assert (`fn render_split_emits_divider_child`).

---

## 6. GPUI test-support feature 재평가 — USER-DECISION 게이트

### 6.1 SPEC-V3-003 AC-P-5 의 deferral 사유

SPEC-V3-003 contract.md §11.6:
> "AC-P-5 carry-over 재평가: T11 시점에 DEFER 결정. MS-3 진입 시 다시 [USER-DECISION-REQUIRED: test-support-feature-adoption]. **default: DEFER (필수 아님, AC-P-5 자체 재후순)**"

deferral 이유:
- `gpui` crate 의 `test-support` feature 활성화는 `Cargo.toml` 수정 필요.
- SPEC-V3-003 MS-3 시점에는 persistence + CI 가 우선이라 채택 비용 vs 가치 비교에서 DEFER 우세.

### 6.2 SPEC-V3-004 에서의 가치 재평가

본 SPEC 은 render layer 가 핵심이므로 `TestAppContext` (gpui 의 headless render harness) 가 다음 AC 검증의 결정적 도구가 된다:
- AC-R-1 (TabContainer Entity render → element tree 검증)
- AC-R-2 (PaneTree split → HStack/VStack 자식 수 검증)
- AC-R-5 (divider drag boundary rejection — headless mouse event 시뮬레이션)
- AC-R-?? headless resize (SPEC-V3-003 AC-P-5 carry-over 해소)

채택 비용:
- `Cargo.toml` `dev-dependencies` 의 `gpui = { version = "0.2", features = ["test-support"] }` 추가 (1 줄).
- API 호환성 깨짐 위험은 features = additive 라 없음.
- CI 환경: `gpui` test-support 가 macOS/Linux 양쪽에서 빌드되는지 별도 검증 필요 (Spike).

채택 비채택 (logic-level only) 시:
- AC-R-1/2 는 element tree 를 in-memory data structure 로 추적하는 우회 (예: `render_pane_tree_spec` 함수가 `LayoutSpec` enum 반환 → unit test).
- AC-R-5 의 boundary rejection 은 `GpuiDivider::on_drag` 단위 (이미 AC-P-6 으로 검증됨) + RootView wire-up 의 logic-level 재현 (mock cx).

USER-DECISION 시점: MS-1 진입 직전 ([USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-004]).

권장: **채택 (option a)**. 이유:
- render layer SPEC 의 본질 = 실제 화면 검증.
- test-support 미채택 시 logic-level 대체 코드량이 오히려 더 늘어남.
- 1 줄 Cargo.toml 변경, additive feature.
- carry-over AC-P-5 동시 해소.

---

## 7. 의존성 고고학 — gpui-component 미채택 재확인

SPEC-V3-003 Plan Spike 1 결과: GPUI 0.2.2 native 의 `interactivity()` API 가 divider drag 에 충분 (Spike 1 PASS, Spike 2 무효). 본 SPEC 은 그 결정을 그대로 상속한다 — gpui-component longbridge 의존성 없음.

### 7.1 gpui Keystroke 의 안정성

Context7 query "claude-rs/gpui Keystroke API" 시도가 GLM 호환성 이슈로 fallback 필요. zed/gpui main 브랜치는 `gpui::Keystroke` 를 v0.2 → v0.3 타겟으로 변경 중이지만, 본 SPEC 의 0.2.2 pin 은 변경 없음 (`@MX:WARN` 이미 명시).

---

## 8. 재사용 자산 (변경 금지 vs 변경 대상)

### 8.1 변경 금지 (FROZEN — Terminal Core 무변경 원칙 상속)

- `crates/moai-studio-terminal/**` 전체 (SPEC-V3-002 RG-V3-002-1 그대로).
- `crates/moai-studio-ui/src/terminal/**` (TerminalSurface render API 재사용만).
- `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}` 의 **공개 API**. 내부 구현 (private helper) 은 필요 시 추가만 허용.
- `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` 의 **공개 API**. private helper 추가는 허용.
- `crates/moai-studio-workspace/src/persistence.rs` (SPEC-V3-003 MS-3 산출, 본 SPEC 시점 무관).

### 8.2 변경 대상

- `crates/moai-studio-ui/src/lib.rs:72-99` `RootView` 정의 → `pane_splitter` 필드를 `tab_container: Option<Entity<TabContainer>>` 로 교체.
- `crates/moai-studio-ui/src/lib.rs:170-202` `Render for RootView` → `tab_container` 분기로 교체, key event handler 등록.
- `crates/moai-studio-ui/src/lib.rs:294-308` `main_body`, `content_area` 시그니처 + 본문 변경.
- `crates/moai-studio-ui/Cargo.toml` `dev-dependencies` 에 `gpui` test-support feature 추가 (USER-DECISION PASS 시).

### 8.3 신규

- `crates/moai-studio-ui/src/tabs/container.rs` 에 `impl Render for TabContainer` 추가 (또는 별도 `tabs/render.rs` 모듈로 분리 — MS-1 시점 결정).
- `crates/moai-studio-ui/src/panes/render.rs` (신규) — `render_pane_tree<L: Render + 'static>(...)` 재귀 렌더 함수.
- `crates/moai-studio-ui/src/panes/divider.rs` 에 `impl Render for GpuiDivider` 또는 `fn divider_element(...)` 추가.
- `crates/moai-studio-ui/src/tabs/keys.rs` 에 `keystroke_to_tab_key(ks: &gpui::Keystroke) -> Option<(KeyModifiers, TabKeyCode)>` 신규.
- `crates/moai-studio-ui/tests/integration_render.rs` 신규 — TestAppContext 기반 (USER-DECISION PASS 가정).

---

## 9. 테스트 전략 분석

### 9.1 logic-level (test-support 무관)

이미 SPEC-V3-003 에서 작성된 자산을 본 SPEC 변경이 깨뜨리지 않음을 보증.
- `cargo test -p moai-studio-ui --lib` 53 unit + bar tests + focus tests 0 regression.
- `cargo test -p moai-studio-ui --test '*'` integration 2 regression 0.
- `cargo test -p moai-studio-terminal` 13 regression 0 (RG-P-7 carry).

### 9.2 render-level (USER-DECISION PASS 시)

`TestAppContext` 패턴 (zed/gpui 참조, 0.2.2 헤더 기반 추정):
```
let cx = TestAppContext::single_threaded();
cx.new_window(|cx| {
    let tab_container = cx.new(|_cx| TabContainer::new());
    let root = cx.new(|cx| RootView::with_tab_container(cx, tab_container.clone()));
    // simulate cmd+t keystroke
    // assert tab_container.read(cx).tab_count() == 2
});
```

`TestAppContext::simulate_keystroke(...)` 가 0.2.2 에 존재하는지 Spike 1 에서 검증. 부재 시 RootView 의 handle method 를 직접 호출하는 hybrid (key dispatch 까지는 unit, 그 이후는 integration).

### 9.3 manual smoke test (MS-3 의 핵심 검증)

`cargo run -p moai-studio-app` 으로 실제 윈도우 띄우기:
- §1.3 의 5 가지 사용자 가시 동작을 사람 눈으로 확인.
- 자동화 어렵지만 SPEC PASS 의 최종 escape hatch.

---

## 10. 위험 요약 (spec.md §12 로 승계)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-R1 | gpui Entity API 학습 곡선 (Render trait, Context, listener) | 구현 지연, 잘못된 패턴 | TerminalSurface impl Render (이미 prod) reference, MS-1 spike 1 ≤ 4h budget |
| R-R2 | gpui test-support feature CI 빌드 실패 | 테스트 환경 분기 | spike 0 = test-support feature 빌드 검증 (≤ 1h), 실패 시 logic-level fallback |
| R-R3 | divider drag GPUI API 가 0.2.2 에서 빈약 | AC-R-5 검증 어려움 | SPEC-V3-003 Spike 1 결과 PASS (interactivity API 확인) — 그 코드 패턴 재사용 |
| R-R4 | TabContainer.update 호출 시 PaneTree mutation 배선 복잡 | 이벤트 → 상태 변경 누락 | 명확한 update 경로 (Section 4.3) + per-event 단위 테스트 |
| R-R5 | render layer 도입이 logic-level test 를 깨뜨림 | regression | RG-P-7 무변경 원칙을 본 SPEC RG-R-? 로 상속 |
| R-R6 | Cmd/Ctrl modifier 키 바인딩 충돌 (다른 OS-level 단축키) | UX 이슈 | SPEC-V3-003 AC-P-9a/9b 와 동일 정책, 회귀 0 |

---

## 11. 결정 요약 (Decisions)

| ID | 결정 | 근거 |
|----|------|------|
| D-R1 | gpui-component 미도입 유지 | SPEC-V3-003 Spike 1 PASS 결과 상속 |
| D-R2 | RootView 필드 rename: `pane_splitter` → `tab_container: Option<Entity<TabContainer>>` | SPEC-V3-003 spec.md §9.2 가 사실상 본 SPEC 의 정의를 미리 확정 |
| D-R3 | `impl Render for TabContainer` — 단일 entity 가 활성 탭의 PaneTree + 탭 바를 모두 렌더 | GPUI Entity 단일 책임 패턴, RootView 가 `Entity<TabContainer>` 만 보유 |
| D-R4 | `render_pane_tree<L>` 재귀 함수, `L: Render + 'static` 제약 | Generic 유지 — prod L=`Entity<TerminalSurface>`, test L=`Entity<TestPane>` |
| D-R5 | Cmd/Ctrl 키 dispatch 는 RootView 의 `Window::on_key_down` 에서 처리 후 `tab_container.update(...)` 로 전파 | GPUI 0.2.2 stable 경로, Action macro 보다 단순 |
| D-R6 | SPEC-V3-003 AC-P-4 의 render-layer 해소를 본 SPEC AC-R-5 로 정의 | spec.md §9 carry-over 명시 |
| D-R7 | gpui test-support 는 USER-DECISION 게이트로 MS-1 진입 직전 결정 | 비채택 시 fallback path 명시 |

---

## 12. 시작 시 알아야 할 5 가지

1. SPEC-V3-003 의 `RootView::pane_splitter` 는 임시 명명. 본 SPEC 은 그 필드를 진짜 `tab_container` 로 교체하는 것이 핵심.
2. PaneTree 와 TabContainer 는 이미 잘 작동하는 logic 자료구조다 — 본 SPEC 은 그것을 화면에 그리는 작업이고, **logic 변경은 최소화**.
3. AC-R-5 (divider drag 통합) 가 SPEC-V3-003 AC-P-4 의 직접 승계자. 그 carry-over 가 본 SPEC PASS 의 가장 중요한 시그널.
4. gpui-test-support feature 채택은 USER-DECISION. 채택 시 비용 1 줄, 가치 큼. 비채택 시 우회 경로 존재.
5. GPUI 0.2.2 의 API 안정성 위험 (R-R1) 은 SPEC-V3-002 부터 누적된 기존 위험이며, 본 SPEC 은 해당 위험을 추가로 키우지 않는다 (오직 stable 경로만 사용).

---

작성 완료: 2026-04-25
다음: spec.md (canonical contract), plan.md (execution table)
