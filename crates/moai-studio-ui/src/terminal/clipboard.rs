//! arboard 3.0 로컬 클립보드 — SIGINT (`Ctrl+C`) 와 복사 경로 명확 분리.
//!
//! SPEC-V3-002 RG-V3-002-4 (로컬 복사 경로).
//!
//! @MX:WARN: clipboard-vs-sigint-split
//! @MX:REASON: macOS `Cmd+C` / Linux `Ctrl+Shift+C` 는 선택 텍스트를 arboard 로 복사한다.
//!   `Ctrl+C` (선택 없음) 는 raw byte 0x03 (SIGINT) 을 PTY stdin 으로 write 한다.
//!   두 경로를 혼동하면 사용자가 의도하지 않은 프로세스 종료가 발생한다.
//!   OSC 52 원격 클립보드는 parser silently ignore (Phase 3 이관, SPEC §6 제외).

use arboard::Clipboard;

/// 선택 텍스트를 시스템 클립보드에 복사한다.
///
/// macOS: `Cmd+C` 트리거.
/// Linux: `Ctrl+Shift+C` 트리거.
///
/// SIGINT (`Ctrl+C`) 경로와 **완전히 별개** — 이 함수는 PTY stdin 을 건드리지 않는다.
/// OSC 52 원격 클립보드 시퀀스는 이 함수에서 처리하지 않는다 (Phase 3 이관).
pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.to_owned())
}

/// SIGINT 바이트 (0x03) 반환 — PTY stdin write 용.
///
/// 호출자: `Ctrl+C` (선택 없음) 이벤트 핸들러.
/// 이 함수는 클립보드를 건드리지 않는다.
pub fn sigint_bytes() -> &'static [u8] {
    b"\x03"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigint_bytes_is_ctrl_c() {
        // Ctrl+C = ASCII 0x03
        assert_eq!(sigint_bytes(), &[0x03]);
    }

    #[test]
    fn sigint_bytes_not_empty() {
        assert!(!sigint_bytes().is_empty());
    }
}
