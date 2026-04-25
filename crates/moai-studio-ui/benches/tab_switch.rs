// @MX:NOTE: [AUTO] bench-tab-switch
// AC-P-19 SLA: Cmd/Ctrl+1↔9 50 사이클의 평균 전환 시간 ≤ 50ms.
// 측정 대상: dispatch_tab_key + TabContainer::switch_tab 연계 경로.
// criterion 0.5, harness=false 로 실행. --test 모드로 smoke-run 지원.
//
// 사이클 정의: 탭 1→2→...→9→1 을 1사이클로 정의 (9회 switch_tab 호출).
// 50 사이클 = 450회 switch_tab 호출. 측정 단위는 1 사이클(9 switch).
// criterion 은 내부적으로 warm-up + sample 반복 후 통계를 산출한다.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use moai_studio_ui::panes::focus::KeyModifiers;
use moai_studio_ui::tabs::{
    container::TabContainer,
    keys::{TabKeyCode, dispatch_tab_key},
};

/// 탭 전환 벤치마크용 KeyModifiers — 현재 플랫폼의 modifier 만 활성화.
fn platform_modifiers() -> KeyModifiers {
    KeyModifiers {
        cmd: cfg!(target_os = "macos"),
        ctrl: !cfg!(target_os = "macos"),
        alt: false,
        shift: false,
    }
}

/// 9개 탭이 있는 TabContainer 를 초기화하여 반환한다.
///
/// 최초 생성 시 1개의 탭이 포함되므로 new_tab 을 8번 호출한다.
fn build_container_with_9_tabs() -> TabContainer {
    let mut container = TabContainer::new();
    for _ in 0..8 {
        container.new_tab(None);
    }
    // active 를 0 으로 초기화
    container.switch_tab(0).unwrap();
    container
}

/// AC-P-19: Cmd/Ctrl+1~9 경로 — 50 사이클(9 switch / cycle) 탭 전환 벤치.
///
/// 1 사이클: 탭 1→2→3→4→5→6→7→8→9 → 1 (인덱스 0→1→...→8→0).
/// dispatch_tab_key(Digit(n)) 를 통해 SwitchToTab(n-1) 명령을 얻고
/// TabContainer::switch_tab 을 호출하는 전체 경로를 측정한다.
fn bench_tab_switch_50_cycles(c: &mut Criterion) {
    let mods = platform_modifiers();

    c.bench_function("tab_switch_50_cycles", |b| {
        b.iter(|| {
            let mut container = build_container_with_9_tabs();
            // 50 사이클 × 9 switch = 450 switch_tab 호출
            for _ in 0..50 {
                for digit in 1u8..=9 {
                    // 탭 n 으로 이동 (Cmd/Ctrl+digit)
                    if let Some(cmd) = dispatch_tab_key(mods, TabKeyCode::Digit(digit)) {
                        use moai_studio_ui::tabs::keys::TabCommand;
                        if let TabCommand::SwitchToTab(idx) = cmd {
                            let _ = black_box(container.switch_tab(idx));
                        }
                    }
                }
            }
            black_box(container)
        })
    });
}

criterion_group!(benches, bench_tab_switch_50_cycles);
criterion_main!(benches);
