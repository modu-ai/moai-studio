//! GPUI key event → ANSI bytes 변환 — PTY stdin write 경로.
//!
//! SPEC-V3-002 RG-V3-002-4 (키 입력 경로).
//!
//! @MX:NOTE: key-to-ansi-dispatch
//! 키 조합 분기 우선순위:
//!   1. 클립보드 키 (Cmd+C macOS / Ctrl+Shift+C Linux) — clipboard.rs 로 위임
//!   2. SIGINT (Ctrl+C, 선택 없음) — raw byte 0x03
//!   3. 일반 키 (Enter, Backspace, Arrows, F1-F12, ASCII) — ANSI escape sequence
//!
//! T5 에서는 libghostty_vt::KeyEncoder 없이 표준 ANSI escape 직접 구현.
//! T6(ghostty-spike) 완료 후 KeyEncoder 연동을 검토한다.

use gpui::Keystroke;

/// GPUI `Keystroke` 를 PTY stdin 으로 보낼 ANSI bytes 로 변환한다.
///
/// 반환 값:
/// - `Some(bytes)` — PTY stdin 에 write 할 바이트 열
/// - `None` — 처리 불필요 (클립보드 단축키, 알 수 없는 키 등)
///
/// 클립보드 키 (Cmd+C / Ctrl+Shift+C) 는 이 함수에서 처리하지 않는다.
/// 호출자가 먼저 `is_clipboard_copy` 로 검사해야 한다.
pub fn keystroke_to_ansi_bytes(ks: &Keystroke) -> Option<Vec<u8>> {
    let mods = &ks.modifiers;
    let key = ks.key.as_str();

    // ── Ctrl 조합 키 ──────────────────────────────────────────────────────────
    if mods.control && !mods.platform && !mods.alt {
        return ctrl_key_to_bytes(key, mods.shift);
    }

    // ── Alt/Meta 조합 키 (터미널 escape prefix) ───────────────────────────────
    if mods.alt && !mods.platform && !mods.control {
        // Alt+key → ESC + ANSI(key)
        let inner = plain_key_to_bytes(key, mods.shift)?;
        let mut bytes = vec![0x1b_u8];
        bytes.extend_from_slice(&inner);
        return Some(bytes);
    }

    // ── Cmd/Win/Super 조합 — 터미널 pass-through 아님 → None ─────────────────
    if mods.platform {
        return None;
    }

    // ── 수정자 없는 일반 키 / Shift 조합 ─────────────────────────────────────
    plain_key_to_bytes(key, mods.shift)
}

