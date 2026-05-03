//! arboard 3.0 local clipboard — SIGINT (`Ctrl+C`) and copy paths clearly separated.
//!
//! SPEC-V3-002 RG-V3-002-4 (local copy path).
//! SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 (REQ-OL-008/009/010): ClipboardWriter trait.
//!
//! @MX:WARN: clipboard-vs-sigint-split
//! @MX:REASON: macOS `Cmd+C` / Linux `Ctrl+Shift+C` copies selected text to arboard.
//!   `Ctrl+C` (no selection) writes raw byte 0x03 (SIGINT) to PTY stdin.
//!   Confusing the two paths causes unintended process termination.
//!   OSC 52 remote clipboard is parser silently ignored (Phase 3 carry, SPEC §6 excluded).

use arboard::Clipboard;
use std::sync::{Arc, Mutex};

// @MX:NOTE: [AUTO] clipboard-writer-trait-injection
// @MX:SPEC: SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 (REQ-OL-008)
// Polymorphic clipboard injection point for testing and production use.
// Fan-in: handle_click_for_copy + copy_url_at helper + tests → eligible for NOTE.

/// Clipboard write abstraction — allows injection of mock implementations in tests.
///
/// SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 (REQ-OL-008): `Send + Sync` bound enables
/// storage in `Box<dyn ClipboardWriter + Send + Sync>` on `TerminalSurface`.
pub trait ClipboardWriter: Send + Sync {
    /// Write `text` to the system clipboard.
    fn write(&self, text: &str) -> Result<(), arboard::Error>;
}

/// Production clipboard writer backed by arboard.
///
/// SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 (REQ-OL-009): delegates to the existing
/// `copy_to_clipboard` free function — does NOT call `arboard::Clipboard::new()`
/// directly so the single entry point is preserved.
#[derive(Default)]
pub struct ArboardClipboardWriter;

impl ClipboardWriter for ArboardClipboardWriter {
    fn write(&self, text: &str) -> Result<(), arboard::Error> {
        copy_to_clipboard(text)
    }
}

/// Test-only clipboard writer that captures written strings for assertion.
///
/// SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 (REQ-OL-010): stores every `write` call in
/// an `Arc<Mutex<Vec<String>>>` so that the captured history is accessible even after
/// the writer is moved into a `Box<dyn ClipboardWriter>`.
#[derive(Clone, Default)]
pub struct MockClipboardWriter {
    captured: Arc<Mutex<Vec<String>>>,
}

impl ClipboardWriter for MockClipboardWriter {
    fn write(&self, text: &str) -> Result<(), arboard::Error> {
        let mut guard = self
            .captured
            .lock()
            .expect("MockClipboardWriter lock poisoned");
        guard.push(text.to_owned());
        Ok(())
    }
}

impl MockClipboardWriter {
    /// Returns a snapshot of all strings written so far, in order.
    pub fn contents(&self) -> Vec<String> {
        self.captured
            .lock()
            .expect("MockClipboardWriter lock poisoned")
            .clone()
    }
}

/// Copies selected text to the system clipboard.
///
/// macOS: `Cmd+C` trigger.
/// Linux: `Ctrl+Shift+C` trigger.
///
/// SIGINT (`Ctrl+C`) path is **completely separate** — this function does not touch
/// PTY stdin.  OSC 52 remote clipboard sequences are not handled here (Phase 3 carry).
pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.to_owned())
}

/// Returns SIGINT byte (0x03) — for PTY stdin write.
///
/// Caller: `Ctrl+C` (no selection) event handler.
/// This function does not touch the clipboard.
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

    // ── SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-2 — ClipboardWriter trait tests ──

    /// AC-OL-8: ClipboardWriter + Send + Sync bounds are satisfied by MockClipboardWriter.
    #[test]
    fn clipboard_writer_trait_object_compiles() {
        let _: Box<dyn ClipboardWriter + Send + Sync> = Box::new(MockClipboardWriter::default());
    }

    /// Default MockClipboardWriter has no captured writes.
    #[test]
    fn mock_clipboard_writer_default_contents_empty() {
        let m = MockClipboardWriter::default();
        assert!(m.contents().is_empty());
    }

    /// A single write is captured correctly.
    #[test]
    fn mock_clipboard_writer_captures_single_write() {
        let m = MockClipboardWriter::default();
        m.write("hello").unwrap();
        assert_eq!(m.contents(), vec!["hello"]);
    }

    /// AC-OL-9: Multiple writes are stored in insertion order.
    #[test]
    fn mock_clipboard_writer_captures_multiple_writes_in_order() {
        let m = MockClipboardWriter::default();
        m.write("a").unwrap();
        m.write("b").unwrap();
        m.write("c").unwrap();
        assert_eq!(m.contents(), vec!["a", "b", "c"]);
    }

    /// AC-OL-10: ArboardClipboardWriter can be instantiated and boxed as a trait object.
    /// Actual `write` is not called in CI (headless environment has no display server).
    #[test]
    fn arboard_clipboard_writer_can_be_instantiated() {
        let _ = ArboardClipboardWriter;
        let _: Box<dyn ClipboardWriter + Send + Sync> = Box::new(ArboardClipboardWriter);
    }
}
