// @MX:ANCHOR: [AUTO] explorer-module-root
// @MX:REASON: [AUTO] SPEC-V3-005 MS-1 explorer 모듈의 단일 진입점.
//   fan_in >= 3: lib.rs (RootView 필드), 미래 integration tests, SPEC-V3-008 향후 참조.
// @MX:SPEC: SPEC-V3-005

pub mod path;
pub mod tree;
pub mod view;

pub use tree::{FsError, FsNode};
pub use view::FileExplorer;
