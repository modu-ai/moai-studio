//! 이벤트 큐: tokio 기반 비동기 production, sync polling 기반 consumption.
//!
//! Swift UI 는 `poll_event(workspace_id)` 를 고빈도로 호출한다.
//! Rust 는 `tokio::spawn` 태스크가 `push(...).await` 로 큐에 적재한다.
//! 큐 잠금은 `std::sync::Mutex` (경합 <1us) 로 FFI 경계 오버헤드를 <1ms 로 유지한다.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 이벤트 큐 공유 핸들.
#[derive(Clone)]
pub(crate) struct EventQueueHandle {
    inner: Arc<Mutex<VecDeque<String>>>,
}

impl EventQueueHandle {
    /// 이벤트를 큐 끝에 추가한다 (async — tokio task 내부에서 호출).
    pub(crate) async fn push(&self, payload: String) {
        // 실제 구현에서는 await 지점이 없지만, future API 확장을 위해 async 유지.
        let mut guard = self.inner.lock().expect("EventQueue mutex poisoned");
        guard.push_back(payload);
    }

    /// non-blocking pop. 비어있으면 None.
    pub(crate) fn try_pop(&self) -> Option<String> {
        let mut guard = self.inner.lock().expect("EventQueue mutex poisoned");
        guard.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}

/// `EventQueue` 는 factory 네임스페이스.
pub(crate) struct EventQueue;

impl EventQueue {
    pub(crate) fn new_handle() -> EventQueueHandle {
        EventQueueHandle {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn push_then_try_pop_returns_payload() {
        let h = EventQueue::new_handle();
        h.push("{\"a\":1}".into()).await;
        assert_eq!(h.try_pop().as_deref(), Some("{\"a\":1}"));
        assert_eq!(h.try_pop(), None);
    }

    #[tokio::test]
    async fn fifo_ordering() {
        let h = EventQueue::new_handle();
        h.push("one".into()).await;
        h.push("two".into()).await;
        assert_eq!(h.try_pop().as_deref(), Some("one"));
        assert_eq!(h.try_pop().as_deref(), Some("two"));
    }

    #[tokio::test]
    async fn multiple_handles_share_queue() {
        let h1 = EventQueue::new_handle();
        let h2 = h1.clone();
        h1.push("shared".into()).await;
        assert_eq!(h2.len(), 1);
        assert_eq!(h2.try_pop().as_deref(), Some("shared"));
        assert_eq!(h1.len(), 0);
    }
}
