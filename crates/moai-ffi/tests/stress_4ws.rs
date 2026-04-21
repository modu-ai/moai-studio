//! M1 C-3: 4-워크스페이스 10분 스트레스 테스트.
//!
//! `cargo test stress_4ws -- --ignored --nocapture` 로 실행.
//! 일반 `cargo test` 에서는 `#[ignore]` 로 건너뜀.
//!
//! @MX:NOTE: [AUTO] #[ignore] — CI에서 자동 실행되지 않음. 수동 opt-in 실행 전용.

use moai_ffi::RustCore;

#[test]
#[ignore] // 실행: cargo test stress_4ws -- --ignored --nocapture
fn stress_4ws_10min() {
    // 4개 워크스페이스 생성 → 이벤트 구독 → 10분 폴링 → 데드락/패닉 없음 검증
    let core = RustCore::new();
    let mut ws_ids = vec![];
    for i in 0..4 {
        let id = core.create_workspace(format!("stress-{i}"), "/tmp".to_string());
        assert!(!id.is_empty(), "워크스페이스 {i} 생성 실패");
        core.subscribe_events(id.clone());
        ws_ids.push(id);
    }

    let end = std::time::Instant::now() + std::time::Duration::from_secs(60 * 10);
    let mut poll_count = 0u64;
    while std::time::Instant::now() < end {
        for id in &ws_ids {
            let _ = core.poll_event(id.clone());
            poll_count += 1;
        }
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps 폴링
    }

    println!("총 poll 횟수: {poll_count}");

    for id in ws_ids {
        assert!(core.delete_workspace(id), "워크스페이스 삭제 실패");
    }
    println!("PASS: 10분 스트레스 완료 — 데드락/패닉 없음");
}

#[test]
fn stress_4ws_short_smoke() {
    // CI에서 실행되는 1초 스모크: poll 루프 자체가 컴파일·동작하는지 검증
    let core = RustCore::new();
    let mut ws_ids = vec![];
    for i in 0..4 {
        let id = core.create_workspace(format!("smoke-{i}"), "/tmp".to_string());
        assert!(!id.is_empty());
        core.subscribe_events(id.clone());
        ws_ids.push(id);
    }

    let end = std::time::Instant::now() + std::time::Duration::from_millis(200);
    while std::time::Instant::now() < end {
        for id in &ws_ids {
            let _ = core.poll_event(id.clone());
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    for id in ws_ids {
        assert!(core.delete_workspace(id));
    }
}
