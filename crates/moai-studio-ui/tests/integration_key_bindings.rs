//! 키 바인딩 통합 테스트 — MS-1 + MS-2.
//!
//! ## MS-1 (SPEC-V3-003 T6)
//!
//! FocusRouter 의 플랫폼 modifier 상수 및 dispatch_key 동작을 라이브러리 경계를 통해 검증.
//!
//! - `macos_cmd_bindings_ms1` (AC-P-9a MS-1 부분): macOS 에서 PLATFORM_MOD == Cmd
//! - `linux_ctrl_bindings_ms1` (AC-P-9b MS-1 부분): macOS 외 플랫폼에서 PLATFORM_MOD == Ctrl
//!
//! ## MS-2 (SPEC-V3-003 T9)
//!
//! TabCommand dispatcher 와 TabContainer 를 연동하여 키 → 탭 조작 파이프라인을 검증.
//!
//! - `macos_ms2_cmd_t_creates_new_tab` (AC-P-9a MS-2): macOS Cmd+T → TabContainer.new_tab
//! - `linux_ms2_ctrl_t_creates_new_tab` (AC-P-9b MS-2): Linux Ctrl+T → TabContainer.new_tab
//! - `ms2_cmd_ctrl_1_switches_to_tab_0` (AC-P-9a/9b): Cmd/Ctrl+1 → TabContainer.switch_tab(0)
//! - `ms2_cmd_ctrl_brace_prev_next_tab` (AC-P-9a/9b): {/} 키 → 이전/다음 탭

use moai_studio_ui::panes::{KeyModifiers, PLATFORM_MOD, PlatformMod, dispatch_key};
use moai_studio_ui::tabs::{TabCommand, TabContainer, TabKeyCode, dispatch_tab_key};

// ============================================================
// MS-1 통합 검증
// ============================================================

/// macOS: 라이브러리 경계를 통해 PLATFORM_MOD == Cmd 확인 (AC-P-9a MS-1).
#[cfg(target_os = "macos")]
#[test]
fn macos_cmd_bindings_ms1() {
    assert_eq!(
        PLATFORM_MOD,
        PlatformMod::Cmd,
        "macOS 에서 PLATFORM_MOD 는 Cmd 여야 한다 (AC-P-9a MS-1)"
    );

    // Cmd+Alt+Right → FocusCommand::Next (라이브러리 경계 통과 검증)
    let mods = KeyModifiers {
        cmd: true,
        alt: true,
        ctrl: false,
        shift: false,
    };
    use moai_studio_ui::panes::FocusCommand;
    use moai_studio_ui::panes::KeyCode;
    assert_eq!(
        dispatch_key(mods, KeyCode::ArrowRight),
        Some(FocusCommand::Next),
        "Cmd+Alt+Right → Next (AC-P-9a MS-1)"
    );
}

/// Linux: 라이브러리 경계를 통해 PLATFORM_MOD == Ctrl 확인 (AC-P-9b MS-1).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ctrl_bindings_ms1() {
    assert_eq!(
        PLATFORM_MOD,
        PlatformMod::Ctrl,
        "Linux 에서 PLATFORM_MOD 는 Ctrl 여야 한다 (AC-P-9b MS-1)"
    );

    // Ctrl+Alt+Right → FocusCommand::Next (라이브러리 경계 통과 검증)
    let mods = KeyModifiers {
        ctrl: true,
        alt: true,
        cmd: false,
        shift: false,
    };
    use moai_studio_ui::panes::FocusCommand;
    use moai_studio_ui::panes::KeyCode;
    assert_eq!(
        dispatch_key(mods, KeyCode::ArrowRight),
        Some(FocusCommand::Next),
        "Ctrl+Alt+Right → Next (AC-P-9b MS-1)"
    );
}

// ============================================================
// MS-2 통합 검증 — macOS
// ============================================================

/// macOS: Cmd+T → dispatch_tab_key → TabContainer.new_tab (AC-P-9a MS-2 전체).
///
/// 라이브러리 경계를 통해 Cmd+T 키 이벤트가 실제 탭 생성으로 이어지는
/// 완전한 파이프라인을 검증한다.
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_t_creates_new_tab() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_tab_key(mods, TabKeyCode::T);
    assert_eq!(
        cmd,
        Some(TabCommand::NewTab),
        "Cmd+T → NewTab (AC-P-9a MS-2)"
    );

    let mut container = TabContainer::new();
    let initial_count = container.tab_count();
    if let Some(TabCommand::NewTab) = cmd {
        container.new_tab(None);
    }
    assert_eq!(
        container.tab_count(),
        initial_count + 1,
        "NewTab 명령 후 탭 수 1 증가 (AC-P-9a MS-2)"
    );
}

/// macOS: Cmd+1..9 전체 범위 → SwitchToTab(0..8) (AC-P-9a MS-2).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_1_to_9_switches_tabs() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    for n in 1u8..=9 {
        let cmd = dispatch_tab_key(mods, TabKeyCode::Digit(n));
        assert_eq!(
            cmd,
            Some(TabCommand::SwitchToTab((n - 1) as usize)),
            "Cmd+{} → SwitchToTab({}) (AC-P-9a MS-2)",
            n,
            n - 1
        );
    }
}

