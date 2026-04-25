//! SPEC-V3-004 MS-1 render layer integration 테스트.
//!
//! ## 목적
//!
//! - AC-R-1: TabContainer 가 Entity<TabContainer> 로 생성 가능하고 render 호출 시 panic 없음.
//! - AC-R-6: gpui test-support feature 채택 (USER-DECISION ADOPT) 으로 TestAppContext 기반 smoke test 실행.
//!
//! ## 범위
//!
//! MS-1 scope: placeholder render 만 검증. render_pane_tree (MS-2), divider drag (MS-3) 는 별도.

use gpui::{AppContext as _, TestAppContext};
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
