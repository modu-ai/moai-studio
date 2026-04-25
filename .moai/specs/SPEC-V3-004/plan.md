# SPEC-V3-004 Implementation Plan

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render`
범위: SPEC-V3-004 spec.md MS-1 / MS-2 / MS-3 + USER-DECISION 게이트.
선행: SPEC-V3-003 종결 (53 unit + 2 integration tests, AC-P-4/AC-P-5 carry-over closed-with-deferral).

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|------|----------------------|-----|----|
| **T1** | MS-1 | USER-DECISION + Spike 0 | `Cargo.toml` (가능 시), Spike 보고서 inline | — | (게이트) AC-R-6 |
| **T2** | MS-1 | RootView 필드 교체 | `crates/moai-studio-ui/src/lib.rs:72-99` | T1 | AC-R-1 |
| **T3** | MS-1 | impl Render for TabContainer | `crates/moai-studio-ui/src/tabs/container.rs` (Render trait 추가) | T2 | AC-R-1 |
| **T4** | MS-2 | render_pane_tree 재귀 함수 | `crates/moai-studio-ui/src/panes/render.rs` (신규), `panes/mod.rs` re-export | T3 | AC-R-2, AC-R-7 |
| **T5** | MS-2 | keystroke_to_tab_key + 키 dispatch wire | `crates/moai-studio-ui/src/tabs/keys.rs`, `crates/moai-studio-ui/src/lib.rs` (key handler 등록) | T3 | AC-R-3, AC-R-4 |
| **T6** | MS-2 | TabBar GPUI element 변환 | `crates/moai-studio-ui/src/tabs/bar.rs` (helper 추가, 기존 logic 무변경) | T3 | AC-R-1 추가 검증 |
| **T7** | MS-3 | divider element + drag 핸들러 | `crates/moai-studio-ui/src/panes/divider.rs` (helper 추가), render.rs 자식 배치 | T4, T5 | AC-R-5 |
| **T8** | MS-3 | AC-R-5 / AC-R-7 통합 검증 | `crates/moai-studio-ui/tests/integration_render.rs` (신규, USER-DECISION 결과에 따라 fallback) | T7 | AC-R-5, AC-R-7 |
| **T9** | 전체 | regression + smoke test + commit | (git 작업, progress.md 갱신) | T1~T8 | AC-R-8 |

---

## 2. T1 — USER-DECISION + Spike 0 (gpui test-support feature)

### 2.1 USER-DECISION 호출

[USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-004]

질문 (AskUserQuestion):
- "gpui crate 의 test-support feature 를 dev-dependencies 에 추가하시겠습니까?"
- (a) **권장: 추가**. 비용 1 줄, AC-R-2/3/4/5 가 실제 GPUI 환경에서 검증됨.
- (b) 추가하지 않음. logic-level fallback 으로 진행. 우회 코드 약 100-150 LOC 추가.

### 2.2 Spike 0 (option a 채택 시)

- `Cargo.toml` `dev-dependencies` 의 `criterion = "0.5"` 라인 옆에 `gpui = { version = "0.2", features = ["test-support"] }` 추가.
- `cargo test -p moai-studio-ui --no-run` 으로 빌드 통과 검증 (macOS 로컬).
- Linux CI 빌드 검증 — `feature/SPEC-V3-004-render` 의 첫 push 가 검증 자체.
- 빌드 실패 시: 자동으로 option (b) 로 fallback, progress.md 에 기록.

### 2.3 Option (b) 선택 시

- Cargo.toml 변경 없음.
- `tests/integration_render.rs` 는 logic-level wrapper 로 작성. 예: RootView 의 handle_key 메서드를 직접 호출하는 unit-style integration.
- progress.md 의 USER-DECISION 항목에 비채택 사실 + 사유 + 우회 전략 기록.

---

## 3. T2 — RootView 필드 교체

### 3.1 변경 대상

`crates/moai-studio-ui/src/lib.rs:72-99` 의 `RootView` 구조체:

Before:
```
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    pub pane_splitter: Option<Entity<terminal::TerminalSurface>>,  // ← legacy
}
```

After:
```
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    // @MX:ANCHOR: [AUTO] root-view-tab-container-binding
    // @MX:REASON: [AUTO] SPEC-V3-004 RG-R-1. tab_container 는 content_area 진입점이며
    //   key dispatch (RG-R-4) 와 divider drag (RG-R-3) 의 mutation target 이다.
    //   fan_in >= 3: T2 init, T5 key handler, T7 divider drag.
    pub tab_container: Option<Entity<tabs::TabContainer>>,
}
```

### 3.2 영향 범위

- `RootView::new` 시그니처: 동일 유지, `tab_container: None` 으로 초기화.
- `apply_added_workspace`, `activate_workspace`, `handle_*_workspace`: 변경 없음.
- `Render::render` 의 `pane_splitter.clone()` 호출 → `tab_container.clone()` 로 교체.
- `main_body` 와 `content_area` 시그니처: `Option<Entity<TerminalSurface>>` → `Option<Entity<TabContainer>>` 로 교체.
- 기존 7 unit tests 의 `view.pane_splitter` 검증 → `view.tab_container` 검증으로 rename.
- 신규 unit test 1: `tab_container_is_none_by_default`.

### 3.3 AC 매핑

- AC-R-1 (a) 부분: tab_container 가 Some 으로 초기화 가능함을 unit test 로 검증.

---

## 4. T3 — impl Render for TabContainer

### 4.1 신규 코드 위치

`crates/moai-studio-ui/src/tabs/container.rs` 하단에 Render impl 추가 (OD-R2: 인라인 채택).

```
impl gpui::Render for TabContainer {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        // 1) TabBar element 생성 (T6 helper 활용)
        let labels: Vec<&str> = self.tabs.iter().map(|t| t.title.as_str()).collect();
        let bar_items = TabBar::items(&labels, self.active_tab_idx);
        let bar = render_tab_bar(&bar_items);

