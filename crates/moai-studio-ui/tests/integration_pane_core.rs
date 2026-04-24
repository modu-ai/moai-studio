//! Pane core 통합 테스트 — SPEC-V3-003 MS-1 T7 (AC-P-1, AC-P-2 완전 검증).
//!
//! ## 목적
//!
//! T4 의 단위 테스트 (`splitter_gpui_native::tests`) 는 `GpuiNativeSplitter` 내부 동작을
//! 직접 검증한다. 본 통합 테스트는 **`moai_studio_ui` 라이브러리 경계를 통해** 동일 API 에
//! 접근하여 re-export 경로가 올바른지 확인한다.
//!
//! - `split_creates_and_drops_correctly_via_splitter` — AC-P-1: split 호출 시 새 PaneId 를 반환하고
//!   PaneTree leaf_count 가 증가함을 라이브러리 경계 너머에서 검증.
//! - `close_frees_pane_drops_arc_payload` — AC-P-2: close 호출 시 Arc payload 의 strong_count 가
//!   감소하여 메모리가 해제됨을 라이브러리 경계 너머에서 검증.

use moai_studio_ui::panes::{GpuiNativeSplitter, PaneId, PaneSplitter};
use std::sync::{Arc, Mutex};

// ============================================================
// AC-P-1 통합 검증 — split 이 새 PaneId 를 반환하고 leaf_count 가 증가
// ============================================================

/// split 호출이 새 PaneId 를 반환하고 PaneTree leaf_count 를 증가시킴을
/// 라이브러리 경계를 통해 검증한다 (AC-P-1 완전 통합).
#[test]
fn split_creates_and_drops_correctly_via_splitter() {
    let root_id = PaneId::new_from_literal("integ-root");
    let mut splitter = GpuiNativeSplitter::new_with_factory(
        root_id.clone(),
        "integ-root-payload".to_string(),
        Box::new(|id| format!("integ-pane-{}", id.0)),
    );

    // 초기 상태: leaf 1개
    assert_eq!(
        splitter.tree().leaf_count(),
        1,
        "초기 leaf_count == 1 (AC-P-1)"
    );
    assert_eq!(
        splitter.focused().map(|id| id.0.as_str()),
        Some("integ-root"),
        "초기 focus 는 root (AC-P-1)"
    );

    // split_horizontal → 새 PaneId 반환 + leaf_count 증가
    let new_id = splitter
        .split_horizontal(root_id.clone())
        .expect("split_horizontal 성공해야 함 (AC-P-1)");
    assert_eq!(
        splitter.tree().leaf_count(),
        2,
        "split 후 leaf_count == 2 (AC-P-1)"
    );
    assert_ne!(new_id.0, "integ-root", "새 pane id 는 root 와 달라야 함");

    // close_pane → leaf_count 감소 (AC-P-2 부분)
    splitter
        .close_pane(new_id)
        .expect("close_pane 성공해야 함 (AC-P-2)");
    assert_eq!(
        splitter.tree().leaf_count(),
        1,
        "close 후 leaf_count == 1 (AC-P-2)"
    );
}

// ============================================================
// AC-P-2 통합 검증 — close 가 Arc payload 를 drop
// ============================================================

/// close_pane 호출 시 leaf payload 의 Arc strong_count 가 감소하여
/// 메모리 해제가 이루어짐을 라이브러리 경계를 통해 검증한다 (AC-P-2 완전 통합).
#[test]
fn close_frees_pane_drops_arc_payload() {
    let root_arc: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let root_id = PaneId::new_from_literal("arc-integ-root");
    let root_payload = Arc::clone(&root_arc);

    let factory_arc: Arc<Mutex<i32>> = Arc::new(Mutex::new(1));
    let factory_arc_clone = Arc::clone(&factory_arc);

    let mut splitter: GpuiNativeSplitter<Arc<Mutex<i32>>> = GpuiNativeSplitter::new_with_factory(
        root_id.clone(),
        root_payload,
        Box::new(move |_id| Arc::clone(&factory_arc_clone)),
    );

    // split_horizontal 으로 factory_arc 의 clone 을 payload 로 갖는 새 leaf 생성
    let new_id = splitter
        .split_horizontal(root_id.clone())
        .expect("split 성공 (AC-P-2)");
    assert_eq!(splitter.tree().leaf_count(), 2, "split 후 leaf 2개");

    // split 후 factory_arc 의 strong_count: factory closure 내부 + leaf payload = 최소 2
    let count_before = Arc::strong_count(&factory_arc);
    assert!(
        count_before >= 2,
        "split 후 strong_count >= 2, 실제: {count_before} (AC-P-2)"
    );

    // close_pane → leaf payload drop
    splitter.close_pane(new_id).expect("close 성공 (AC-P-2)");
    assert_eq!(
        splitter.tree().leaf_count(),
        1,
        "close 후 leaf 1개 (AC-P-2)"
    );

    // drop 확인: strong_count 가 감소해야 함
    let count_after = Arc::strong_count(&factory_arc);
    assert!(
        count_after < count_before,
        "close 후 strong_count 감소해야 함 (AC-P-2): before={count_before}, after={count_after}"
    );
}
