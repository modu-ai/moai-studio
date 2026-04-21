//! T-004 RED→GREEN: send_user_message + subscribe_events + poll_event
//!
//! 목표:
//! - tokio::spawn 기반 비동기 발행 → sync poll 소비가 동작해야 한다.
//! - FFI 경계 호출 오버헤드가 <1ms (10k 호출 기준) 이어야 한다.

use moai_ffi::RustCore;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn wait_for_event(core: &RustCore, ws: &str, max_wait: Duration) -> Option<String> {
    let start = Instant::now();
    loop {
        if let Some(ev) = core.poll_event(ws.to_string()) {
            return Some(ev);
        }
        if start.elapsed() > max_wait {
            return None;
        }
        sleep(Duration::from_millis(1));
    }
}

#[test]
fn send_message_then_poll_receives_event() {
    let core = RustCore::new();
    let ws = core.create_workspace("unit".into(), "/tmp/unit".into());
    assert!(core.subscribe_events(ws.clone()));
    assert!(core.send_user_message(ws.clone(), "hello".into()));

    let ev = wait_for_event(&core, &ws, Duration::from_millis(500))
        .expect("이벤트가 500ms 내에 도착해야 함");
    assert!(ev.contains("\"type\":\"user_message\""));
    assert!(ev.contains("hello"));
}

#[test]
fn send_message_to_missing_workspace_is_rejected() {
    let core = RustCore::new();
    assert!(!core.send_user_message("missing".into(), "x".into()));
}

#[test]
fn subscribe_to_missing_workspace_is_rejected() {
    let core = RustCore::new();
    assert!(!core.subscribe_events("missing".into()));
}

#[test]
fn poll_event_returns_none_when_queue_empty() {
    let core = RustCore::new();
    let ws = core.create_workspace("q".into(), "/tmp/q".into());
    assert_eq!(core.poll_event(ws), None);
}

/// NFR-FFI: FFI 호출 오버헤드 <1ms (평균)
///
/// `version()` 은 가장 단순한 FFI 호출이므로 경계 오버헤드의 하한을 측정한다.
/// 10_000 회 호출 / 총 시간 / 호출당 평균을 기록하며, 1ms 를 초과하면 실패.
// @MX:ANCHOR: [AUTO] NFR-FFI 검증 앵커
// @MX:REASON: [AUTO] FFI 경계 오버헤드 <1ms 비기능 요구사항 회귀 탐지 (NFR §5)
#[test]
fn ffi_call_overhead_under_one_millisecond() {
    let core = RustCore::new();
    const ITERS: u32 = 10_000;

    let start = Instant::now();
    for _ in 0..ITERS {
        let _v = core.version();
    }
    let elapsed = start.elapsed();
    let per_call = elapsed / ITERS;

    eprintln!(
        "FFI version() {} 회 총 {:?}, 평균 {:?}",
        ITERS, elapsed, per_call
    );
    assert!(
        per_call < Duration::from_millis(1),
        "FFI 호출당 오버헤드 {:?} >= 1ms (NFR 위반)",
        per_call
    );
}