        // 2) 활성 탭의 PaneTree 렌더 (T4 의 render_pane_tree 활용)
        let body = render_pane_tree(&self.active_tab().pane_tree);

        gpui::div()
            .flex()
            .flex_col()
            .size_full()
            .child(bar)
            .child(body)
    }
}
```

### 4.2 의존

- T4 (render_pane_tree) 가 MS-2 에 있으므로, T3 의 MS-1 시점에서는 placeholder body (단일 leaf 의 string 표시) 로 시작 → T4 완료 시 render_pane_tree 호출로 교체.

### 4.3 AC 매핑

- AC-R-1 (b) 부분: tab_container 의 render 가 panic 없이 동작.
- AC-R-1 추가: 빈 PaneTree 또는 단일 leaf 의 가시 렌더.

### 4.4 신규 단위 테스트

- `tabs::container::tests::tab_container_render_does_not_panic_on_single_leaf` (TestAppContext 기반 또는 logic-level wrapper).

---

## 5. T4 — render_pane_tree 재귀 함수

### 5.1 신규 파일

`crates/moai-studio-ui/src/panes/render.rs`:

```
//! PaneTree → GPUI element tree 재귀 변환 (SPEC-V3-004 RG-R-2).

use crate::panes::{PaneTree, SplitDirection};
use gpui::{IntoElement, ParentElement, Styled, div};

// @MX:ANCHOR: [AUTO] render-pane-tree-recursion
// @MX:REASON: [AUTO] SPEC-V3-004 REQ-R-010 ~ REQ-R-014. PaneTree → GPUI 변환 진입점.
//   fan_in >= 3: TabContainer.render, integration_render tests, 향후 PTY-per-pane SPEC.

