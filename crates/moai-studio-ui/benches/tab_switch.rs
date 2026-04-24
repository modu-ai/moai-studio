// @MX:ANCHOR: [AUTO] bench-tab-switch
// @MX:REASON: [AUTO] AC-P-19 측정 harness 의 단일 진입점.
//   9 탭 × 50 사이클 tab_switch 성능 기준선 기록.
//   미래 회귀 감지용 criterion 레포트 생성 포인트 (fan_in >= 3 예상: CI, perf-bot, T11 run).

//! `bench_tab_switch` — AC-P-19 탭 전환 성능 측정 (SPEC-V3-003 T11).
//!
//! ## 측정 대상
//!
//! - `TabContainer::switch_tab` 9 탭 × 50 사이클 avg 지연 ≤ 50ms (AC-P-19 목표).
//!
//! ## 참고
//!
//! bench 수치는 CI runner 환경에 따라 변동된다.
//! assertion 으로 강제하지 않고 criterion HTML 레포트에 기록만 한다.
//! Fatal 회귀 감지는 장기 모니터링 도구에서 처리 (본 SPEC 범위 외).

use criterion::{Criterion, criterion_group, criterion_main};
use moai_studio_ui::panes::{GpuiNativeSplitter, PaneId};
use moai_studio_ui::tabs::{Tab, TabContainer, TabId};

// ============================================================
// 헬퍼
// ============================================================

/// 테스트용 GpuiNativeSplitter<String> 를 생성한다.
fn make_splitter(label: &str) -> GpuiNativeSplitter<String> {
    let root_id = PaneId::new_unique();
    GpuiNativeSplitter::new_with_factory(
        root_id,
        label.to_string(),
        Box::new(|id| format!("pane-{}", id.0)),
    )
}

/// 테스트용 Tab<String> 을 생성한다.
fn make_tab(title: &str) -> Tab<String> {
    Tab {
        id: TabId::new_unique(),
        title: title.to_string(),
        splitter: make_splitter(title),
        last_focused_pane: None,
    }
}

// ============================================================
// Benchmark: 9 tabs × 50 cycles (AC-P-19)
// ============================================================

/// 9 탭 × 50 사이클 tab_switch 성능 측정 (AC-P-19).
///
/// 각 반복(b.iter)에서 setup 비용을 제외하고 switch_tab 만 측정한다.
fn bench_tab_switch_9_tabs_50_cycles(c: &mut Criterion) {
    c.bench_function("tab_switch_9_tabs_50_cycles", |b| {
        // setup: 초기 컨테이너 + 8개 추가 탭 = 9 탭
        let initial = make_tab("tab-0");
        let mut container = TabContainer::new(initial);
        for i in 1..9 {
            container.new_tab(format!("tab-{i}"), make_splitter(&format!("s{i}")));
        }
        assert_eq!(container.tabs.len(), 9, "setup: 9 탭 구성 확인");

        b.iter(|| {
            // 50 사이클: 순방향 0→8 × 25 + 역방향 8→0 × 25
            for _ in 0..25 {
                for idx in 0..9_usize {
                    container.switch_tab(idx).unwrap();
                }
                for idx in (0..9_usize).rev() {
                    container.switch_tab(idx).unwrap();
                }
            }
        });
    });
}

criterion_group!(benches, bench_tab_switch_9_tabs_50_cycles);
criterion_main!(benches);
