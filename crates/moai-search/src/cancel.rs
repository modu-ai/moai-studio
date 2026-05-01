//! Cooperative cancellation token for search workers.
//!
//! Workers clone the token and poll `is_cancelled()` per file entry and per line.
//! The session owner calls `cancel()` to signal all workers to stop.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// @MX:ANCHOR: [AUTO] CancelToken — shared cancellation primitive; expected fan_in >= 3
//             (walk_workspace, SearchSession::cancel_all, SearchPanel cancel button).
// @MX:REASON: All workers and the session manager clone this token.

/// Cooperative cancellation token.
///
/// Wraps an `Arc<AtomicBool>` so that clones share the same underlying flag.
/// Calling [`cancel`] on any clone cancels all clones.
///
/// # Example
///
/// ```rust
/// use moai_search::CancelToken;
///
/// let token = CancelToken::new();
/// let worker_token = token.clone();
/// assert!(!worker_token.is_cancelled());
/// token.cancel();
/// assert!(worker_token.is_cancelled());
/// ```
#[derive(Debug, Clone)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Creates a new, non-cancelled token.
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Signals all clones that work should stop.
    ///
    /// Subsequent calls to [`is_cancelled`] on any clone return `true`.
    /// This operation is idempotent and safe to call from any thread.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Returns `true` if [`cancel`] has been called on any clone.
    ///
    /// Uses `Relaxed` ordering — cancellation is best-effort, not a strict
    /// synchronisation barrier.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests — T4
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// T4a: A freshly created token is not cancelled.
    #[test]
    fn test_cancel_token_default_false() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled(), "new token must not be cancelled");
    }

    /// T4b: Clones share the same underlying flag — cancelling one cancels all.
    #[test]
    fn test_cancel_token_clone_shares_state() {
        let a = CancelToken::new();
        let b = a.clone();
        let c = b.clone();

        a.cancel();

        assert!(b.is_cancelled(), "clone b must observe cancellation");
        assert!(c.is_cancelled(), "clone c must observe cancellation");
    }

    /// T4c: `cancel()` transitions `is_cancelled()` from false to true.
    #[test]
    fn test_cancel_propagates_after_cancel_call() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled(), "must start as not cancelled");
        token.cancel();
        assert!(token.is_cancelled(), "must be cancelled after cancel()");
    }
}