pub fn render_pane_tree<L>(tree: &PaneTree<L>) -> impl IntoElement
where
    L: gpui::Render + 'static,
{
    match tree {
        PaneTree::Leaf(leaf) => render_leaf(leaf),
        PaneTree::Split { direction, ratio, first, second, .. } => {
            match direction {
                SplitDirection::Horizontal => div()
                    .flex().flex_row().size_full()
                    .child(render_pane_tree(first.as_ref()))
                    .child(divider_vertical())
                    .child(render_pane_tree(second.as_ref())),
                SplitDirection::Vertical => div()
                    .flex().flex_col().size_full()
                    .child(render_pane_tree(first.as_ref()))
                    .child(divider_horizontal())
                    .child(render_pane_tree(second.as_ref())),
            }
        }
    }
}
```

### 5.2 AC 매핑

- AC-R-2: 1 회 split 시 element tree 자식 수 검증 (leaf 2 + divider 1).
- AC-R-7: 3 level split 시 divider element 정확히 3 개 (split 노드 수와 일치).

### 5.3 단위 테스트

USER-DECISION (a) 채택 시: TestAppContext 로 element tree 탐색.
USER-DECISION (b) 채택 시: render_pane_tree 가 반환하는 element 의 spec-only structural representation 을 검증하는 helper (예: `fn count_dividers_in_spec(tree: &PaneTree<...>) -> usize`).

신규 테스트:
- `panes::render::tests::single_leaf_returns_no_divider`
- `panes::render::tests::single_horizontal_split_emits_one_vertical_divider`
- `panes::render::tests::three_level_split_emits_three_dividers` (AC-R-7)

---

## 6. T5 — keystroke_to_tab_key + RootView key dispatch

### 6.1 keystroke_to_tab_key 신규 함수

`crates/moai-studio-ui/src/tabs/keys.rs` 하단:

```
/// gpui::Keystroke → (KeyModifiers, TabKeyCode) 변환.
///
/// REQ-R-030. RootView::handle_key_down 이 단독 호출자 (fan_in == 1, 향후 확장 가능).
pub fn keystroke_to_tab_key(ks: &gpui::Keystroke) -> (KeyModifiers, TabKeyCode) {
    let mods = KeyModifiers {
        cmd: ks.modifiers.command,   // gpui 0.2.2 의 정확한 필드명은 spike 1 에서 확인
        ctrl: ks.modifiers.control,
        shift: ks.modifiers.shift,
        alt: ks.modifiers.alt,
    };
    let code = match ks.key.as_str() {
        "t" | "T" => TabKeyCode::T,
        "1" => TabKeyCode::Digit(1), "2" => TabKeyCode::Digit(2),
        "3" => TabKeyCode::Digit(3), "4" => TabKeyCode::Digit(4),
        "5" => TabKeyCode::Digit(5), "6" => TabKeyCode::Digit(6),
        "7" => TabKeyCode::Digit(7), "8" => TabKeyCode::Digit(8),
        "9" => TabKeyCode::Digit(9),
        "\\" => TabKeyCode::Backslash,
        "{" => TabKeyCode::BraceOpen,
        "}" => TabKeyCode::BraceClose,
        _ => TabKeyCode::Other,
    };
    (mods, code)
}
```

### 6.2 RootView 의 key handler 등록

`lib.rs` 의 Render::render 또는 `run_app` 의 윈도우 생성 시점에 `Window::on_key_down` (또는 그에 준하는 GPUI 0.2.2 API) 핸들러 등록.

```
// in run_app 또는 RootView::render 의 div() chain 에:
.on_key_down(cx.listener(|this: &mut RootView, ev: &gpui::KeyDownEvent, _window, cx| {
    let (mods, code) = tabs::keys::keystroke_to_tab_key(&ev.keystroke);
    if let Some(cmd) = tabs::keys::dispatch_tab_key(mods, code) {
        this.handle_tab_command(cmd, cx);
    }
    // None 인 경우 keystroke 는 활성 leaf 로 자동 forward
}))
```

### 6.3 RootView::handle_tab_command (신규 메서드)

```
fn handle_tab_command(&mut self, cmd: tabs::TabCommand, cx: &mut Context<Self>) {
    let Some(tc) = self.tab_container.as_ref() else { return; };
    tc.update(cx, |tc, cx| {
        match cmd {
            TabCommand::NewTab => { tc.new_tab(None); }
            TabCommand::SwitchToTab(idx) => { tc.switch_tab(idx).ok(); }
            TabCommand::SplitVertical => {
                // 활성 탭의 focused leaf 에 split_horizontal 적용
                if let Some(focused) = tc.active_tab().last_focused_pane.clone() {
                    tc.active_tab_mut().pane_tree.split_horizontal(focused).ok();
                }
            }
            TabCommand::SplitHorizontal => {
                if let Some(focused) = tc.active_tab().last_focused_pane.clone() {
                    tc.active_tab_mut().pane_tree.split_vertical(focused).ok();
                }
            }
            TabCommand::PrevTab => {
                if tc.active_tab_idx > 0 { tc.switch_tab(tc.active_tab_idx - 1).ok(); }
            }
            TabCommand::NextTab => {
                let next = tc.active_tab_idx + 1;
                if next < tc.tabs.len() { tc.switch_tab(next).ok(); }
            }
        }
        cx.notify();
    });
}
```

### 6.4 AC 매핑

- AC-R-3: Cmd/Ctrl+T → tabs.len() == 2.
- AC-R-4: Cmd/Ctrl+\\ → 활성 탭 PaneTree 가 Split 으로 교체 + element tree 에 divider 1 개 추가.

### 6.5 신규 단위 테스트

- `tabs::keys::tests::keystroke_t_with_cmd_returns_t_keycode`
- `tabs::keys::tests::keystroke_digit_1_with_ctrl_returns_digit_1`
- `tabs::keys::tests::keystroke_unknown_returns_other`
- (logic-level 통합) `lib::tests::handle_tab_command_new_tab_increments_count` — RootView 가 mock cx 환경에서 동작 확인.

---

## 7. T6 — TabBar GPUI element 변환

### 7.1 helper 추가

`crates/moai-studio-ui/src/tabs/bar.rs` 하단:

```
/// TabBarItem 목록을 GPUI element 로 렌더.
///
/// SPEC-V3-004 T6: tabs::bar 모듈은 logic-only 였으나 본 SPEC 에서 GPUI element 변환 helper 추가.
/// 기존 TabBar::items 는 무변경, 본 helper 가 그 결과를 소비한다.
pub fn render_tab_bar(items: &[TabBarItem]) -> impl gpui::IntoElement {
    use gpui::{div, rgb, ParentElement, Styled};
    let mut bar = div().flex().flex_row().h(gpui::px(32.0)).w_full();
    for item in items {
        let bg_color = if item.is_active { 0x232327 } else { 0x131315 };
        let weight = if item.is_active { /* gpui FontWeight::Bold equivalent */ };
        bar = bar.child(
            div()
                .px_3().py_1()
                .bg(rgb(bg_color))
                .child(item.label.clone())
        );
    }
    bar
}
```

(정확한 GPUI font-weight 변환은 MS-1 spike 시 확정.)

### 7.2 AC 매핑

- AC-R-1 추가 검증: 탭 바가 활성 탭 indicator (background color + bold) 를 시각적으로 구분.
- SPEC-V3-003 AC-P-27 의 render-layer 검증.

### 7.3 신규 단위 테스트

- (USER-DECISION (a) 시) integration_render: 탭 바 element 의 active 탭 색상 검증.
- 기존 logic test (`active_indicator_is_bold`, `inactive_uses_toolbar_background_token`) 0 regression.

---

## 8. T7 — divider element + drag 핸들러

### 8.1 divider element helper

`crates/moai-studio-ui/src/panes/divider.rs` 하단 (logic 무변경, helper 추가):

```
/// 수직 divider element (Horizontal split 의 좌/우 사이) — SPEC-V3-004 REQ-R-014, REQ-R-020.
pub fn divider_vertical_element(node_id: SplitNodeId) -> impl gpui::IntoElement {
    use gpui::{div, rgb, ParentElement, Styled, MouseButton};
    div()
        .id(format!("div-v-{:x}", /* node_id hash */))
        .w(gpui::px(4.0))
        .h_full()
        .bg(rgb(0x2a2a2e))
        .hover(|s| s.bg(rgb(0x3a3a40)))
        .cursor(gpui::CursorStyle::ResizeLeftRight)
        // drag handler — T7 main 에서 RootView listener 와 결합
}
```

### 8.2 RootView drag state 필드

```
pub struct RootView {
    // ...
    /// 활성 divider drag 상태 (REQ-R-024 동안 다른 이벤트 suppressed).
    pub drag_state: Option<DividerDragState>,
}

