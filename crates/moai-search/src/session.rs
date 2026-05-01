//! Search session lifecycle — spawns per-workspace workers and manages cancellation.
//!
//! `SearchSession` owns the worker threads and the shared `CancelToken`.
//! Dropping the session does not automatically cancel workers; call
//! `cancel_all()` explicitly before drop if early termination is needed.

use std::thread::{self, JoinHandle};

use crate::{cancel::CancelToken, types::SearchOptions};

// @MX:ANCHOR: [AUTO] SearchSession — central session object; fan_in >= 3
//             (MS-2 SearchPanel, tests, future MS-3 navigation abort).
// @MX:REASON: All worker threads are owned and cancelled through this struct.

/// Manages the lifecycle of a multi-workspace search.
///
/// # Usage
///
/// ```rust,no_run
/// use moai_search::{SearchSession, SearchOptions};
/// use std::path::PathBuf;
///
/// let opts = SearchOptions { query: "TODO".to_string(), ..Default::default() };
/// let mut session = SearchSession::new(opts);
/// session.spawn_workers(vec![("ws-1".to_string(), PathBuf::from("/workspace"))]);
/// // ... collect results from the channel ...
/// session.cancel_all();
/// ```
pub struct SearchSession {
    /// Configuration used for all workers in this session.
    opts: SearchOptions,
    /// Shared cancellation token distributed to all workers.
    cancel: CancelToken,
    /// Handles for all spawned worker threads.
    handles: Vec<JoinHandle<()>>,
}

impl SearchSession {
    /// Creates a new session with the given options.
    ///
    /// Workers are not started until [`spawn_workers`] is called.
    pub fn new(opts: SearchOptions) -> Self {
        Self {
            opts,
            cancel: CancelToken::new(),
            handles: Vec::new(),
        }
    }

    /// Returns a clone of the session's cancellation token.
    ///
    /// Can be passed to any code that needs to monitor or trigger cancellation.
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel.clone()
    }

    /// Spawns one worker thread per workspace.
    ///
    /// Each worker receives a clone of the `CancelToken` so that calling
    /// [`cancel_all`] interrupts all workers at their next poll point.
    ///
    /// # Parameters
    ///
    /// - `workspaces`: `(workspace_id, root_path)` pairs for each active workspace.
    ///
    /// Results are currently collected within the thread (future MS-2 will
    /// wire an `mpsc::channel` — REQ-GS-020).
    pub fn spawn_workers(&mut self, workspaces: Vec<(String, std::path::PathBuf)>) {
        for (workspace_id, root) in workspaces {
            let opts = self.opts.clone();
            let cancel = self.cancel.clone();

            let handle = thread::spawn(move || {
                match crate::walker::walk_workspace(&root, &workspace_id, &opts, &cancel) {
                    Ok(_hits) => {
                        // TODO(MS-2): send hits through mpsc channel.
                        tracing::debug!(workspace_id, hits = _hits.len(), "worker completed");
                    }
                    Err(err) => {
                        tracing::warn!(workspace_id, %err, "worker error");
                    }
                }
            });

            self.handles.push(handle);
        }
    }

    /// Signals all workers to stop and waits for them to finish.
    ///
    /// After this call `is_cancelled()` on the shared token returns `true`.
    pub fn cancel_all(&mut self) {
        self.cancel.cancel();
        self.join_all();
    }

    /// Waits for all worker threads to finish without cancelling them.
    pub fn join_all(&mut self) {
        for handle in self.handles.drain(..) {
            if let Err(err) = handle.join() {
                tracing::warn!(?err, "worker thread panicked");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — T12
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// T12: SearchSession spawns workers and cancel_all() stops them cleanly.
    #[test]
    fn test_search_session_spawn_and_cancel() {
        let dir = tempdir().expect("tempdir creation failed");
        let root = dir.path().to_owned();

        // Write a few files so the workers have something to do.
        fs::write(root.join("a.rs"), "use std::io;\n").unwrap();
        fs::write(root.join("b.rs"), "use std::fmt;\n").unwrap();

        let opts = SearchOptions {
            query: "use".to_string(),
            ..Default::default()
        };

        let mut session = SearchSession::new(opts);

        // Verify token starts uncancelled.
        assert!(
            !session.cancel_token().is_cancelled(),
            "session token must start uncancelled"
        );

        // Spawn workers for one workspace.
        session.spawn_workers(vec![("ws-session-test".to_string(), root)]);

        // cancel_all must propagate cancellation and join cleanly.
        session.cancel_all();

        // After cancel_all() the token must be set.
        assert!(
            session.cancel_token().is_cancelled(),
            "cancel token must be set after cancel_all()"
        );

        // All handles must have been joined (handles vec is drained).
        assert!(
            session.handles.is_empty(),
            "all worker handles must be joined after cancel_all()"
        );
    }
}
