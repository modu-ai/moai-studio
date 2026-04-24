// @MX:NOTE: [AUTO] ac-p-5-headless-resize
// SPEC-V3-003 MS-1 carry-over: AC-P-5 headless resize 통합 테스트.
// gpui 0.2.2 에 `test-support` feature 가 없어 TestAppContext 를 사용할 수 없다.
// USER-DECISION: test-support-feature-adoption = 추가 (default) 로 결정했으나,
//   cargo info gpui@0.2.2 검사 결과 해당 feature 가 crates.io 0.2.2 에 미존재.
// TODO(T11.1): gpui test-support feature 가 활성화되면 #[ignore] 제거 + 구현 완성.
//   해소 조건: `gpui` crate 에 `test-support` feature 추가 또는 버전 업그레이드.

//! AC-P-5 headless resize 통합 테스트 (SPEC-V3-003 T11 MS-1 carry-over).
//!
//! ## 목표 (해소 시)
//!
//! `TestAppContext` 를 사용하여 window resize 이벤트 발생 후
//! deepest pane 이 `PaneConstraints::MIN_COLS / MIN_ROWS` 미만이면
//! hide (혹은 auto-close) 됨을 검증한다.
//!
//! ## 현재 상태
//!
//! gpui 0.2.2 (crates.io) 에 `test-support` feature 가 없어 `TestAppContext` 를
//! 활성화할 수 없다. 모든 테스트는 `#[ignore]` 처리되며
//! TODO(T11.1) 로 후속 SPEC 에서 해소한다.
//!
//! ## 참고
//!
//! - `cargo info gpui` → features 목록에 `test-support` 미존재 (2026-04-24 확인).
//! - SPEC-V3-003 계약 §10.2 MS-1 carry-over: AC-P-5.

use moai_studio_ui::panes::{GpuiNativeSplitter, PaneConstraints, PaneId};

// ============================================================
// 헬퍼
// ============================================================

/// 테스트용 GpuiNativeSplitter<String> 을 생성한다.
fn make_splitter() -> (GpuiNativeSplitter<String>, PaneId) {
    let root_id = PaneId::new_unique();
    let splitter = GpuiNativeSplitter::new_with_factory(
        root_id.clone(),
        "root".to_string(),
        Box::new(|id| format!("pane-{}", id.0)),
    );
    (splitter, root_id)
}

// ============================================================
// AC-P-5: headless resize — #[ignore] + TODO(T11.1)
// ============================================================

/// window resize 후 deepest pane 이 MIN_COLS/MIN_ROWS 미만이면 hide 됨을 검증한다.
///
/// ## 현재 상태
///
/// `gpui::TestAppContext` 를 사용할 수 없어 `#[ignore]` 처리.
/// TODO(T11.1): gpui test-support feature 활성화 시 구현 완성.
#[test]
#[ignore = "TODO(T11.1): gpui 0.2.2 에 test-support feature 미존재. 버전 업그레이드 후 해소."]
fn headless_resize_hides_pane_below_min_constraints() {
    // TODO(T11.1): TestAppContext 기반 구현 예시 (future):
    //
    // use gpui::TestAppContext;
    // let cx = TestAppContext::new(...);
    // let (mut splitter, root_id) = make_splitter();
    //
    // // 8-leaf split 로 deepest pane 구성
    // let mut last_id = root_id;
    // for _ in 0..7 {
    //     let new_id = splitter.split_horizontal(last_id.clone()).unwrap();
    //     last_id = new_id.clone();
    // }
    // let deepest = last_id;
    //
    // // window resize → 전체 폭이 MIN_COLS * 8 미만 → deepest pane 은 MIN_COLS 미만
    // cx.simulate_window_resize(Bounds {
    //     size: Size { width: Pixels(PaneConstraints::MIN_COLS as f32 * 6.0), height: Pixels(200.0) },
    //     ..Default::default()
    // });
    //
    // // deepest pane 이 hidden (leaf_count 감소 또는 is_visible == false) 를 확인
    // // assert!(splitter.tree().leaf_count() < 8 || !splitter.tree().is_visible(&deepest));

    // 이 지점은 #[ignore] 로 도달하지 않는다.
    let _ = PaneConstraints::MIN_COLS;
    let _ = PaneConstraints::MIN_ROWS;
}

/// MIN_COLS / MIN_ROWS 상수 접근 smoke test — AC-P-5 의존 상수 존재 확인.
///
/// #[ignore] 없이 항상 실행된다. 상수가 존재하는 한 PASS.
#[test]
fn pane_constraints_constants_accessible() {
    // AC-P-5 headless test 가 의존하는 상수가 공개 API 로 접근 가능한지 확인한다.
    assert_eq!(PaneConstraints::MIN_COLS, 40, "MIN_COLS == 40");
    assert_eq!(PaneConstraints::MIN_ROWS, 10, "MIN_ROWS == 10");
}

/// make_splitter 헬퍼 smoke test — bench/test 공통 패턴 검증.
#[test]
fn make_splitter_creates_single_leaf() {
    let (splitter, _root_id) = make_splitter();
    assert_eq!(splitter.tree().leaf_count(), 1, "초기 leaf_count == 1");
}
