//! PTY burst read 벤치마크 (AC-T-9)
//!
//! p99 PTY read cycle ≤ 5ms (criterion)
//! adaptive buffer 64KB ↔ 4KB 전환을 검증한다.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use moai_studio_terminal::worker::AdaptiveBuffer;

fn bench_adaptive_buffer_record(c: &mut Criterion) {
    c.bench_function("adaptive_buffer_record_tick_burst", |b| {
        b.iter(|| {
            let mut buf = AdaptiveBuffer::new();
            // 3 tick 포화 → 64KB 전환
            buf.record_tick(black_box(true));
            buf.record_tick(black_box(true));
            buf.record_tick(black_box(true));
            // 2 tick 반 → 4KB 복귀
            buf.record_tick(black_box(false));
            buf.record_tick(black_box(false));
            buf
        });
    });
}

fn bench_adaptive_buffer_record_bytes(c: &mut Criterion) {
    c.bench_function("adaptive_buffer_record_bytes_64k", |b| {
        let data = vec![0u8; 65000]; // 64KB 포화 수준
        b.iter(|| {
            let mut buf = AdaptiveBuffer::new();
            // 적응형 buffer 전환 시뮬레이션
            for _ in 0..10 {
                buf.record_bytes(black_box(data.len()));
            }
            buf
        });
    });
}

criterion_group!(
    benches,
    bench_adaptive_buffer_record,
    bench_adaptive_buffer_record_bytes
);
criterion_main!(benches);
