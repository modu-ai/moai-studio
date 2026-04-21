//! moai-supervisor: tokio 기반 액터 트리 슈퍼바이저
//!
//! `RootSupervisor` 가 모든 `WorkspaceSupervisor` 를 소유·조율한다. M0 에서는
//! 메모리 내 HashMap 이었지만 M1 부터는 `moai-store` / `moai-git` / `moai-fs`
//! 와 통합되어 실제 생명주기를 관리한다.

#![allow(clippy::new_without_default)]

pub mod lifecycle;
pub mod restore;
pub mod root;
pub mod workspace;

pub use lifecycle::{LifecycleError, WorkspaceCreateRequest};
pub use root::{RootSupervisor, SupervisorError};
pub use workspace::{WorkspaceHandle, WorkspaceId, WorkspaceSnapshot, WorkspaceState};
