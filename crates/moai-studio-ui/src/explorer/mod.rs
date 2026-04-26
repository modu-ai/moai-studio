// @MX:ANCHOR: [AUTO] explorer-module-root
// @MX:REASON: [AUTO] SPEC-V3-005 explorer 모듈의 단일 진입점.
//   fan_in >= 3: lib.rs (RootView 필드), 미래 integration tests, SPEC-V3-008 향후 참조.
// @MX:SPEC: SPEC-V3-005

pub mod config;
pub mod context_menu;
pub mod dnd;
pub mod git_status;
pub mod path;
pub mod search;
pub mod tree;
pub mod view;
pub mod watch;

pub use config::FsConfig;
pub use context_menu::{ContextAction, ContextMenu, ContextTarget, InlineEdit, InlineEditKind};
pub use dnd::{DragPayload, DropError};
pub use git_status::{GitStatus, GitStatusProvider, MoaiGitStatusProvider, roll_up_priority};
pub use tree::{FsError, FsNode};
pub use view::{FileExplorer, FileOpenEvent};
pub use watch::{FsDelta, WatchDebouncer};
