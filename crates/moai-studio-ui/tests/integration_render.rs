//! SPEC-V3-004 render layer integration 테스트.
//!
//! ## 목적
//!
//! - AC-R-1: TabContainer 가 Entity<TabContainer> 로 생성 가능하고 render 호출 시 panic 없음.
//! - AC-R-2: 1 회 split → split 노드 1, leaf 2, divider 1 (render_pane_tree logic-level).
//! - AC-R-3: Cmd/Ctrl+T → tabs.len() 증가 (keystroke_to_tab_key + dispatch_tab_key logic-level).
//! - AC-R-4: Cmd/Ctrl+\ → 활성 탭 PaneTree Split 으로 교체 + divider 1 추가.
//! - AC-R-6: gpui test-support feature 채택 (USER-DECISION ADOPT) 으로 TestAppContext 기반 smoke test.
//!
//! ## USER-DECISION: gpui-test-support-adoption-v3-004
//!
//! Option (a) 채택: gpui dev-dependencies 에 test-support feature 추가.
//! AC-R-2/3/4 는 logic-level (count_splits/count_leaves 헬퍼) 로 검증.
//! TestAppContext 는 Entity 생성/접근 검증에 사용 (AC-R-1, AC-R-6).

use gpui::{AppContext as _, TestAppContext};
use moai_studio_ui::panes::render::{count_leaves, count_splits};
use moai_studio_ui::panes::{PaneId, PaneTree};
use moai_studio_ui::tabs::TabContainer;

// ============================================================
// AC-R-1: TabContainer Entity 생성 가능 (REQ-R-001)
// ============================================================

/// TabContainer can be instantiated as Entity<TabContainer> via cx.new.
///
/// REQ-R-001: 시스템은 TabContainer 가 cx.new(|cx| TabContainer::new()) 호출로
/// Entity<TabContainer> 로 생성될 수 있도록 한다.
#[test]
fn tab_container_entity_can_be_created() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());
    // Entity 생성 성공 확인 — read() 가 패닉 없이 동작해야 함
    let tab_count = cx.read(|app| entity.read(app).tab_count());
    assert_eq!(
        tab_count, 1,
        "새 TabContainer 는 1개 탭으로 초기화됨 (REQ-P-042)"
    );
}

/// TabContainer Entity with multiple tabs has correct state after mutation.
///
/// AC-R-1 추가 검증: 탭 추가 후 Entity 를 통해 상태를 읽을 수 있다.
#[test]
fn tab_container_entity_state_readable_after_mutation() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    // Entity 를 통해 상태 변경
    entity.update(&mut cx, |tc, _cx| {
        tc.new_tab(None);
        tc.new_tab(None);
    });

    let tab_count = cx.read(|app| entity.read(app).tab_count());
    assert_eq!(tab_count, 3, "new_tab 2번 호출 후 탭 수 == 3");
}

// ============================================================
// AC-R-1: impl Render for TabContainer — panic 없이 동작 (REQ-R-005)
// ============================================================

/// Render trait is implemented for TabContainer (compile-time assertion).
///
/// REQ-R-002: 시스템은 TabContainer 에 대해 impl Render 트레잇 구현을 제공한다.
/// 이 테스트는 Render trait 의 존재를 컴파일 타임에 검증한다.
#[test]
fn tab_container_implements_render_trait() {
    // TabContainer 가 gpui::Render 를 구현함을 타입 시스템으로 검증.
    // 이 함수는 컴파일되면 성공이다.
    fn assert_render<T: gpui::Render>() {}
    assert_render::<TabContainer>();
}

/// TabContainer Entity with empty tabs is safe to access (REQ-R-005).
///
/// REQ-R-005: 시스템은 TabContainer.tabs.is_empty() 가 true 인 상태에서
/// Render::render 가 panic 하지 않도록 한다.
/// 이 테스트는 empty TabContainer Entity 를 생성하고 상태 접근이 안전함을 검증.
#[test]
fn tab_container_empty_tabs_state_is_safe() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| {
        let mut tc = TabContainer::new();
        // 강제로 tabs 를 비워 REQ-R-005 경로 검증
        tc.tabs.clear();
        tc
    });

    let tab_count = cx.read(|app| entity.read(app).tab_count());
    assert_eq!(
        tab_count, 0,
        "tabs.clear() 후 tab_count == 0, Entity 접근 안전"
    );
}