/// Ctrl+key → ANSI bytes 변환.
///
/// - Ctrl+a~z → 0x01~0x1a (ISO 6429 C0 제어 문자)
/// - Ctrl+[ → ESC (0x1b)
/// - Ctrl+\ → FS (0x1c)
/// - Ctrl+] → GS (0x1d)
/// - Ctrl+^ → RS (0x1e)
/// - Ctrl+_ → US (0x1f)
fn ctrl_key_to_bytes(key: &str, shift: bool) -> Option<Vec<u8>> {
    match key {
        // SIGINT — 호출자(TerminalSurface::handle_key_down)가 먼저 is_sigint 로 처리한다.
        // 여기 도달하는 경우는 선택 영역이 없는 Ctrl+C 이다.
        "c" if !shift => Some(vec![0x03]),
        // Ctrl+D = EOF
        "d" if !shift => Some(vec![0x04]),
        // Ctrl+Z = SIGTSTP
        "z" if !shift => Some(vec![0x1a]),
        // Ctrl+[ = ESC
        "[" => Some(vec![0x1b]),
        // Ctrl+\ = FS
        "\\" => Some(vec![0x1c]),
        // Ctrl+] = GS
        "]" => Some(vec![0x1d]),
        // Ctrl+^ = RS
        "^" => Some(vec![0x1e]),
        // Ctrl+_ = US
        "_" => Some(vec![0x1f]),
        // Ctrl+a~z → 0x01~0x1a
        k if k.len() == 1 => {
            let c = k.chars().next()?.to_ascii_lowercase();
            if c.is_ascii_alphabetic() {
                Some(vec![(c as u8) - b'a' + 1])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 수정자 없는 일반 키 / Shift 조합 → ANSI bytes 변환.
fn plain_key_to_bytes(key: &str, shift: bool) -> Option<Vec<u8>> {
    match key {
        // ── 입력 특수 키 ────────────────────────────────────────────────────
        "enter" => Some(vec![b'\r']), // CR (0x0d)
        "tab" => {
            if shift {
                Some(b"\x1b[Z".to_vec()) // Shift+Tab = CBT
            } else {
                Some(vec![b'\t']) // HT (0x09)
            }
        }
        "backspace" | "delete" if !shift => Some(vec![0x7f]), // DEL
        "backspace" | "delete" if shift => Some(b"\x1b[3~".to_vec()), // Shift+Del
        "escape" => Some(vec![0x1b]),
        "space" => Some(vec![b' ']),

        // ── 방향키 ────────────────────────────────────────────────────────
        "up" => Some(b"\x1b[A".to_vec()),
        "down" => Some(b"\x1b[B".to_vec()),
        "right" => Some(b"\x1b[C".to_vec()),
        "left" => Some(b"\x1b[D".to_vec()),

        // ── 위치 키 ───────────────────────────────────────────────────────
        "home" => Some(b"\x1b[H".to_vec()),
        "end" => Some(b"\x1b[F".to_vec()),
        "pageup" => Some(b"\x1b[5~".to_vec()),
        "pagedown" => Some(b"\x1b[6~".to_vec()),
        "insert" => Some(b"\x1b[2~".to_vec()),

        // ── 함수키 F1-F12 ──────────────────────────────────────────────────
        "f1" => Some(b"\x1bOP".to_vec()),
        "f2" => Some(b"\x1bOQ".to_vec()),
        "f3" => Some(b"\x1bOR".to_vec()),
        "f4" => Some(b"\x1bOS".to_vec()),
        "f5" => Some(b"\x1b[15~".to_vec()),
        "f6" => Some(b"\x1b[17~".to_vec()),
        "f7" => Some(b"\x1b[18~".to_vec()),
        "f8" => Some(b"\x1b[19~".to_vec()),
        "f9" => Some(b"\x1b[20~".to_vec()),
        "f10" => Some(b"\x1b[21~".to_vec()),
        "f11" => Some(b"\x1b[23~".to_vec()),
        "f12" => Some(b"\x1b[24~".to_vec()),

        // ── 일반 ASCII 문자 ─────────────────────────────────────────────────
        k if k.len() == 1 => {
            let c = k.chars().next()?;
            let out_char = if shift { c.to_ascii_uppercase() } else { c };
            let mut buf = [0u8; 4];
            let encoded = out_char.encode_utf8(&mut buf);
            Some(encoded.as_bytes().to_vec())
        }

        // ── 알 수 없는 키 ──────────────────────────────────────────────────
        _ => None,
    }
}

/// 클립보드 복사 단축키인지 판별한다.
///
/// - macOS: `Cmd+C` (modifiers.platform && key=="c")
/// - Linux/Windows: `Ctrl+Shift+C` (control && shift && key=="c")
///
/// 이 키 조합은 PTY stdin 전송을 하지 않고 arboard 복사 경로로 처리한다.
/// SIGINT (Ctrl+C 선택 없음) 와 **명확히 구분**된다.
pub fn is_clipboard_copy(ks: &Keystroke) -> bool {
    let mods = &ks.modifiers;
    let key = ks.key.as_str();

    // macOS Cmd+C
    let macos_copy =
        mods.platform && !mods.control && !mods.shift && !mods.alt && key.eq_ignore_ascii_case("c");

    // Linux Ctrl+Shift+C
    let linux_copy =
        mods.control && mods.shift && !mods.platform && !mods.alt && key.eq_ignore_ascii_case("c");

    macos_copy || linux_copy
}

/// SIGINT 단축키인지 판별한다 (Ctrl+C, 수정자 단독).
///
/// 선택 영역이 없을 때만 SIGINT 로 처리한다.
/// 선택 영역 있을 때 Ctrl+C 는 클립보드 copy 로 처리한다 (플랫폼 따라 상이).
pub fn is_sigint(ks: &Keystroke) -> bool {
    let mods = &ks.modifiers;
    mods.control && !mods.platform && !mods.shift && !mods.alt && ks.key.eq_ignore_ascii_case("c")
}

// ============================================================
// 유닛 테스트 — keystroke_to_ansi_bytes
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Modifiers;

    fn ks(key: &str) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: None,
        }
    }

    fn ks_mod(key: &str, mods: Modifiers) -> Keystroke {
        Keystroke {
            modifiers: mods,
            key: key.to_string(),
            key_char: None,
        }
    }

    fn ctrl(key: &str) -> Keystroke {
        ks_mod(
            key,
            Modifiers {
                control: true,
                ..Default::default()
            },
        )
    }

    fn cmd(key: &str) -> Keystroke {
        ks_mod(
            key,
            Modifiers {
                platform: true,
                ..Default::default()
            },
        )
    }

    fn ctrl_shift(key: &str) -> Keystroke {
        ks_mod(
            key,
            Modifiers {
                control: true,
                shift: true,
                ..Default::default()
            },
        )
    }

    // --- 일반 문자 ---

    #[test]
    fn ascii_char_a_produces_byte_97() {
        let bytes = keystroke_to_ansi_bytes(&ks("a")).unwrap();
        assert_eq!(bytes, b"a");
    }

    #[test]
    fn ascii_char_z_produces_byte_122() {
        let bytes = keystroke_to_ansi_bytes(&ks("z")).unwrap();
        assert_eq!(bytes, b"z");
    }

    // --- 특수 키 ---

    #[test]
    fn enter_produces_carriage_return() {
        let bytes = keystroke_to_ansi_bytes(&ks("enter")).unwrap();
        assert_eq!(bytes, b"\r");
    }

    #[test]
    fn tab_produces_horizontal_tab() {
        let bytes = keystroke_to_ansi_bytes(&ks("tab")).unwrap();
        assert_eq!(bytes, b"\t");
    }

    #[test]
    fn escape_produces_esc_byte() {
        let bytes = keystroke_to_ansi_bytes(&ks("escape")).unwrap();
        assert_eq!(bytes, &[0x1b]);
    }

    #[test]
    fn backspace_produces_del_byte() {
        let bytes = keystroke_to_ansi_bytes(&ks("backspace")).unwrap();
        assert_eq!(bytes, &[0x7f]);
    }

    // --- 방향키 ---

    #[test]
    fn arrow_up_produces_csi_a() {
        let bytes = keystroke_to_ansi_bytes(&ks("up")).unwrap();
        assert_eq!(bytes, b"\x1b[A");
    }

    #[test]
    fn arrow_down_produces_csi_b() {
        let bytes = keystroke_to_ansi_bytes(&ks("down")).unwrap();
        assert_eq!(bytes, b"\x1b[B");
    }

    #[test]
    fn arrow_right_produces_csi_c() {
        let bytes = keystroke_to_ansi_bytes(&ks("right")).unwrap();
        assert_eq!(bytes, b"\x1b[C");
    }

    #[test]
    fn arrow_left_produces_csi_d() {
        let bytes = keystroke_to_ansi_bytes(&ks("left")).unwrap();
        assert_eq!(bytes, b"\x1b[D");
    }

    // --- Ctrl 조합 ---

    #[test]
    fn ctrl_a_produces_0x01() {
        let bytes = keystroke_to_ansi_bytes(&ctrl("a")).unwrap();
        assert_eq!(bytes, &[0x01]);
    }

    #[test]
    fn ctrl_c_produces_sigint_byte() {
        let bytes = keystroke_to_ansi_bytes(&ctrl("c")).unwrap();
        assert_eq!(bytes, &[0x03]);
    }

    #[test]
    fn ctrl_d_produces_eof_byte() {
        let bytes = keystroke_to_ansi_bytes(&ctrl("d")).unwrap();
        assert_eq!(bytes, &[0x04]);
    }

    #[test]
    fn ctrl_z_produces_sigtstp_byte() {
        let bytes = keystroke_to_ansi_bytes(&ctrl("z")).unwrap();
        assert_eq!(bytes, &[0x1a]);
    }

    // --- Cmd/Win → None (클립보드 처리 전 단계) ---

    #[test]
    fn cmd_c_returns_none_for_ansi_encoding() {
        // Cmd+C 는 clipboard.rs 경로로 처리 → ANSI bytes 없음
        assert!(keystroke_to_ansi_bytes(&cmd("c")).is_none());
    }

    #[test]
    fn cmd_v_returns_none_for_ansi_encoding() {
        assert!(keystroke_to_ansi_bytes(&cmd("v")).is_none());
    }

    // --- 클립보드 단축키 판별 ---

    #[test]
    fn is_clipboard_copy_detects_cmd_c_macos() {
        assert!(is_clipboard_copy(&cmd("c")));
    }

    #[test]
    fn is_clipboard_copy_detects_ctrl_shift_c_linux() {
        assert!(is_clipboard_copy(&ctrl_shift("c")));
    }

    #[test]
    fn is_clipboard_copy_rejects_plain_c() {
        assert!(!is_clipboard_copy(&ks("c")));
    }

    #[test]
    fn is_clipboard_copy_rejects_ctrl_c_without_shift() {
        assert!(!is_clipboard_copy(&ctrl("c")));
    }

    // --- SIGINT 판별 ---

    #[test]
    fn is_sigint_detects_ctrl_c() {
        assert!(is_sigint(&ctrl("c")));
    }

    #[test]
    fn is_sigint_rejects_cmd_c() {
        assert!(!is_sigint(&cmd("c")));
    }

    #[test]
    fn is_sigint_rejects_ctrl_shift_c() {
        assert!(!is_sigint(&ctrl_shift("c")));
    }

    // --- 함수키 ---

    #[test]
    fn f1_produces_ss3_p() {
        let bytes = keystroke_to_ansi_bytes(&ks("f1")).unwrap();
        assert_eq!(bytes, b"\x1bOP");
    }

    #[test]
    fn f5_produces_csi_15_tilde() {
        let bytes = keystroke_to_ansi_bytes(&ks("f5")).unwrap();
        assert_eq!(bytes, b"\x1b[15~");
    }

    // --- 위치키 ---

    #[test]
    fn home_produces_csi_h() {
        let bytes = keystroke_to_ansi_bytes(&ks("home")).unwrap();
        assert_eq!(bytes, b"\x1b[H");
    }

    #[test]
    fn pageup_produces_csi_5_tilde() {
        let bytes = keystroke_to_ansi_bytes(&ks("pageup")).unwrap();
        assert_eq!(bytes, b"\x1b[5~");
    }
}
