// @MX:ANCHOR: [AUTO] workspace-watcher-multiplexer
// @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-2 + SPEC-V3-008 미래 통합을 위한 thin wrapper.
//   FsWatcher 를 wrapping 하여 구독자가 여러 명 가능한 broadcast 채널을 노출한다.
//   fan_in >= 2: explorer (V3-005), git UI (V3-008 미래).
// @MX:SPEC: SPEC-V3-005

use std::path::{Path, PathBuf};

use tokio::sync::broadcast;

use crate::{FsEvent, FsWatcher, FsWatcherError, WorkspaceKey};

/// 단일 워크스페이스를 감시하고 이벤트를 broadcast 채널로 발행하는 helper.
///
/// - `FsWatcher` 의 시그니처는 변경하지 않고 thin wrapper 로 추가한다 (REQ-R5).
/// - `subscribe()` 로 여러 구독자(explorer, SPEC-V3-008 등)가 동일 이벤트를 수신 가능.
pub struct WorkspaceWatcher {
    /// 내부 FsWatcher (감시 등록 완료 상태)
    _inner: FsWatcher,
    /// broadcast 송신자 — subscribe() 로 복수 수신자 생성 가능
    tx: broadcast::Sender<(WorkspaceKey, FsEvent)>,
    /// 감시 중인 워크스페이스 키
    workspace_key: WorkspaceKey,
    /// 감시 중인 루트 경로
    pub root: PathBuf,
}

impl WorkspaceWatcher {
    /// 주어진 워크스페이스 키와 루트 경로로 감시를 시작한다.
    ///
    /// 내부적으로 `FsWatcher::new()` 로 감시자를 생성하고 즉시 `root` 경로 감시를 등록한다.
    /// 반환 즉시 이벤트 수신 루프가 백그라운드 스레드에서 시작된다.
    pub fn new(workspace_key: WorkspaceKey, root: &Path) -> Result<Self, FsWatcherError> {
        let (mut watcher, mut rx) = FsWatcher::new()?;
        watcher.watch(root)?;

        // broadcast 채널 — 버퍼 128 이벤트
        let (tx, _initial_rx) = broadcast::channel::<(WorkspaceKey, FsEvent)>(128);
        let tx_clone = tx.clone();
        let key = workspace_key;

        // FsWatcher 의 tokio::mpsc::Receiver 를 broadcast 로 교량
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // 구독자가 없어도 오류 무시 (broadcast 특성)
                let _ = tx_clone.send((key, event));
            }
        });

        Ok(Self {
            _inner: watcher,
            tx,
            workspace_key,
            root: root.to_path_buf(),
        })
    }

    /// 새 구독자를 등록한다. 이후 발행되는 이벤트만 수신한다.
    pub fn subscribe(&self) -> broadcast::Receiver<(WorkspaceKey, FsEvent)> {
        self.tx.subscribe()
    }

    /// 현재 워크스페이스 키를 반환한다.
    pub fn workspace_key(&self) -> WorkspaceKey {
        self.workspace_key
    }
}

// ============================================================
// 단위 테스트 — T3 lifecycle
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn workspace_watcher_creation_succeeds() {
        // WorkspaceWatcher 가 tempdir 경로로 성공적으로 생성되어야 한다
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let result = WorkspaceWatcher::new(42, dir.path());
        assert!(
            result.is_ok(),
            "WorkspaceWatcher 생성 실패: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn workspace_watcher_subscribe_returns_receiver() {
        // subscribe() 가 호출 가능하고 receiver 를 반환해야 한다
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let watcher = WorkspaceWatcher::new(1, dir.path()).expect("WorkspaceWatcher 생성 실패");

        let _rx1 = watcher.subscribe();
        let _rx2 = watcher.subscribe();
        // 두 subscriber 가 등록 가능해야 한다 (broadcast 특성)
        assert_eq!(watcher.tx.receiver_count(), 2);
    }

    #[tokio::test]
    async fn workspace_watcher_key_matches() {
        // workspace_key() 가 생성 시 전달한 키를 반환해야 한다
        let dir = tempdir().expect("임시 디렉토리 생성 실패");
        let watcher = WorkspaceWatcher::new(99, dir.path()).expect("WorkspaceWatcher 생성 실패");

        assert_eq!(watcher.workspace_key(), 99);
    }
}
