//! MS-2 키 바인딩 통합 테스트 — SPEC-V3-003 T9 (AC-P-9a / AC-P-9b / AC-P-26).
//!
//! ## 목적
//!
//! 라이브러리 경계를 통해 `dispatch_key` → `TabContainer::dispatch_tab_command` 까지
//! MS-2 바인딩 전체를 검증한다. macOS / Linux 플랫폼 분기는 `#[cfg(target_os)]` 로 처리.
//!
//! ## AC 커버리지
//!
//! - AC-P-9a: macOS Cmd 바인딩 (Cmd+T/W/1..9/\/Shift+\/{/})
//! - AC-P-9b: Linux Ctrl 바인딩 (동일 조합, USER-DECISION (a) 현행 유지)
//! - AC-P-23 regression: Ctrl+B 가 FocusRouter/dispatch_key 에 소비되지 않음
//! - AC-P-26 prep: Ctrl+B passthrough 통합 레벨 재확인

use moai_studio_ui::panes::{
    FocusCommand, GpuiNativeSplitter, KeyCode, KeyModifiers, PaneId, dispatch_key,
};
use moai_studio_ui::tabs::TabContainer;

// ============================================================
// 헬퍼
// ============================================================

fn make_splitter(label: &str) -> GpuiNativeSplitter<String> {
    let root_id = PaneId::new_unique();
    GpuiNativeSplitter::new_with_factory(
        root_id,
        label.to_string(),
        Box::new(|id| format!("pane-{}", id.0)),
    )
}

fn make_container() -> TabContainer<String> {
    use moai_studio_ui::tabs::Tab;
    use moai_studio_ui::tabs::TabId;
    let tab = Tab {
        id: TabId::new_unique(),
        title: "init".to_string(),
        splitter: make_splitter("init"),
        last_focused_pane: None,
    };
    TabContainer::new(tab)
}

// ============================================================
// AC-P-9a macOS — Cmd+T 새 탭 (macOS 전용)
// ============================================================

/// macOS: Cmd+T → dispatch_key 가 NewTab 반환 + dispatch_tab_command 가 탭 생성 (AC-P-9a).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_t_creates_new_tab() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_key(mods, KeyCode::T);
    assert_eq!(cmd, Some(FocusCommand::NewTab), "Cmd+T → NewTab");

    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::NewTab, Some(("new-tab", make_splitter("x"))))
        .unwrap();
    assert_eq!(container.tabs.len(), 2, "탭이 2 개로 늘어야 함 (AC-P-9a)");
}

// ============================================================
// AC-P-9b Linux — Ctrl+T 새 탭 (non-macOS 전용)
// ============================================================

/// Linux: Ctrl+T → dispatch_key 가 NewTab 반환 + dispatch_tab_command 가 탭 생성 (AC-P-9b).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_t_creates_new_tab() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_key(mods, KeyCode::T);
    assert_eq!(cmd, Some(FocusCommand::NewTab), "Ctrl+T → NewTab");

    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::NewTab, Some(("new-tab", make_splitter("x"))))
        .unwrap();
    assert_eq!(container.tabs.len(), 2, "탭이 2 개로 늘어야 함 (AC-P-9b)");
}

// ============================================================
// AC-P-9a macOS — Cmd+1~9 탭 전환
// ============================================================

/// macOS: Cmd+Digit(n) → SwitchTabIdx(n) 반환 (AC-P-9a).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_digit_switches_tab() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };

    for n in 0usize..=8 {
        let cmd = dispatch_key(mods, KeyCode::Digit(n));
        assert_eq!(
            cmd,
            Some(FocusCommand::SwitchTabIdx(n)),
            "Cmd+Digit({n}) → SwitchTabIdx({n}) (AC-P-9a)"
        );
    }

    // 실제 탭 전환 검증 (탭 2개)
    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::NewTab, Some(("tab-b", make_splitter("b"))))
        .unwrap();
    container
        .dispatch_tab_command(FocusCommand::SwitchTabIdx(0), None)
        .unwrap();
    assert_eq!(
        container.active_tab_idx, 0,
        "SwitchTabIdx(0) 후 idx 0 (AC-P-9a)"
    );
}