/// macOS: Cmd+\ → SplitVertical, Cmd+Shift+\ → SplitHorizontal (AC-P-9a MS-2).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_backslash_splits() {
    let mods_no_shift = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    let mods_shift = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: true,
    };
    assert_eq!(
        dispatch_tab_key(mods_no_shift, TabKeyCode::Backslash),
        Some(TabCommand::SplitVertical),
        "Cmd+\\ → SplitVertical (AC-P-9a MS-2)"
    );
    assert_eq!(
        dispatch_tab_key(mods_shift, TabKeyCode::Backslash),
        Some(TabCommand::SplitHorizontal),
        "Cmd+Shift+\\ → SplitHorizontal (AC-P-9a MS-2)"
    );
}

/// macOS: Cmd+{ → PrevTab, Cmd+} → NextTab (AC-P-9a MS-2).
#[cfg(target_os = "macos")]
#[test]
fn macos_ms2_cmd_braces_prev_next_tab() {
    let mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    assert_eq!(
        dispatch_tab_key(mods, TabKeyCode::BraceOpen),
        Some(TabCommand::PrevTab),
        "Cmd+{{ → PrevTab (AC-P-9a MS-2)"
    );
    assert_eq!(
        dispatch_tab_key(mods, TabKeyCode::BraceClose),
        Some(TabCommand::NextTab),
        "Cmd+}} → NextTab (AC-P-9a MS-2)"
    );
}

// ============================================================
// MS-2 통합 검증 — Linux/Windows (S4 default-a: Ctrl 기반)
// ============================================================

/// Linux: Ctrl+T → dispatch_tab_key → TabContainer.new_tab (AC-P-9b MS-2 전체).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_t_creates_new_tab() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let cmd = dispatch_tab_key(mods, TabKeyCode::T);
    assert_eq!(
        cmd,
        Some(TabCommand::NewTab),
        "Ctrl+T → NewTab (AC-P-9b MS-2)"
    );

    let mut container = TabContainer::new();
    let initial_count = container.tab_count();
    if let Some(TabCommand::NewTab) = cmd {
        container.new_tab(None);
    }
    assert_eq!(
        container.tab_count(),
        initial_count + 1,
        "NewTab 명령 후 탭 수 1 증가 (AC-P-9b MS-2)"
    );
}

/// Linux: Ctrl+1..9 전체 범위 → SwitchToTab(0..8) (AC-P-9b MS-2).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_1_to_9_switches_tabs() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    for n in 1u8..=9 {
        let cmd = dispatch_tab_key(mods, TabKeyCode::Digit(n));
        assert_eq!(
            cmd,
            Some(TabCommand::SwitchToTab((n - 1) as usize)),
            "Ctrl+{} → SwitchToTab({}) (AC-P-9b MS-2)",
            n,
            n - 1
        );
    }
}

/// Linux: Ctrl+\ → SplitVertical, Ctrl+Shift+\ → SplitHorizontal (AC-P-9b MS-2).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_backslash_splits() {
    let mods_no_shift = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let mods_shift = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: true,
    };
    assert_eq!(
        dispatch_tab_key(mods_no_shift, TabKeyCode::Backslash),
        Some(TabCommand::SplitVertical),
        "Ctrl+\\ → SplitVertical (AC-P-9b MS-2)"
    );
    assert_eq!(
        dispatch_tab_key(mods_shift, TabKeyCode::Backslash),
        Some(TabCommand::SplitHorizontal),
        "Ctrl+Shift+\\ → SplitHorizontal (AC-P-9b MS-2)"
    );
}

/// Linux: Ctrl+{ → PrevTab, Ctrl+} → NextTab (AC-P-9b MS-2).
#[cfg(not(target_os = "macos"))]
#[test]
fn linux_ms2_ctrl_braces_prev_next_tab() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    assert_eq!(
        dispatch_tab_key(mods, TabKeyCode::BraceOpen),
        Some(TabCommand::PrevTab),
        "Ctrl+{{ → PrevTab (AC-P-9b MS-2)"
    );
    assert_eq!(
        dispatch_tab_key(mods, TabKeyCode::BraceClose),
        Some(TabCommand::NextTab),
        "Ctrl+}} → NextTab (AC-P-9b MS-2)"
    );
}

// ============================================================
// 플랫폼 무관 — AC-P-26 tmux passthrough 통합 검증
// ============================================================

/// Ctrl+B 는 tab dispatcher 에 소비되지 않는다 (AC-P-26, 플랫폼 무관).
///
/// PLATFORM_MOD 가 Ctrl 이든 Cmd 이든, Ctrl+B (TabKeyCode::Other) 는
/// dispatch_tab_key 에서 None 을 반환하여 OS/GPUI 레벨로 패스스루된다.
#[test]
fn ctrl_b_passthrough_in_tab_dispatcher() {
    let mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    assert_eq!(
        dispatch_tab_key(mods, TabKeyCode::Other),
        None,
        "Ctrl+B 는 tab dispatcher 에 소비되지 않아야 한다 (AC-P-26)"
    );
}
