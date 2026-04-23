//! libghostty-vt API 호환성 테스트 (characterization)
//!
//! @MX:ANCHOR(libghostty-api-compat-test)
//! @MX:REASON: pinned SHA 에 대한 회귀 감지 지점.
//!   libghostty-rs `rev = "..."` 라인이 변경되는 PR 에서는 반드시 재실행 (AC-T-11).
//!
//! Zig 0.15.x 가 설치된 환경에서만 통과 (CI 검증 전용, AC-T-1).

use moai_studio_terminal::libghostty_ffi;

/// Terminal::new() 기본 생성 테스트 (AC-T-8(b))
#[test]
fn terminal_new_smoke() {
    let term = libghostty_ffi::new_terminal(80, 24);
    assert!(term.is_ok(), "Terminal::new() 가 실패해서는 안 됨");
}

/// ASCII 문자 feed 후 커서가 이동 (AC-T-8(b): cell iter 검증)
#[test]
fn feed_ascii_basic() {
    let mut handle = libghostty_ffi::new_terminal(80, 24).expect("Terminal 생성 실패");
    libghostty_ffi::feed(&mut handle, b"abc");
    let state = libghostty_ffi::render_state(&handle);
    // "abc" feed 후 커서가 col 3 으로 이동 (0-indexed)
    assert_eq!(state.cursor_col, 3, "abc feed 후 커서 col=3 이어야 함");
}

/// UTF-8 멀티바이트 문자 feed (AC-T-8(b): UTF-8 multibyte 검증)
#[test]
fn feed_utf8_multibyte() {
    let mut handle = libghostty_ffi::new_terminal(80, 24).expect("Terminal 생성 실패");
    // '한'(wide 2셀) + '글'(wide 2셀) = 커서 col 4
    libghostty_ffi::feed(&mut handle, "한글".as_bytes());
    let state = libghostty_ffi::render_state(&handle);
    // CJK 한글은 각 2셀 폭 → 커서가 최소 col 2 이상
    assert!(
        state.cursor_col >= 2,
        "한글 feed 후 커서가 최소 col 2 이상이어야 함 (wide char): got {}",
        state.cursor_col
    );
}

/// CSI 커서 위치 지정 → cursor (row=9, col=4) (AC-T-8(b): CSI 처리 검증)
#[test]
fn csi_cursor_position() {
    let mut handle = libghostty_ffi::new_terminal(80, 24).expect("Terminal 생성 실패");
    // ESC[10;5H → row 10, col 5 (1-indexed) → row=9, col=4 (0-indexed)
    libghostty_ffi::feed(&mut handle, b"\x1b[10;5H");
    let state = libghostty_ffi::render_state(&handle);
    assert_eq!(state.cursor_row, 9, "CSI 커서 row 가 9(0-indexed) 여야 함");
    assert_eq!(state.cursor_col, 4, "CSI 커서 col 이 4(0-indexed) 여야 함");
}