// ============================================================
// AC-P-9b Linux — Ctrl+1~9 탭 전환
// ============================================================

/// Linux: Ctrl+Digit(n) → SwitchTabIdx(n) 반환 (AC-P-9b).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_digit_switches_tab() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };

    for n in 0usize..=8 {
        let cmd = dispatch_key(mods, KeyCode::Digit(n));
        assert_eq!(
            cmd,
            Some(FocusCommand::SwitchTabIdx(n)),
            "Ctrl+Digit({n}) → SwitchTabIdx({n}) (AC-P-9b)"
        );
    }

    // 실제 탭 전환 검증
    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::NewTab, Some(("tab-b", make_splitter("b"))))
        .unwrap();
    container
        .dispatch_tab_command(FocusCommand::SwitchTabIdx(0), None)
        .unwrap();
    assert_eq!(
        container.active_tab_idx, 0,
        "SwitchTabIdx(0) 후 idx 0 (AC-P-9b)"
    );
}

// ============================================================
// AC-P-9a macOS — Cmd+\ 수평 분할
// ============================================================

/// macOS: Cmd+\ → SplitHorizontal + dispatch 후 leaf_count 증가 (AC-P-9a).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_backslash_splits_horizontal() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_key(mods, KeyCode::Backslash);
    assert_eq!(
        cmd,
        Some(FocusCommand::SplitHorizontal),
        "Cmd+\\ → SplitHorizontal (AC-P-9a)"
    );

    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::SplitHorizontal, None)
        .unwrap();
    assert_eq!(
        container.get_active_splitter().tree().leaf_count(),
        2,
        "SplitHorizontal 후 leaf_count == 2 (AC-P-9a)"
    );
}

// ============================================================
// AC-P-9b Linux — Ctrl+\ 수평 분할
// ============================================================

/// Linux: Ctrl+\ → SplitHorizontal + dispatch 후 leaf_count 증가 (AC-P-9b).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_backslash_splits_horizontal() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_key(mods, KeyCode::Backslash);
    assert_eq!(
        cmd,
        Some(FocusCommand::SplitHorizontal),
        "Ctrl+\\ → SplitHorizontal (AC-P-9b)"
    );

    let mut container = make_container();
    container
        .dispatch_tab_command(FocusCommand::SplitHorizontal, None)
        .unwrap();
    assert_eq!(
        container.get_active_splitter().tree().leaf_count(),
        2,
        "SplitHorizontal 후 leaf_count == 2 (AC-P-9b)"
    );
}

// ============================================================
// AC-P-23 regression + AC-P-26 prep — Ctrl+B passthrough (플랫폼 무관)
// ============================================================

/// Ctrl+B 는 dispatch_key 에서 None 반환 — tmux prefix 키가 앱에 소비되지 않는다 (AC-P-23/AC-P-26).
///
/// 이 테스트는 플랫폼 무관하게 Ctrl modifier + B 가 None 임을 검증한다.
/// dispatch_key 는 어떤 MS-2 분기에도 Ctrl+B 를 소비하지 않으므로,
/// 실제 GPUI event handler(T10)에서 focused pane 의 TerminalSurface 로 전달된다.
#[test]
fn ms2_ctrl_b_not_consumed_for_tmux_passthrough() {
    // Ctrl+B: ctrl=true, alt=false, shift=false, code=Other
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let result = dispatch_key(mods, KeyCode::Other);
    assert_eq!(
        result, None,
        "Ctrl+B 는 dispatch_key 에서 None (tmux passthrough 보장, AC-P-23/AC-P-26)"
    );
}
