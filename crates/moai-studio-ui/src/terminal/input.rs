//! GPUI key event → ANSI bytes → PTY stdin write 경로.
//!
//! SPEC-V3-002 RG-V3-002-4 (키 입력 경로).
//!
//! @MX:TODO: T5 에서 구현 예정.
//! @MX:REASON: GPUI KeyEvent + libghostty_vt::KeyEncoder 연동은 T3 완료 후 가능.
//!   현재는 기본 ASCII 바이트 변환 stub 만 제공한다.
//!   T5 에서: GPUI KeyEvent → KeyEncoder::encode() → ANSI bytes → pty.feed().

/// 키 이름 문자열을 ANSI bytes 로 변환한다 (T5 에서 완성).
///
/// 반환 값이 `None` 이면 PTY stdin write 를 생략한다 (처리 불가 키).
///
/// T5 구현 후에는 GPUI `KeyEvent` 를 직접 수신하여
/// `libghostty_vt::KeyEncoder::encode()` 를 호출한다.
#[allow(dead_code)]
pub fn key_to_ansi_bytes(_key_str: &str) -> Option<Vec<u8>> {
    // TODO(T5): GPUI KeyEvent → libghostty_vt::KeyEncoder::encode() → ANSI bytes
    None
}