pub struct DividerDragState {
    pub split_node_id: SplitNodeId,
    pub orientation: SplitDirection,
    pub start_xy: (f32, f32),
    pub initial_ratio: f32,
    pub total_px: f32,
}
```

### 8.3 mouse_down / mouse_move / mouse_up 핸들러

```
divider_element.on_mouse_down(MouseButton::Left, cx.listener(|this, ev, _w, cx| {
    this.drag_state = Some(DividerDragState { ... });
    cx.notify();
}))
.on_mouse_move(cx.listener(|this, ev, _w, cx| {
    if let Some(ds) = &this.drag_state {
        let delta_px = match ds.orientation {
            SplitDirection::Horizontal => ev.position.x.0 - ds.start_xy.0,
            SplitDirection::Vertical   => ev.position.y.0 - ds.start_xy.1,
        };
        let mut divider = panes::GpuiDivider::new(ds.orientation);
        let new_ratio = divider.on_drag(delta_px, ds.total_px);
        this.tab_container.as_ref().map(|tc| tc.update(cx, |tc, cx| {
            tc.active_tab_mut().pane_tree.set_ratio(&ds.split_node_id, new_ratio).ok();
            cx.notify();
        }));
    }
}))
.on_mouse_up(MouseButton::Left, cx.listener(|this, _ev, _w, cx| {
    this.drag_state = None;
    cx.notify();
}));
```

### 8.4 AC 매핑

- AC-R-5: divider drag → set_ratio → boundary rejection 통합 검증.
- AC-R-7: split 노드 당 divider 1 개 유지.

---

## 9. T8 — AC-R-5 / AC-R-7 통합 검증

### 9.1 신규 통합 테스트 파일

`crates/moai-studio-ui/tests/integration_render.rs`:

```
//! SPEC-V3-004 RG-R-3, RG-R-5 통합 테스트.

