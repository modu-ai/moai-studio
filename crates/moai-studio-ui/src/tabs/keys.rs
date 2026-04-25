//! MS-2 탭 키 바인딩 — key → [`TabCommand`] 매핑.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-4 (REQ-P-030 ~ REQ-P-034): platform_mod 매크로
//! - spec.md §5 RG-P-5 REQ-P-040/041/042: 탭 생성/전환
//! - contract.md §10.2 AC-P-9a/9b, AC-P-26
//! - contract.md §10.6 S4 default (a): Linux 는 Ctrl 기반 유지
//!
//! ## 설계 원칙
//!
//! GPUI 독립 순수 Rust. [`dispatch_tab_key`] 는 [`KeyModifiers`] + [`TabKeyCode`] 를
//! 받아 [`TabCommand`] 를 반환한다. None 이면 caller 가 passthrough 처리.
//!
//! AC-P-26 tmux passthrough: PLATFORM_MOD = Ctrl 인 경우 순수 Ctrl+B 는
//! dispatch_tab_key 에서 None 반환 → OS/GPUI 레벨로 전달됨 (REQ-P-034).

use crate::panes::focus::{KeyModifiers, PLATFORM_MOD, PlatformMod};
use gpui::Keystroke;

// ============================================================
// TabKeyCode — MS-2 확장 키 코드
// ============================================================

/// MS-2 탭 바인딩에서 사용하는 키 코드.
///
/// MS-1 의 `KeyCode` (panes::focus) 를 대체하지 않고 탭 전용 키 코드를 별도 정의한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKeyCode {
    /// T 키 (새 탭: Cmd/Ctrl+T).
    T,
    /// 숫자 1~9 (탭 직접 이동: Cmd/Ctrl+1..9).
    Digit(u8),
    /// 백슬래시 키 (수직 split: Cmd/Ctrl+\).
    Backslash,
    /// 여는 중괄호 키 (이전 탭: Cmd/Ctrl+{).
    BraceOpen,
    /// 닫는 중괄호 키 (다음 탭: Cmd/Ctrl+}).
    BraceClose,
    /// 기타 키 (passthrough).
    Other,
}

// ============================================================
// TabCommand
// ============================================================

/// 탭 관련 사용자 명령.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabCommand {
    /// 새 탭 생성 (REQ-P-040, Cmd/Ctrl+T).
    NewTab,
    /// 인덱스 기반 탭 전환 (0-based, Cmd/Ctrl+1 → idx=0, ..., Cmd/Ctrl+9 → idx=8).
    SwitchToTab(usize),
    /// 수직 split (Cmd/Ctrl+\).
    SplitVertical,
    /// 수평 split (Cmd/Ctrl+Shift+\).
    SplitHorizontal,
    /// 이전 탭으로 이동 (Cmd/Ctrl+{).
    PrevTab,
    /// 다음 탭으로 이동 (Cmd/Ctrl+}).
    NextTab,
}

// ============================================================
// keystroke_to_tab_key — GPUI Keystroke → (KeyModifiers, TabKeyCode) 변환
// ============================================================

// @MX:ANCHOR: [AUTO] keystroke-bridge
// @MX:REASON: [AUTO] SPEC-V3-004 REQ-R-030. GPUI Keystroke → (KeyModifiers, TabKeyCode) 변환 진입점.
//   fan_in >= 3: RootView::on_key_down 핸들러 (T5), integration_render 테스트 (AC-R-3), 향후 shortcut manager.
//   macOS: Keystroke.modifiers.platform == Cmd. Linux/Windows: Keystroke.modifiers.control == Ctrl.
// @MX:NOTE: [AUTO] gpui-keystroke-modifiers-mapping
// gpui 0.2.2 의 Modifiers 필드: control, alt, shift, platform (macOS=Cmd, Linux/Win=Super).
// macOS 에서 Cmd 는 Keystroke.modifiers.platform == true.
// 기존 KeyModifiers.cmd 와의 매핑: platform → cmd (macOS), control → ctrl.
/// GPUI [`Keystroke`] 를 [`KeyModifiers`] + [`TabKeyCode`] 쌍으로 변환한다.
///
/// SPEC-V3-004 REQ-R-030: RootView::on_key_down 의 단독 호출자.
///
/// ## GPUI modifiers 매핑
///
/// | GPUI field             | KeyModifiers field | 비고                        |
/// |------------------------|--------------------|-----------------------------|
/// | `modifiers.platform`   | `cmd`              | macOS=Cmd, Linux/Win=Super  |
/// | `modifiers.control`    | `ctrl`             |                             |
/// | `modifiers.shift`      | `shift`            |                             |
/// | `modifiers.alt`        | `alt`              |                             |
pub fn keystroke_to_tab_key(ks: &Keystroke) -> (KeyModifiers, TabKeyCode) {
    let mods = KeyModifiers {
        // macOS: Keystroke.modifiers.platform == Cmd key.
        // Linux/Windows: control 로 처리, platform (Super) 는 cmd 에 포함하지 않음.
        cmd: ks.modifiers.platform,
        ctrl: ks.modifiers.control,
        shift: ks.modifiers.shift,
        alt: ks.modifiers.alt,
    };
    let code = match ks.key.as_str() {
        "t" | "T" => TabKeyCode::T,
        "1" => TabKeyCode::Digit(1),
        "2" => TabKeyCode::Digit(2),
        "3" => TabKeyCode::Digit(3),
        "4" => TabKeyCode::Digit(4),
        "5" => TabKeyCode::Digit(5),
        "6" => TabKeyCode::Digit(6),
        "7" => TabKeyCode::Digit(7),
        "8" => TabKeyCode::Digit(8),
        "9" => TabKeyCode::Digit(9),
        "\\" => TabKeyCode::Backslash,
        "{" => TabKeyCode::BraceOpen,
        "}" => TabKeyCode::BraceClose,
        _ => TabKeyCode::Other,
    };
    (mods, code)
}

