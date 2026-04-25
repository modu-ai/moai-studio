//! RingBuffer — 고정 용량의 원형 버퍼 (RG-AD-1, REQ-AD-004)
//!
//! capacity 초과 시 oldest event 를 자동 evict 한다.
//! SPEC-V3-010 default capacity: 1000 events (USER-DECISION-AD-A).

use std::collections::VecDeque;

// @MX:ANCHOR: [AUTO] ring-buffer-eviction
// @MX:REASON: [AUTO] capacity invariant — auto-evict 보장.
//   SPEC: SPEC-V3-010 REQ-AD-004. fan_in >= 3: StreamIngestor, SseIngestor, EventTimelineView.

/// 고정 용량 원형 버퍼.
///
/// capacity 초과 시 oldest 항목을 evict 한다 (REQ-AD-004).
/// freeze 후에는 push 를 거부한다 (REQ-AD-007).
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
    frozen: bool,
}

impl<T> RingBuffer<T> {
    /// 주어진 용량으로 RingBuffer 를 생성한다.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity 는 0 보다 커야 한다");
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
            frozen: false,
        }
    }

    /// 기본 용량(1000) 으로 RingBuffer 를 생성한다 (AD-A default).
    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }

    /// 항목을 push 한다. capacity 초과 시 oldest 를 evict.
    ///
    /// frozen 상태이면 `Err(RingBufferError::Frozen)` 을 반환한다 (REQ-AD-007).
    pub fn push(&mut self, item: T) -> Result<(), RingBufferError> {
        if self.frozen {
            return Err(RingBufferError::Frozen);
        }
        if self.buf.len() >= self.capacity {
            self.buf.pop_front(); // oldest evict
        }
        self.buf.push_back(item);
        Ok(())
    }

    /// 모든 항목을 insertion 순서대로 반복한다.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    /// 현재 저장된 항목 수.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// 버퍼가 비어 있으면 true.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// 최대 capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 버퍼를 freeze 한다. freeze 후 push 는 에러를 반환한다 (REQ-AD-007).
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// freeze 상태 여부.
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
}

/// RingBuffer 에러 종류.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum RingBufferError {
    /// freeze 된 버퍼에 push 를 시도했다 (REQ-AD-007).
    #[error("ring buffer 가 frozen 상태이다 — push 거부")]
    Frozen,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// capacity 설정이 올바른지 확인.
    #[test]
    fn capacity_is_set_correctly() {
        let rb: RingBuffer<i32> = RingBuffer::new(42);
        assert_eq!(rb.capacity(), 42);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
    }

    /// capacity 초과 시 oldest 가 evict 되어야 한다 (REQ-AD-004).
    #[test]
    fn push_evicts_oldest_when_at_capacity() {
        let mut rb = RingBuffer::new(3);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        assert_eq!(rb.len(), 3);

        // 4 번째 push → 1 이 evict 됨
        rb.push(4).unwrap();
        assert_eq!(rb.len(), 3);

        let items: Vec<i32> = rb.iter().copied().collect();
        assert_eq!(items, vec![2, 3, 4]);
    }

    /// iter 는 insertion 순서대로 반환해야 한다.
    #[test]
    fn iter_in_insertion_order() {
        let mut rb = RingBuffer::new(5);
        for i in 0..5u32 {
            rb.push(i).unwrap();
        }
        let items: Vec<u32> = rb.iter().copied().collect();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    /// len / capacity invariant: len <= capacity.
    #[test]
    fn len_never_exceeds_capacity() {
        let cap = 10;
        let mut rb = RingBuffer::new(cap);
        for i in 0..50u32 {
            rb.push(i).unwrap();
            assert!(rb.len() <= cap, "len {} > capacity {}", rb.len(), cap);
        }
        assert_eq!(rb.len(), cap);
    }

    /// freeze 후 push 는 Err(Frozen) 을 반환해야 한다 (REQ-AD-007).
    #[test]
    fn push_after_freeze_returns_error() {
        let mut rb = RingBuffer::new(5);
        rb.push(1).unwrap();
        rb.freeze();
        assert!(rb.is_frozen());

        let err = rb.push(2);
        assert_eq!(err, Err(RingBufferError::Frozen));
        // freeze 후 len 은 변하지 않는다
        assert_eq!(rb.len(), 1);
    }
}