#[cfg(feature = "gpui-test-support")]  // USER-DECISION 결과 (a) 시
mod gpui_integration {
    use moai_studio_ui::*;
    use gpui::TestAppContext;

    #[test]
    fn divider_drag_clamps_to_min_cols_at_render_layer() {
        let mut cx = TestAppContext::single_threaded();
        cx.new_window(|cx| {
            let tc = cx.new(|_| TabContainer::new());
            tc.update(cx, |tc, _| {
                let leaf_id = tc.active_tab().pane_tree.first_leaf_id();
                tc.active_tab_mut().pane_tree.split_horizontal(leaf_id).unwrap();
            });
            let root = cx.new(|_| RootView::with_tab_container(tc.clone(), test_path()));
            // simulate mouse down/move/up on divider
            // assert ratio clamped at MIN_COLS boundary
        });
    }
}

#[cfg(not(feature = "gpui-test-support"))]
mod logic_fallback {
    // RootView::handle_tab_command 직접 호출 + GpuiDivider::on_drag 결과를 통합 환경에서 검증
    #[test]
    fn divider_on_drag_clamps_at_min_cols_logic_level() {
        // ... GpuiDivider unit + RootView state mutation 결합 테스트
    }
}
```

### 9.2 AC 매핑

- AC-R-5 PASS 시 SPEC-V3-003 AC-P-4 carry-over 공식 해소.
- AC-R-6 PASS — USER-DECISION 결과를 progress.md 에 기록.

---

## 10. T9 — Regression + Smoke + Commit

### 10.1 Regression 검증 명령

```
cargo test -p moai-studio-terminal --all-targets    # SPEC-V3-002, 13 tests, 0 regression
cargo test -p moai-studio-workspace --all-targets   # SPEC-V3-001/003 persistence, 0 regression
cargo test -p moai-studio-ui --lib                  # 53 unit + new render tests, 0 regression
cargo test -p moai-studio-ui --test '*'             # integration_render + 기존 통합 테스트
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

