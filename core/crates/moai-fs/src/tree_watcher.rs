//! FileTree 전용 Watcher 스켈레톤 (SPEC-M2-001 MS-4 T-051).
//!
//! @MX:NOTE: [AUTO] MS-4 에서 폴링 방식(500ms) 채택.
//!            Swift FileTreeSurface 가 list_directory_json 을 반복 호출한다.
//!            MS-7+ 에서 notify-push 이벤트 기반으로 업그레이드 예정.
//!            이 모듈은 미래 push 방식을 위한 자리 표시자다.

// TODO(MS-7): notify 기반 push 이벤트 구현

/// FileTree 이벤트 종류.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeEventKind {
    Created,
    Modified,
    Removed,
}

/// FileTree 이벤트.
#[derive(Debug, Clone)]
pub struct FileTreeEvent {
    /// 워크스페이스 루트 기준 상대 경로
    pub path: String,
    pub kind: TreeEventKind,
}
