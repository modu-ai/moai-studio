//! Workspace 경로별 파일 감시 + EventBus 퍼블리싱 (SPEC-M1-001 RG-M1-4, T-008).
//!
//! `FsEventBus` 는 `tokio::sync::broadcast` 채널을 래핑한다. 여러 구독자
//! (UI, hook-http, ide-server) 가 동시에 이벤트를 수신할 수 있다. 각
//! 워크스페이스의 root path 는 `start_watching`/`stop_watching` 로 등록·해제하며,
//! 발행되는 이벤트는 `(WorkspaceKey, FsEvent)` 튜플을 `WorkspaceEvent` 로 래핑한다.

// @MX:ANCHOR: [AUTO] FS → EventBus 단일 발행 지점 (fan_in>=3: supervisor/hook-http/ui)
// @MX:REASON: [AUTO] multiple subscriber 패턴이 필수 — broadcast 채널 교체 금지.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;

use crate::{FsEvent, FsWatcherError};

/// workspace 식별 키 (supervisor 의 WorkspaceId 와 호환되는 u64).
pub type WorkspaceKey = u64;

/// broadcast 채널로 전송되는 이벤트.
#[derive(Debug, Clone)]
pub struct WorkspaceEvent {
    /// 이벤트가 속한 workspace id
    pub workspace: WorkspaceKey,
    /// 파일 시스템 이벤트
    pub event: FsEvent,
}

/// 여러 워크스페이스의 감시자를 모아 관리하는 EventBus.
pub struct FsEventBus {
    tx: broadcast::Sender<WorkspaceEvent>,
    watchers: Mutex<HashMap<WorkspaceKey, RecommendedWatcher>>,
}

impl FsEventBus {
    /// 새 EventBus 를 생성한다. 내부 broadcast 채널 버퍼는 256.
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(256);
        Arc::new(Self {
            tx,
            watchers: Mutex::new(HashMap::new()),
        })
    }

    /// 새 구독자를 등록한다. 반환된 `Receiver` 는 이후 발행되는 이벤트만 수신한다.
    pub fn subscribe(&self) -> broadcast::Receiver<WorkspaceEvent> {
        self.tx.subscribe()
    }

    /// 현재 구독자 수 (테스트/디버깅용).
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// 지정 워크스페이스의 경로 감시를 시작한다. 이미 감시 중이면 재등록한다.
    pub fn start_watching(
        self: &Arc<Self>,
        workspace: WorkspaceKey,
        path: &Path,
    ) -> Result<(), FsWatcherError> {
        let tx = self.tx.clone();
        let (std_tx, std_rx) = std_mpsc::channel::<notify::Event>();

        let mut watcher = notify::RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result {
                    let _ = std_tx.send(event);
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(100)),
        )?;
        watcher.watch(path, RecursiveMode::Recursive)?;

        // 수신 스레드: notify → broadcast
        std::thread::spawn(move || {
            while let Ok(event) = std_rx.recv() {
                let make_event: Option<fn(PathBuf) -> FsEvent> = match event.kind {
                    EventKind::Create(_) => Some(FsEvent::Created),
                    EventKind::Modify(_) => Some(FsEvent::Modified),
                    EventKind::Remove(_) => Some(FsEvent::Removed),
                    _ => None,
                };
                let Some(make) = make_event else { continue };
                for p in event.paths {
                    // 구독자가 없어도 오류로 취급하지 않음 (lagging은 broadcast 특성)
                    let _ = tx.send(WorkspaceEvent {
                        workspace,
                        event: make(p),
                    });
                }
            }
        });

        let mut guard = self
            .watchers
            .lock()
            .map_err(|_| FsWatcherError::ChannelClosed)?;
        guard.insert(workspace, watcher);
        Ok(())
    }

    /// 지정 워크스페이스의 감시를 중단한다. 없으면 `Ok(false)`.
    pub fn stop_watching(&self, workspace: WorkspaceKey) -> Result<bool, FsWatcherError> {
        let mut guard = self
            .watchers
            .lock()
            .map_err(|_| FsWatcherError::ChannelClosed)?;
        Ok(guard.remove(&workspace).is_some())
    }

    /// 현재 감시 중인 워크스페이스 수.
    pub fn watch_count(&self) -> usize {
        self.watchers.lock().map(|g| g.len()).unwrap_or(0)
    }
}