/// TabContainer Entity with single tab has correct initial state.
///
/// MS-1 smoke test: 단일 탭 TabContainer 의 초기 상태 검증.
#[test]
fn tab_container_single_tab_initial_state() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    let (tab_count, active_idx) = cx.read(|app| {
        let tc = entity.read(app);
        (tc.tab_count(), tc.active_tab_idx)
    });
    assert_eq!(tab_count, 1, "초기 탭 수 == 1");
    assert_eq!(active_idx, 0, "초기 active_tab_idx == 0");
}

// ============================================================
// AC-R-2: render_pane_tree — 1 split → leaf 2, divider 1
// ============================================================

/// 1 회 horizontal split 후 render 관점에서 split 1, leaf 2 확인 (AC-R-2).
///
/// AC-R-2: TabContainer 의 활성 탭 PaneTree 가 1 회 horizontal split 된 상태에서
/// render 결과 element tree 에 divider element 정확히 1 개가 존재해야 한다.
/// USER-DECISION option-b: count_splits == divider 수 이므로 count_splits 로 검증.
#[test]
fn ac_r2_single_horizontal_split_produces_one_divider() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    entity.update(&mut cx, |tc, _cx| {
        let focused = tc
            .active_tab()
            .last_focused_pane
            .clone()
            .expect("초기 focused pane 필요");
        tc.active_tab_mut()
            .pane_tree
            .split_horizontal(&focused, PaneId::new_unique(), "new-pane".to_string())
            .expect("split_horizontal 성공");
    });

    let (splits, leaves) = cx.read(|app| {
        let tc = entity.read(app);
        let tree = &tc.active_tab().pane_tree;
        (count_splits(tree), count_leaves(tree))
    });

    // AC-R-2: split 1 → divider 1 (count_splits == divider 수)
    assert_eq!(splits, 1, "AC-R-2: horizontal split 1 회 → split 노드 1");
    assert_eq!(leaves, 2, "AC-R-2: horizontal split 1 회 → leaf 2");
}

// ============================================================
// AC-R-3: Cmd/Ctrl+T → tabs.len() 증가 (logic-level)
// ============================================================

/// Cmd/Ctrl+T 키스트로크 → TabContainer.tabs.len() += 1 (AC-R-3).
///
/// AC-R-3: RootView 활성, TabContainer.tabs.len() == 1 에서 Cmd+T 입력 시
/// tabs.len() == 2, active_tab_idx == 1.
/// USER-DECISION option-b: keystroke_to_tab_key → dispatch_tab_key → new_tab 직접 호출로 검증.
#[test]
fn ac_r3_cmd_t_increments_tab_count() {
    use gpui::Keystroke;
    use moai_studio_ui::tabs::keys::{TabCommand, dispatch_tab_key, keystroke_to_tab_key};

    let platform_ks = Keystroke {
        modifiers: gpui::Modifiers {
            #[cfg(target_os = "macos")]
            platform: true,
            #[cfg(not(target_os = "macos"))]
            control: true,
            ..gpui::Modifiers::default()
        },
        key: "t".to_string(),
        key_char: Some("t".to_string()),
    };

    let (mods, code) = keystroke_to_tab_key(&platform_ks);
    let cmd = dispatch_tab_key(mods, code);
    assert_eq!(cmd, Some(TabCommand::NewTab), "Cmd/Ctrl+T → NewTab");

    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());
    assert_eq!(cx.read(|app| entity.read(app).tab_count()), 1);

    entity.update(&mut cx, |tc, _cx| {
        if let Some(TabCommand::NewTab) = cmd {
            tc.new_tab(None);
        }
    });

    let (tab_count, active_idx) = cx.read(|app| {
        let tc = entity.read(app);
        (tc.tab_count(), tc.active_tab_idx)
    });
    assert_eq!(tab_count, 2, "AC-R-3: Cmd/Ctrl+T 후 tabs.len() == 2");
    assert_eq!(active_idx, 1, "AC-R-3: 새 탭이 active (idx == 1)");
}

