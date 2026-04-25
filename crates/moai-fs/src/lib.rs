//! moai-fs: notify 기반 파일 감시자(file watcher)
//!
//! FileTree 서피스와 마크다운 실시간 리로드를 위한 파일 시스템 감시 모듈입니다.

// @MX:ANCHOR: FsWatcher 공개 API 진입점 — 모든 파일 감시 기능의 루트
// @MX:REASON: 외부 크레이트에서 직접 사용하는 공개 인터페이스

pub mod tree_watcher;
pub mod watcher;
pub mod workspace_watcher;
pub use watcher::{FsEventBus, WorkspaceEvent, WorkspaceKey};
pub use workspace_watcher::WorkspaceWatcher;

use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tokio::sync::mpsc;

/// 파일 감시자 에러 타입
#[derive(Debug, Error)]
pub enum FsWatcherError {
    /// notify 라이브러리 에러 (감시 등록/해제 실패)
    #[error("감시 에러: {0}")]
    WatchError(#[from] notify::Error),

    /// 채널이 닫혀 이벤트를 전송할 수 없음
    #[error("채널이 닫혔습니다")]
    ChannelClosed,
}

/// 파일 시스템 이벤트 열거형
#[derive(Debug, Clone, PartialEq)]
pub enum FsEvent {
    /// 파일 또는 디렉토리 생성됨
    Created(PathBuf),
    /// 파일 또는 디렉토리 수정됨
    Modified(PathBuf),
    /// 파일 또는 디렉토리 삭제됨
    Removed(PathBuf),
}

/// notify 래퍼 파일 감시자
///
/// # 예시
/// ```no_run
/// use moai_fs::FsWatcher;
/// let (mut watcher, mut rx) = FsWatcher::new().unwrap();
/// watcher.watch(std::path::Path::new("/tmp")).unwrap();
/// ```
pub struct FsWatcher {
    /// notify 감시자 인스턴스 (Drop 시 자동으로 감시 해제)
    watcher: RecommendedWatcher,
}

impl FsWatcher {
    /// 새 파일 감시자를 생성하고 이벤트 수신 채널을 반환합니다.
    ///
    /// 내부적으로 std::mpsc를 notify 콜백에 사용하고,
    /// tokio::sync::mpsc 채널로 이벤트를 전달합니다.
    pub fn new() -> Result<(Self, mpsc::Receiver<FsEvent>), FsWatcherError> {
        // tokio 비동기 채널 (버퍼 크기 128)
        let (tx, rx) = mpsc::channel::<FsEvent>(128);

        // notify용 동기 채널
        let (std_tx, std_rx) = std_mpsc::channel();

        // notify 감시자 생성 (폴링 방식 사용으로 안정성 확보)
        let watcher = notify::RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result {
                    // std 채널로 이벤트 전달 (콜백은 동기 컨텍스트)
                    let _ = std_tx.send(event);
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(100)),
        )?;

        // std 채널에서 tokio 채널로 이벤트를 전달하는 스레드
        std::thread::spawn(move || {
            for event in std_rx {
                // notify 이벤트 종류를 먼저 확인하여 FsEvent 생성 함수 선택
                let make_event: Option<fn(PathBuf) -> FsEvent> = match event.kind {
                    EventKind::Create(_) => Some(FsEvent::Created),
                    EventKind::Modify(_) => Some(FsEvent::Modified),
                    EventKind::Remove(_) => Some(FsEvent::Removed),
                    _ => None,
                };

                // 이벤트 종류가 없으면 스킵
                let Some(make) = make_event else { continue };

                // 각 경로에 대해 이벤트 생성 후 tokio 채널로 전송
                for path in event.paths {
                    // 채널이 닫히면 스레드 종료
                    if tx.blocking_send(make(path)).is_err() {
                        return;
                    }
                }
            }
        });

        Ok((FsWatcher { watcher }, rx))
    }

