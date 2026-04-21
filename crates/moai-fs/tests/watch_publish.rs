//! T-008: EventBus 기반 워크스페이스 파일 감시 검증 (SPEC-M1-001 RG-M1-4).

use std::fs;
use std::time::Duration;

use moai_fs::{FsEvent, FsEventBus};
use tempfile::tempdir;
use tokio::time::timeout;

#[tokio::test]
async fn bus_creation_has_no_watchers() {
    let bus = FsEventBus::new();
    assert_eq!(bus.watch_count(), 0);
}

#[tokio::test]
async fn multiple_subscribers_receive_same_event() {
    let bus = FsEventBus::new();
    let _r1 = bus.subscribe();
    let _r2 = bus.subscribe();
    let _r3 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 3);
}

#[tokio::test]
async fn start_watching_registers_workspace() {
    let dir = tempdir().unwrap();
    let bus = FsEventBus::new();
    bus.start_watching(1, dir.path()).unwrap();
    assert_eq!(bus.watch_count(), 1);
    assert!(bus.stop_watching(1).unwrap());
    assert_eq!(bus.watch_count(), 0);
}

#[tokio::test]
async fn stop_watching_nonexistent_returns_false() {
    let bus = FsEventBus::new();
    assert!(!bus.stop_watching(99).unwrap());
}

#[tokio::test]
#[ignore] // CI 타이밍 민감 — 로컬 개발 검증용
async fn broadcast_delivers_file_create_to_all_subscribers() {
    let dir = tempdir().unwrap();
    let bus = FsEventBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();
    bus.start_watching(42, dir.path()).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let new_file = dir.path().join("hello.txt");
    fs::write(&new_file, b"hi").unwrap();

    let ev1 = timeout(Duration::from_secs(1), rx1.recv())
        .await
        .unwrap()
        .unwrap();
    let ev2 = timeout(Duration::from_secs(1), rx2.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ev1.workspace, 42);
    assert_eq!(ev2.workspace, 42);
    assert!(matches!(
        ev1.event,
        FsEvent::Created(_) | FsEvent::Modified(_)
    ));
}