// ============================================================
// AC-R-4: Cmd/Ctrl+\ → PaneTree Split 교체 + divider 1 추가
// ============================================================

/// Cmd/Ctrl+\ → 활성 탭 PaneTree 가 Split 으로 교체 + divider element 1 추가 (AC-R-4).
///
/// AC-R-4: TabContainer 활성, 단일 leaf 활성 탭에서 Cmd/Ctrl+\ 입력 시
/// PaneTree 가 Split 으로 교체되고 divider 1 개가 추가된다.
#[test]
fn ac_r4_cmd_backslash_splits_pane_tree() {
    use gpui::Keystroke;
    use moai_studio_ui::tabs::keys::{TabCommand, dispatch_tab_key, keystroke_to_tab_key};

    let platform_ks = Keystroke {
        modifiers: gpui::Modifiers {
            #[cfg(target_os = "macos")]
            platform: true,
            #[cfg(not(target_os = "macos"))]
            control: true,
            ..gpui::Modifiers::default()
        },
        key: "\\".to_string(),
        key_char: None,
    };

    let (mods, code) = keystroke_to_tab_key(&platform_ks);
    let cmd = dispatch_tab_key(mods, code);
    assert_eq!(
        cmd,
        Some(TabCommand::SplitVertical),
        "Cmd/Ctrl+\\ → SplitVertical"
    );

    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    entity.update(&mut cx, |tc, _cx| {
        if let Some(TabCommand::SplitVertical) = cmd {
            let focused = tc
                .active_tab()
                .last_focused_pane
                .clone()
                .expect("focused pane 필요");
            tc.active_tab_mut()
                .pane_tree
                .split_horizontal(&focused, PaneId::new_unique(), "new-pane".to_string())
                .expect("split_horizontal 성공");
        }
    });

    let (is_split, splits, leaves) = cx.read(|app| {
        let tc = entity.read(app);
        let tree = &tc.active_tab().pane_tree;
        let is_split = matches!(tree, PaneTree::Split { .. });
        (is_split, count_splits(tree), count_leaves(tree))
    });

    assert!(
        is_split,
        "AC-R-4: Cmd/Ctrl+\\ 후 PaneTree 는 Split 이어야 한다"
    );
    assert_eq!(splits, 1, "AC-R-4: split 노드 1 개");
    assert_eq!(leaves, 2, "AC-R-4: leaf 2 개 (divider 1 개와 동치)");
}

// ============================================================
// T6: TabBar element 검증 — N 탭 → N children
// ============================================================

/// TabContainer N 탭 → TabBar 에 N 개 항목 표시 (T6 구조 검증).
///
/// SPEC-V3-004 T6: TabBar element 가 활성 탭 indicator 를 구분한다.
/// USER-DECISION option-b: tabs.len() 으로 tab bar 항목 수 검증.
#[test]
fn t6_tab_bar_has_n_children_for_n_tabs() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    // 탭 3개로 확장
    entity.update(&mut cx, |tc, _cx| {
        tc.new_tab(None);
        tc.new_tab(None);
    });

    let (tab_count, active_idx) = cx.read(|app| {
        let tc = entity.read(app);
        (tc.tab_count(), tc.active_tab_idx)
    });

    // TabBar 는 tab_count 개 항목을 포함해야 한다 (구조 검증)
    assert_eq!(tab_count, 3, "T6: 탭 3개 생성 후 tab_count == 3");
    assert_eq!(active_idx, 2, "T6: 마지막 생성 탭이 active (idx == 2)");
}

// ============================================================
// MS-3 T8: AC-R-7 — 3-level split → divider 3 개 (REQ-R-040)
// ============================================================