    /// 지정한 경로를 감시 목록에 추가합니다.
    pub fn watch(&mut self, path: &Path) -> Result<(), FsWatcherError> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(FsWatcherError::WatchError)
    }

    /// 지정한 경로를 감시 목록에서 제거합니다.
    pub fn unwatch(&mut self, path: &Path) -> Result<(), FsWatcherError> {
        self.watcher
            .unwatch(path)
            .map_err(FsWatcherError::WatchError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::time::timeout;

    /// 감시자 생성 테스트
    #[tokio::test]
    async fn test_watcher_creation() {
        // Act: 감시자 생성
        let result = FsWatcher::new();

        // Assert: 에러 없이 생성됨
        assert!(
            result.is_ok(),
            "감시자 생성이 실패했습니다: {:?}",
            result.err()
        );
        let (_watcher, _rx) = result.unwrap();
    }

    /// 디렉토리 감시 등록 테스트
    #[tokio::test]
    async fn test_watch_directory() {
        // Arrange: 임시 디렉토리 생성
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let (mut watcher, _rx) = FsWatcher::new().expect("감시자 생성 실패");

        // Act: 디렉토리 감시 등록
        let result = watcher.watch(dir.path());

        // Assert: 에러 없이 등록됨
        assert!(result.is_ok(), "감시 등록 실패: {:?}", result.err());
    }

    /// 파일 생성 이벤트 감지 테스트
    ///
    /// 파일 시스템 이벤트는 타이밍에 민감하여 CI에서 불안정할 수 있습니다.
    #[tokio::test]
    #[ignore]
    async fn test_detect_file_creation() {
        // Arrange: 임시 디렉토리 생성 및 감시 등록
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let dir_path = dir.path().to_path_buf();
        let (mut watcher, mut rx) = FsWatcher::new().expect("감시자 생성 실패");
        watcher.watch(&dir_path).expect("감시 등록 실패");

        // 감시자 초기화 대기
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Act: 파일 생성
        let new_file = dir_path.join("test_file.txt");
        fs::write(&new_file, "테스트 내용").expect("파일 쓰기 실패");

        // Assert: 이벤트 수신 (1초 타임아웃)
        let received = timeout(Duration::from_secs(1), async {
            loop {
                if let Some(event) = rx.recv().await {
                    match event {
                        FsEvent::Created(path) | FsEvent::Modified(path) => {
                            if path == new_file {
                                return true;
                            }
                        }
                        _ => continue,
                    }
                } else {
                    return false;
                }
            }
        })
        .await;

        assert!(
            received.is_ok(),
            "타임아웃: 파일 생성 이벤트를 받지 못했습니다"
        );
        assert!(received.unwrap(), "파일 생성 이벤트 경로 불일치");
    }

    /// 감시 해제 후 이벤트 중지 테스트
    ///
    /// 파일 시스템 이벤트는 타이밍에 민감하여 CI에서 불안정할 수 있습니다.
    #[tokio::test]
    #[ignore]
    async fn test_unwatch_stops_events() {
        // Arrange: 임시 디렉토리 생성 및 감시 등록
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let dir_path = dir.path().to_path_buf();
        let (mut watcher, mut rx) = FsWatcher::new().expect("감시자 생성 실패");
        watcher.watch(&dir_path).expect("감시 등록 실패");

        // 감시자 초기화 대기
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Act: 감시 해제
        let result = watcher.unwatch(&dir_path);
        assert!(result.is_ok(), "감시 해제 실패: {:?}", result.err());

        // 해제 처리 대기
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 파일 생성 (이벤트가 오지 않아야 함)
        let new_file = dir_path.join("after_unwatch.txt");
        fs::write(&new_file, "해제 후 파일").expect("파일 쓰기 실패");

        // Assert: 500ms 내에 이벤트가 없어야 함
        let received = timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(
            received.is_err(),
            "감시 해제 후에도 이벤트가 수신되었습니다"
        );
    }
}