// ============================================================
// dispatch_tab_key
// ============================================================

// @MX:ANCHOR: [AUTO] tab-key-dispatch
// @MX:REASON: [AUTO] MS-2 탭 키 이벤트 → TabCommand 단일 진입점.
//   fan_in >= 3: T9 integration tests, T11 bench, RootView GPUI key handler.
//   macOS: Cmd 기반, Linux/Windows: Ctrl 기반 (S4 default-a 결정 반영).
//   AC-P-9a/9b 전체 커버, AC-P-26 tmux passthrough 보장.
/// MS-2 키 이벤트를 [`TabCommand`] 로 변환한다.
///
/// ## 매핑 테이블 (macOS Cmd / Linux+Windows Ctrl)
///
/// | 키 조합                     | 명령                              |
/// |-----------------------------|-----------------------------------|
/// | PLATFORM_MOD + T            | [`TabCommand::NewTab`]            |
/// | PLATFORM_MOD + Digit(n)     | [`TabCommand::SwitchToTab(n-1)`]  |
/// | PLATFORM_MOD + \            | [`TabCommand::SplitVertical`]     |
/// | PLATFORM_MOD + Shift + \    | [`TabCommand::SplitHorizontal`]   |
/// | PLATFORM_MOD + {            | [`TabCommand::PrevTab`]           |
/// | PLATFORM_MOD + }            | [`TabCommand::NextTab`]           |
/// | 그 외                       | `None` (passthrough)              |
///
/// # AC-P-26 tmux passthrough 보장
///
// @MX:NOTE: [AUTO] ac-p-26-tmux-passthrough
// REQ-P-034: 중첩 tmux 환경에서 Ctrl+B 는 host 가 소비하지 않고 OS/GPUI 레벨로 전달.
// dispatch_tab_key 는 PLATFORM_MOD + 특정 키 조합만 소비하므로,
// 순수 Ctrl+B (Digit/T/\/{/} 아님, platform_mod+alt 아님) 는 항상 None 반환.
pub fn dispatch_tab_key(modifiers: KeyModifiers, code: TabKeyCode) -> Option<TabCommand> {
    let platform_mod_active = match PLATFORM_MOD {
        PlatformMod::Cmd => modifiers.cmd,
        PlatformMod::Ctrl => modifiers.ctrl,
    };

    if !platform_mod_active {
        return None;
    }

    match code {
        TabKeyCode::T if !modifiers.shift => Some(TabCommand::NewTab),
        TabKeyCode::Digit(n) if (1..=9).contains(&n) && !modifiers.shift => {
            Some(TabCommand::SwitchToTab((n - 1) as usize))
        }
        TabKeyCode::Backslash if !modifiers.shift => Some(TabCommand::SplitVertical),
        TabKeyCode::Backslash if modifiers.shift => Some(TabCommand::SplitHorizontal),
        TabKeyCode::BraceOpen => Some(TabCommand::PrevTab),
        TabKeyCode::BraceClose => Some(TabCommand::NextTab),
        _ => None,
    }
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------
    // 헬퍼 — 플랫폼 modifier 만 활성화된 KeyModifiers
    // -------------------------------------------------------

    /// 현재 플랫폼의 modifier 만 활성화된 KeyModifiers 생성.
    fn platform_only() -> KeyModifiers {
        KeyModifiers {
            cmd: cfg!(target_os = "macos"),
            ctrl: !cfg!(target_os = "macos"),
            alt: false,
            shift: false,
        }
    }

    // -------------------------------------------------------
    // AC-P-9a: macOS Cmd 기반 탭 바인딩
    // -------------------------------------------------------

    /// macOS: Cmd+T → NewTab.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_t_maps_to_new_tab() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::T),
            Some(TabCommand::NewTab)
        );
    }

    /// macOS: Cmd+1 → SwitchToTab(0).
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_1_maps_to_switch_tab_0() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Digit(1)),
            Some(TabCommand::SwitchToTab(0))
        );
    }

    /// macOS: Cmd+9 → SwitchToTab(8).
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_9_maps_to_switch_tab_8() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Digit(9)),
            Some(TabCommand::SwitchToTab(8))
        );
    }

    /// macOS: Cmd+\ → SplitVertical.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_backslash_maps_to_split_vertical() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Backslash),
            Some(TabCommand::SplitVertical)
        );
    }

    /// macOS: Cmd+Shift+\ → SplitHorizontal.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_shift_backslash_maps_to_split_horizontal() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: true,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Backslash),
            Some(TabCommand::SplitHorizontal)
        );
    }

    /// macOS: Cmd+{ → PrevTab.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_brace_open_maps_to_prev_tab() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::BraceOpen),
            Some(TabCommand::PrevTab)
        );
    }

    /// macOS: Cmd+} → NextTab.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cmd_brace_close_maps_to_next_tab() {
        let mods = KeyModifiers {
            cmd: true,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::BraceClose),
            Some(TabCommand::NextTab)
        );
    }

    // -------------------------------------------------------
    // AC-P-9b: Linux/Windows Ctrl 기반 탭 바인딩 (S4 default-a)
    // -------------------------------------------------------

    /// Linux: Ctrl+T → NewTab.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_t_maps_to_new_tab() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::T),
            Some(TabCommand::NewTab)
        );
    }

    /// Linux: Ctrl+1 → SwitchToTab(0).
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_1_maps_to_switch_tab_0() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Digit(1)),
            Some(TabCommand::SwitchToTab(0))
        );
    }

    /// Linux: Ctrl+9 → SwitchToTab(8).
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_9_maps_to_switch_tab_8() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Digit(9)),
            Some(TabCommand::SwitchToTab(8))
        );
    }

    /// Linux: Ctrl+\ → SplitVertical.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_backslash_maps_to_split_vertical() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Backslash),
            Some(TabCommand::SplitVertical)
        );
    }

    /// Linux: Ctrl+Shift+\ → SplitHorizontal.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_shift_backslash_maps_to_split_horizontal() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: true,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::Backslash),
            Some(TabCommand::SplitHorizontal)
        );
    }

    /// Linux: Ctrl+{ → PrevTab.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_brace_open_maps_to_prev_tab() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::BraceOpen),
            Some(TabCommand::PrevTab)
        );
    }

    /// Linux: Ctrl+} → NextTab.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_ctrl_brace_close_maps_to_next_tab() {
        let mods = KeyModifiers {
            ctrl: true,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(
            dispatch_tab_key(mods, TabKeyCode::BraceClose),
            Some(TabCommand::NextTab)
        );
    }

    // -------------------------------------------------------
    // AC-P-26: tmux passthrough — 플랫폼 무관
    // -------------------------------------------------------

    /// Ctrl+B 는 dispatch_tab_key 에서 소비되지 않는다 (AC-P-26).
    ///
    /// PLATFORM_MOD 가 Ctrl 이더라도 Ctrl+B 는 TabKeyCode::Other 이므로 None 반환.
    /// tmux prefix 키가 host 에 의해 intercept 되지 않음을 보장.
    #[test]
    fn ctrl_b_is_not_consumed_by_tab_dispatcher() {
        // Ctrl 기반 플랫폼 (Linux/Windows) 에서 Ctrl+B 시뮬레이션
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

    /// 플랫폼 modifier 없이 T 키는 passthrough.
    #[test]
    fn t_without_platform_mod_is_passthrough() {
        let mods = KeyModifiers {
            ctrl: false,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(dispatch_tab_key(mods, TabKeyCode::T), None);
    }

    /// 플랫폼 modifier 없이 Digit 는 passthrough.
    #[test]
    fn digit_without_platform_mod_is_passthrough() {
        let mods = KeyModifiers {
            ctrl: false,
            cmd: false,
            alt: false,
            shift: false,
        };
        assert_eq!(dispatch_tab_key(mods, TabKeyCode::Digit(3)), None);
    }

    // -------------------------------------------------------
    // 경계값: Digit 0 과 10 은 매핑 없음
    // -------------------------------------------------------

    /// Digit(0) 은 매핑 없음 (1-9 만 유효).
    #[test]
    fn digit_0_is_not_mapped() {
        let mods = platform_only();
        assert_eq!(dispatch_tab_key(mods, TabKeyCode::Digit(0)), None);
    }

    /// Digit(10) 은 매핑 없음 (1-9 만 유효).
    #[test]
    fn digit_10_is_not_mapped() {
        let mods = platform_only();
        assert_eq!(dispatch_tab_key(mods, TabKeyCode::Digit(10)), None);
    }

    // -------------------------------------------------------
    // dispatch + TabContainer 통합 — key → action 루프
    // -------------------------------------------------------

    /// 탭 dispatcher 로 NewTab 명령을 받아 TabContainer.new_tab 을 호출하면
    /// tab_count 가 증가한다.
    #[test]
    fn new_tab_command_increases_tab_count() {
        use crate::tabs::TabContainer;
        let mods = platform_only();
        let cmd = dispatch_tab_key(mods, TabKeyCode::T);
        assert_eq!(cmd, Some(TabCommand::NewTab));

        let mut container = TabContainer::new();
        assert_eq!(container.tab_count(), 1);
        // NewTab 명령을 TabContainer 에 반영
        if let Some(TabCommand::NewTab) = cmd {
            container.new_tab(None);
        }
        assert_eq!(container.tab_count(), 2);
    }

    /// SwitchToTab(idx) 명령을 TabContainer.switch_tab 에 반영한다.
    #[test]
    fn switch_tab_command_changes_active_tab() {
        use crate::tabs::TabContainer;
        let mut container = TabContainer::new();
        container.new_tab(None); // 탭 1
        container.new_tab(None); // 탭 2

        let mods = platform_only();
        let cmd = dispatch_tab_key(mods, TabKeyCode::Digit(2));
        assert_eq!(cmd, Some(TabCommand::SwitchToTab(1)));

        if let Some(TabCommand::SwitchToTab(idx)) = cmd {
            container.switch_tab(idx).unwrap();
        }
        assert_eq!(container.active_tab_idx, 1);
    }

    /// PrevTab / NextTab 순환 — 탭 3개에서 Next×3 후 Prev×1 이 idx 2 로 복귀.
    #[test]
    fn prev_next_tab_cycle_with_container() {
        use crate::tabs::TabContainer;
        let mut container = TabContainer::new(); // idx 0
        container.new_tab(None); // idx 1 (active 는 1 로 이동)
        container.new_tab(None); // idx 2 (active 는 2 로 이동)
        assert_eq!(container.tab_count(), 3);

        // active 를 0 으로 명시적 이동
        container.switch_tab(0).unwrap();
        assert_eq!(container.active_tab_idx, 0);

        // NextTab 3번: 0→1→2→0 (wrap)
        let mods = platform_only();
        for _ in 0..3 {
            if let Some(TabCommand::NextTab) = dispatch_tab_key(mods, TabKeyCode::BraceClose) {
                let next_idx = (container.active_tab_idx + 1) % container.tab_count();
                container.switch_tab(next_idx).unwrap();
            }
        }
        assert_eq!(container.active_tab_idx, 0);

        // PrevTab 1번: 0→2
        if let Some(TabCommand::PrevTab) = dispatch_tab_key(mods, TabKeyCode::BraceOpen) {
            let prev_idx = if container.active_tab_idx == 0 {
                container.tab_count() - 1
            } else {
                container.active_tab_idx - 1
            };
            container.switch_tab(prev_idx).unwrap();
        }
        assert_eq!(container.active_tab_idx, 2);
    }

    // -------------------------------------------------------
    // T5 AC-R-3: keystroke_to_tab_key 단위 테스트
    // -------------------------------------------------------

    /// macOS: platform modifier(=Cmd) + "t" → (cmd=true, T).
    ///
    /// AC-R-3 관련: keystroke_to_tab_key 가 올바르게 변환해야 dispatch_tab_key → NewTab.
    #[cfg(target_os = "macos")]
    #[test]
    fn keystroke_cmd_t_on_macos_returns_t_keycode() {
        let ks = Keystroke {
            modifiers: gpui::Modifiers {
                platform: true,
                control: false,
                shift: false,
                alt: false,
                function: false,
            },
            key: "t".to_string(),
            key_char: Some("t".to_string()),
        };
        let (mods, code) = keystroke_to_tab_key(&ks);
        assert!(mods.cmd, "macOS platform modifier → cmd=true");
        assert!(!mods.ctrl, "ctrl=false");
        assert_eq!(code, TabKeyCode::T);
        // dispatch_tab_key 까지 연결 확인
        assert_eq!(dispatch_tab_key(mods, code), Some(TabCommand::NewTab));
    }

    /// Linux: control + "t" → (ctrl=true, T).
    ///
    /// AC-R-3 관련: Ctrl+T → NewTab.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn keystroke_ctrl_t_on_linux_returns_t_keycode() {
        let ks = Keystroke {
            modifiers: gpui::Modifiers {
                platform: false,
                control: true,
                shift: false,
                alt: false,
                function: false,
            },
            key: "t".to_string(),
            key_char: Some("t".to_string()),
        };
        let (mods, code) = keystroke_to_tab_key(&ks);
        assert!(!mods.cmd, "Linux: cmd=false");
        assert!(mods.ctrl, "Linux: ctrl=true");
        assert_eq!(code, TabKeyCode::T);
        assert_eq!(dispatch_tab_key(mods, code), Some(TabCommand::NewTab));
    }

    /// backslash 키 → TabKeyCode::Backslash.
    #[test]
    fn keystroke_backslash_returns_backslash_keycode() {
        let ks = Keystroke {
            modifiers: gpui::Modifiers::default(),
            key: "\\".to_string(),
            key_char: None,
        };
        let (_, code) = keystroke_to_tab_key(&ks);
        assert_eq!(code, TabKeyCode::Backslash);
    }

    /// 숫자 1 키 → TabKeyCode::Digit(1).
    #[test]
    fn keystroke_digit_1_returns_digit_1_keycode() {
        let ks = Keystroke {
            modifiers: gpui::Modifiers::default(),
            key: "1".to_string(),
            key_char: Some("1".to_string()),
        };
        let (_, code) = keystroke_to_tab_key(&ks);
        assert_eq!(code, TabKeyCode::Digit(1));
    }

    /// 알 수 없는 키 → TabKeyCode::Other.
    #[test]
    fn keystroke_unknown_key_returns_other() {
        let ks = Keystroke {
            modifiers: gpui::Modifiers::default(),
            key: "q".to_string(),
            key_char: Some("q".to_string()),
        };
        let (_, code) = keystroke_to_tab_key(&ks);
        assert_eq!(code, TabKeyCode::Other);
    }

    // -------------------------------------------------------
    // T5 AC-R-3/AC-R-4: handle_tab_command logic-level 통합 검증
    // -------------------------------------------------------

    /// NewTab 명령 → TabContainer.tabs.len() == 2 (AC-R-3 logic-level).
    #[test]
    fn new_tab_via_keystroke_increments_tab_count() {
        use crate::tabs::TabContainer;

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

        let mut tc = TabContainer::new();
        assert_eq!(tc.tab_count(), 1);
        if let Some(TabCommand::NewTab) = cmd {
            tc.new_tab(None);
        }
        assert_eq!(tc.tab_count(), 2, "AC-R-3: NewTab 후 tabs.len() == 2");
    }

    /// SplitVertical 명령 → 활성 탭 PaneTree 가 Split 으로 교체 (AC-R-4 logic-level).
    #[test]
    fn split_vertical_via_keystroke_changes_pane_tree_to_split() {
        use crate::panes::{PaneId, PaneTree};
        use crate::tabs::TabContainer;

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

        let mut tc = TabContainer::new();
        // 활성 탭의 focused leaf ID 획득
        let focused_id = tc
            .active_tab()
            .last_focused_pane
            .clone()
            .expect("초기 focused pane 있어야 함");

        if let Some(TabCommand::SplitVertical) = cmd {
            // SplitVertical → split_horizontal (좌우 분할) 호출
            tc.active_tab_mut()
                .pane_tree
                .split_horizontal(&focused_id, PaneId::new_unique(), "new-pane".to_string())
                .expect("split_horizontal 성공");
        }

        // AC-R-4: PaneTree 가 Split 으로 교체되었는지 확인
        assert!(
            matches!(tc.active_tab().pane_tree, PaneTree::Split { .. }),
            "AC-R-4: SplitVertical 후 PaneTree 는 Split 이어야 한다"
        );

        // render 관점: split 1 개, divider 1 개
        use crate::panes::render::{count_leaves, count_splits};
        assert_eq!(count_splits(&tc.active_tab().pane_tree), 1, "split 1 개");
        assert_eq!(count_leaves(&tc.active_tab().pane_tree), 2, "leaf 2 개");
    }
}
