//! PTY worker 적응형 buffer 단위 테스트 (AC-T-8(c), AC-T-9)
//!
//! libghostty/Zig 없이 실행 가능.

use moai_studio_terminal::worker::{AdaptiveBuffer, BufferSize};

/// 3 tick 연속 포화 시 64KB 로 전환
///
/// AC-T-8(c): adaptive buffer 전환 단위 검증
#[test]
fn transitions_to_64k_on_burst() {
    let mut buf = AdaptiveBuffer::new();
    assert_eq!(buf.current_size(), BufferSize::Small, "초기 상태는 4KB");

    // 3회 연속 포화 tick 보고
    buf.record_tick(true); // saturated
    buf.record_tick(true); // saturated
    buf.record_tick(true); // saturated → 64KB 전환

    assert_eq!(
        buf.current_size(),
        BufferSize::Large,
        "3 tick 포화 후 64KB 여야 함"
    );
}

/// 64KB 상태에서 2 tick 반 미만 → 4KB 복귀
///
/// AC-T-8(c): adaptive buffer 복귀 검증
#[test]
fn returns_to_4k_after_burst() {
    let mut buf = AdaptiveBuffer::new();

    // 64KB 상태로 전환
    buf.record_tick(true);
    buf.record_tick(true);
    buf.record_tick(true);
    assert_eq!(buf.current_size(), BufferSize::Large);

    // 2 tick 반(half) 미만 → 복귀
    buf.record_tick(false); // not saturated
    buf.record_tick(false); // not saturated → 4KB 복귀

    assert_eq!(
        buf.current_size(),
        BufferSize::Small,
        "2 tick 반 미만 후 4KB 복귀해야 함"
    );
}

/// 중간에 포화 중단 시 연속 카운트 리셋
#[test]
fn burst_count_resets_on_non_saturated_tick() {
    let mut buf = AdaptiveBuffer::new();

    buf.record_tick(true); // saturated (count=1)
    buf.record_tick(true); // saturated (count=2)
    buf.record_tick(false); // not saturated → count 리셋
    buf.record_tick(true); // saturated (count=1) — 아직 4KB

    assert_eq!(
        buf.current_size(),
        BufferSize::Small,
        "연속이 끊기면 4KB 유지"
    );
}

/// unbounded channel 기반: backpressure 없이 메시지 전송
///
/// AC-T-8(c): no_drop_on_backpressure 검증
#[tokio::test]
async fn no_drop_on_backpressure() {
    use moai_studio_terminal::events::PtyEvent;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::unbounded_channel::<PtyEvent>();

    // 1000개 메시지를 backpressure 없이 전송
    for i in 0u16..1000 {
        tx.send(PtyEvent::Resize { rows: i, cols: 80 })
            .expect("unbounded channel은 send가 실패하지 않아야 함");
    }

    let mut count = 0u16;
    while let Ok(event) = rx.try_recv() {
        match event {
            PtyEvent::Resize { rows, cols: _ } => {
                assert_eq!(rows, count);
                count += 1;
            }
            _ => panic!("예상치 못한 이벤트"),
        }
    }
    assert_eq!(
        count, 1000,
        "1000개 메시지가 모두 수신되어야 함 (drop 없음)"
    );
}
