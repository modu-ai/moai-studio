//! `moai-search` — Global search engine for MoAI Studio.
//!
//! Logic-only crate with zero GPUI dependency. Provides:
//! - [`SearchSession`]: lifecycle manager for multi-workspace parallel searches.
//! - [`SearchHit`]: a single matched line with location and preview.
//! - [`SearchOptions`]: configuration (query, caps, case sensitivity).
//! - [`CancelToken`]: cooperative cancellation token shared across workers.
//! - [`walk_workspace`]: iterator-style workspace walker powered by `ignore`.

pub mod cancel;
pub mod matcher;
pub mod session;
pub mod types;
pub mod walker;

// Public re-exports.
pub use cancel::CancelToken;
pub use matcher::Matcher;
pub use session::SearchSession;
pub use types::{SearchError, SearchHit, SearchOptions};
pub use walker::walk_workspace;
