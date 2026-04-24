// @MX:ANCHOR: [AUTO] bench-pane-split
// @MX:REASON: [AUTO] AC-P-18 측정 harness 의 단일 진입점 (MS-1 carry-over).
//   9-leaf split paint 성능 기준선 기록 (목표: ≤ 200ms).
//   미래 회귀 감지용 criterion 레포트 생성 포인트 (fan_in >= 2: CI, T11 run).

//! `bench_pane_split` — AC-P-18 pane split 성능 측정 (SPEC-V3-003 T11 MS-1 carry-over).
//!
//! ## 측정 대상
//!
//! - `GpuiNativeSplitter::split_horizontal` × 8 회 반복으로 9-leaf 트리 구성 avg ≤ 200ms (AC-P-18).
//!
//! ## 참고
//!
//! bench 수치는 CI runner 환경에 따라 변동된다.
//! assertion 으로 강제하지 않고 criterion HTML 레포트에 기록만 한다.

use criterion::{Criterion, criterion_group, criterion_main};
use moai_studio_ui::panes::{GpuiNativeSplitter, PaneId, PaneSplitter};

// ============================================================
// 헬퍼
// ============================================================

/// root 1 개 leaf 로 GpuiNativeSplitter<String> 를 생성한다.
fn make_splitter_fresh() -> (GpuiNativeSplitter<String>, PaneId) {
    let root_id = PaneId::new_unique();
    let splitter = GpuiNativeSplitter::new_with_factory(
        root_id.clone(),
        "root".to_string(),
        Box::new(|id| format!("pane-{}", id.0)),
    );
    (splitter, root_id)
}

// ============================================================
// Benchmark: 9-leaf split 구성 (AC-P-18)
// ============================================================

/// `split_horizontal` × 8 회로 9-leaf 트리 구성 성능 측정 (AC-P-18).
///
/// 각 반복(b.iter)에서 splitter 를 재생성하여 분할 연산만 측정한다.
fn bench_pane_split_9_leaf(c: &mut Criterion) {
    c.bench_function("pane_split_9_leaf", |b| {
        b.iter(|| {
            let (mut splitter, root_id) = make_splitter_fresh();
            // root → 순차 split_horizontal × 8 → 9 leaf
            let mut last_id = root_id;
            for _ in 0..8 {
                let new_id = splitter
                    .split_horizontal(last_id.clone())
                    .expect("split_horizontal 성공");
                last_id = new_id;
            }
            // 컴파일러 최적화 방지: leaf_count 소비
            assert_eq!(splitter.tree().leaf_count(), 9);
        });
    });
}

criterion_group!(benches, bench_pane_split_9_leaf);
criterion_main!(benches);
