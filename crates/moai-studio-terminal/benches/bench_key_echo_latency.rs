//! key-echo latency 벤치마크 (AC-T-4)
//!
//! p99 key-echo latency ≤ 16ms (60fps budget, criterion)

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use moai_studio_terminal::pty::{MockPty, Pty};

fn bench_key_echo_roundtrip(c: &mut Criterion) {
    c.bench_function("key_echo_mock_feed_read", |b| {
        b.iter(|| {
            // MockPty 로 feed → read_available 라운드트립 측정
            let mut pty = MockPty::new(vec![b"hello\n".to_vec()]);
            let key = black_box(b"echo hello\n");
            pty.feed(key).unwrap();

            pty.read_available().unwrap()
        });
    });
}

criterion_group!(benches, bench_key_echo_roundtrip);
criterion_main!(benches);