### 10.2 Smoke test

`cargo run -p moai-studio-app` 실행 후 §1.3 의 5 가지 사용자 가시 동작 확인 (수동, screenshot 또는 메모만).

### 10.3 Commit 전략

본 SPEC 의 plan 단계 산출 (research/plan/spec) 은 단일 commit 으로 처리한다 (사용자 지시).

implementation 단계 (T1~T8) 는 별도 SPEC run 단계에서 milestone 별 commit:
- MS-1 commit: T1-T3 (USER-DECISION + RootView field + impl Render)
- MS-2 commit: T4-T6 (render_pane_tree + key dispatch + tab bar element)
- MS-3 commit: T7-T8 (divider drag + AC-R-5 검증)
- T9 commit: regression + smoke + progress.md final.

---

## 11. Hard Thresholds (sprint exit 전제, SPEC-V3-003 carry)

- [ ] Coverage ≥ 85% per commit (UI render layer 목표)
- [ ] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warning
- [ ] `cargo fmt --all -- --check` 통과
- [ ] SPEC-V3-002 regression 0 (13/13)
- [ ] SPEC-V3-003 logic tests regression 0 (53 unit + 2 integration)
- [ ] 신규 render tests ≥ 5 unit + 1 integration (USER-DECISION 결과 무관)
- [ ] @MX 태그: ANCHOR ≥ 2 신규 (root-view-tab-container-binding, render-pane-tree-recursion), WARN ≥ 0 (gpui-api-churn-risk 는 splitter_gpui_native.rs 에 이미 존재), NOTE ≥ 2

---

## 12. Escalation Protocol

- gpui Render trait 학습이 4h spike 안에 완료되지 않으면 → AskUserQuestion 으로 시간 추가 vs scope 축소 결정.
- TestAppContext 의 simulate_keystroke API 가 0.2.2 에 부재 시 → option (b) logic-level fallback 자동 전환.
- divider drag 의 GPUI mouse 이벤트 API 가 SPEC-V3-003 Spike 1 결과와 다를 시 → spec.md OD-R5 갱신 + plan revision.
- AC-R-5 가 logic-level 로도 검증 불가 시 → SPEC-V3-003 AC-P-4 의 carry-over 가 본 SPEC 에서도 해소되지 않음을 progress.md 에 명시 + 재차 별도 SPEC 으로 분리 (escape hatch 의 escape hatch).

---

## 13. Sprint Exit Criteria (SPEC-V3-004 → 종결 gate)

- AC-R-1 ~ AC-R-8 전원 GREEN
- SPEC-V3-003 carry-over: AC-P-4 → AC-R-5 + AC-R-7 로 직접 해소, AC-P-5 → AC-R-6 USER-DECISION 결과로 해소
- Hard thresholds 전원 통과
- T9 commit 완료, progress.md SPEC complete 기록 (별도 progress.md 는 implementation 단계 산출, 본 plan 단계에서는 미생성)

---

작성: 2026-04-25
브랜치: `feature/SPEC-V3-004-render`
다음: implementation phase (`/moai run SPEC-V3-004`)