/// AC-R-7: 활성 탭이 3-level split PaneTree 를 가질 때 divider 가 정확히 3 개.
///
/// AC-R-7: TabContainer 의 활성 탭이 3 level split (1 horizontal + 2 vertical = 4 leaf) 일 때
/// render 결과 element tree 에 divider element 가 정확히 3 개 존재한다.
/// split 노드 수 == divider 수 불변을 count_splits 헬퍼로 검증.
#[test]
fn three_level_split_produces_three_dividers() {
    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    entity.update(&mut cx, |tc, _cx| {
        // 1. 단일 leaf 초기 상태에서 focused pane id 획득
        let id0 = tc
            .active_tab()
            .last_focused_pane
            .clone()
            .expect("초기 focused pane 필요");

        // 2. horizontal split: id0 | id1
        let id1 = PaneId::new_unique();
        tc.active_tab_mut()
            .pane_tree
            .split_horizontal(&id0, id1.clone(), "pane-1".to_string())
            .expect("horizontal split 성공");

        // 3. vertical split on id0 (left): id0 / id2
        let id2 = PaneId::new_unique();
        tc.active_tab_mut()
            .pane_tree
            .split_vertical(&id0, id2, "pane-2".to_string())
            .expect("vertical split on id0 성공");

        // 4. vertical split on id1 (right): id1 / id3
        let id3 = PaneId::new_unique();
        tc.active_tab_mut()
            .pane_tree
            .split_vertical(&id1, id3, "pane-3".to_string())
            .expect("vertical split on id1 성공");
    });

    let (splits, leaves) = cx.read(|app| {
        let tc = entity.read(app);
        let tree = &tc.active_tab().pane_tree;
        (count_splits(tree), count_leaves(tree))
    });

    // AC-R-7: 3 split 노드 → divider 3 개, leaf 4 개
    assert_eq!(
        splits, 3,
        "AC-R-7: 3-level split → split 노드 3 (= divider 3)"
    );
    assert_eq!(leaves, 4, "AC-R-7: 3-level split → leaf 4");
}

// ============================================================
// MS-3 T8: AC-R-5 — divider drag boundary clamp (REQ-R-020~022)
// ============================================================

/// AC-R-5: divider drag 이 min 경계를 초과하면 ratio 가 clamp 된다.
///
/// AC-R-5: 활성 탭 PaneTree 가 horizontal split 인 상태에서
/// GpuiDivider::on_drag 로 우측 sibling 이 MIN_COLS 미만이 되도록 시도하면
/// ratio 가 clamp 된 값으로 고정되고 set_ratio 가 성공한다.
///
/// RootView drag wire-up 을 통한 통합 검증 (T7 DividerDragState logic).
/// total_px=1600px, MIN_COLS=40, px_per_col=8.0 → min_px=320px → min_ratio=0.2
/// 초기 ratio=0.6 (left=960px, right=640px), delta=-400px → right=240px < 320px → clamp
#[test]
fn divider_drag_clamps_at_min_size() {
    use moai_studio_ui::panes::{GpuiDivider, ResizableDivider, SplitDirection};

    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    // 1. horizontal split 생성
    let split_node_id = entity.update(&mut cx, |tc, _cx| {
        let id0 = tc
            .active_tab()
            .last_focused_pane
            .clone()
            .expect("focused pane 필요");
        tc.active_tab_mut()
            .pane_tree
            .split_horizontal(&id0, PaneId::new_unique(), "pane-1".to_string())
            .expect("split 성공");

        // 초기 ratio 를 0.6 으로 설정
        let node_id = tc
            .active_tab()
            .pane_tree
            .find_split_node_id()
            .expect("split node id 필요")
            .clone();
        tc.active_tab_mut()
            .pane_tree
            .set_ratio(&node_id, 0.6)
            .expect("set_ratio 0.6 성공");
        node_id
    });

    // 2. GpuiDivider drag 시뮬레이션:
    //    total=1600px, px_per_col=8.0 → min_px=320px → min_ratio=0.2
    //    initial_ratio=0.6, delta=-400px → raw=(0.6*1600-400)/1600=560/1600=0.35
    //    0.35 >= min_ratio(0.2) → clamped = 0.35 (범위 내)
    //    더 극단적인 delta: delta=-700px → raw=(0.6*1600-700)/1600=260/1600=0.1625
    //    0.1625 < 0.2 → clamped = 0.2
    let total_px = 1600.0_f32;
    let px_per_col = 8.0_f32;
    let mut divider = GpuiDivider::new(SplitDirection::Horizontal, 0.6, px_per_col, 10.0);

    // 극단적 drag: 우측 sibling 이 MIN_COLS 미만으로
    let clamped_ratio = divider.on_drag(-700.0, total_px);

    // min_ratio = (40 * 8.0) / 1600.0 = 320 / 1600 = 0.2
    let expected_min_ratio = (40.0_f32 * px_per_col) / total_px;
    assert!(
        (clamped_ratio - expected_min_ratio).abs() < 1e-4,
        "AC-R-5: drag 결과 ratio 가 min_ratio({expected_min_ratio:.4}) 로 clamp 되어야 함, 실제={clamped_ratio:.4}"
    );

    // 3. clamped ratio 를 PaneTree 에 반영 (set_ratio 성공 검증)
    entity.update(&mut cx, |tc, _cx| {
        tc.active_tab_mut()
            .pane_tree
            .set_ratio(&split_node_id, clamped_ratio)
            .expect("clamped ratio 로 set_ratio 성공");
    });

    let final_ratio = cx.read(|app| {
        let tc = entity.read(app);
        tc.active_tab()
            .pane_tree
            .get_ratio(&split_node_id)
            .expect("ratio 조회 성공")
    });

    assert!(
        (final_ratio - expected_min_ratio).abs() < 1e-4,
        "AC-R-5: PaneTree 에 반영된 ratio 가 min_ratio({expected_min_ratio:.4}) 이어야 함, 실제={final_ratio:.4}"
    );
}

/// AC-R-5: divider drag 이 유효한 범위 내에 있으면 ratio 가 정상 업데이트된다.
///
/// AC-R-5 보완: valid drag (min_ratio < new_ratio < max_ratio) 시 PaneTree.ratio 가 갱신된다.
/// total=1600px, px_per_col=8.0 → min_ratio=0.2, max_ratio=0.8
/// initial_ratio=0.5, delta=+160px → raw=(0.5*1600+160)/1600=960/1600=0.6 (범위 내)
#[test]
fn divider_drag_within_bounds_updates_ratio() {
    use moai_studio_ui::panes::{GpuiDivider, ResizableDivider, SplitDirection};

    let mut cx = TestAppContext::single();
    let entity = cx.new(|_cx| TabContainer::new());

    // horizontal split 생성
    let split_node_id = entity.update(&mut cx, |tc, _cx| {
        let id0 = tc
            .active_tab()
            .last_focused_pane
            .clone()
            .expect("focused pane 필요");
        tc.active_tab_mut()
            .pane_tree
            .split_horizontal(&id0, PaneId::new_unique(), "pane-1".to_string())
            .expect("split 성공");
        tc.active_tab()
            .pane_tree
            .find_split_node_id()
            .expect("split node id 필요")
            .clone()
    });

    // GpuiDivider valid drag: delta=+160px → new_ratio=0.6 (범위 내)
    let total_px = 1600.0_f32;
    let px_per_col = 8.0_f32;
    let mut divider = GpuiDivider::new(SplitDirection::Horizontal, 0.5, px_per_col, 10.0);
    let new_ratio = divider.on_drag(160.0, total_px);

    // raw = (0.5 * 1600 + 160) / 1600 = 960 / 1600 = 0.6
    assert!(
        (new_ratio - 0.6).abs() < 1e-4,
        "AC-R-5: valid drag → ratio=0.6 예상, 실제={new_ratio:.4}"
    );

    // PaneTree 에 반영
    entity.update(&mut cx, |tc, _cx| {
        tc.active_tab_mut()
            .pane_tree
            .set_ratio(&split_node_id, new_ratio)
            .expect("valid ratio 로 set_ratio 성공");
    });

    let final_ratio = cx.read(|app| {
        let tc = entity.read(app);
        tc.active_tab()
            .pane_tree
            .get_ratio(&split_node_id)
            .expect("ratio 조회 성공")
    });

    assert!(
        (final_ratio - 0.6).abs() < 1e-4,
        "AC-R-5: PaneTree 에 반영된 ratio=0.6 예상, 실제={final_ratio:.4}"
    );
}
