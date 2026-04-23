//! PTY worker thread + 적응형 buffer (SPEC-V3-002 RG-V3-002-3).
//!
//! @MX:NOTE(adaptive-buffer-4k-to-64k)
//! 적응형 buffer 정책:
//!   - 기본: 4KB / 10ms tick
//!   - 3 tick 연속 포화(saturated) → 64KB 전환 (tracing::info 로그)
//!   - 2 tick 연속 반(half) 미만 → 4KB 복귀 (tracing::info 로그)
//!
//! "포화"(saturated) = 읽은 바이트 수 ≥ 현재 buffer 크기의 95%
//! "반(half) 미만" = 읽은 바이트 수 < 현재 buffer 크기의 50%

use crate::events::PtyEvent;
use crate::pty::Pty;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info};

/// PTY worker 가 사용하는 buffer 크기.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSize {
    /// 기본 4KB
    Small = 4096,
    /// 버스트 시 64KB
    Large = 65536,
}

/// 적응형 buffer 상태 머신.
///
/// @MX:NOTE(adaptive-buffer-state-machine)
/// tick 기반: 각 read_available 호출 결과를 record_tick 으로 보고.
/// consecutive_saturated: 연속 포화 횟수 (3회 → Large)
/// consecutive_half: 연속 반 미만 횟수 (2회 → Small)
pub struct AdaptiveBuffer {
    current: BufferSize,
    consecutive_saturated: u8,
    consecutive_half: u8,
}

impl AdaptiveBuffer {
    /// 기본 상태 (4KB) 로 초기화.
    pub fn new() -> Self {
        Self {
            current: BufferSize::Small,
            consecutive_saturated: 0,
            consecutive_half: 0,
        }
    }

    /// 현재 buffer 크기를 반환한다.
    pub fn current_size(&self) -> BufferSize {
        self.current
    }

    /// tick 결과를 보고하고 buffer 크기를 조정한다.
    ///
    /// - saturated=true: 읽은 바이트가 buffer 의 95% 이상
    /// - saturated=false: 읽은 바이트가 buffer 의 50% 미만 (half)
    pub fn record_tick(&mut self, saturated: bool) {
        if saturated {
            self.consecutive_saturated += 1;
            self.consecutive_half = 0;

            if self.consecutive_saturated >= 3 && self.current == BufferSize::Small {
                self.current = BufferSize::Large;
                self.consecutive_saturated = 0;
                info!(
                    buffer_size = "64KB",
                    "적응형 buffer: 3 tick 연속 포화 → 64KB 전환"
                );
            }
        } else {
            self.consecutive_half += 1;
            self.consecutive_saturated = 0;

            if self.consecutive_half >= 2 && self.current == BufferSize::Large {
                self.current = BufferSize::Small;
                self.consecutive_half = 0;
                info!(
                    buffer_size = "4KB",
                    "적응형 buffer: 2 tick 반 미만 → 4KB 복귀"
                );
            }
        }
    }

    /// 읽은 바이트 수를 기반으로 saturated/half 를 자동 판정하고 record_tick 호출.
    pub fn record_bytes(&mut self, bytes_read: usize) {
        let capacity = self.current as usize;
        let saturated = bytes_read >= (capacity * 95 / 100);
        let half = bytes_read < (capacity / 2);

        if saturated {
            self.record_tick(true);
        } else if half {
            self.record_tick(false);
        } else {
            // 중간값: 카운트 리셋 없이 유지
            debug!(bytes_read, capacity, "buffer: 중간 구간 (카운트 유지)");
        }
    }
}

impl Default for AdaptiveBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// @MX:WARN(blocking-read-in-async-context)
/// @MX:REASON(portable-pty-blocking-api): portable-pty 의 Read 구현은 blocking 이다.
///   tokio async runtime 내에서 직접 호출하면 reactor thread 를 점령한다.
///   반드시 tokio::task::block_in_place 로 격리해야 한다.
///
/// PTY worker — PTY read loop 를 실행하고 PtyEvent 를 채널로 emit.
pub struct PtyWorker {
    adaptive: AdaptiveBuffer,
}

impl PtyWorker {
    /// 새 PtyWorker 를 생성한다.
    pub fn new() -> Self {
        Self {
            adaptive: AdaptiveBuffer::new(),
        }
    }

    /// PTY read loop 를 실행한다.
    ///
    /// blocking PTY read 는 tokio::task::block_in_place 로 격리한다.
    /// PtyEvent 는 unbounded_channel 로 전송 (backpressure drop 금지).
    pub async fn run(mut self, mut pty: Box<dyn Pty>, tx: UnboundedSender<PtyEvent>) {
        loop {
            // blocking read 를 block_in_place 로 격리
            let data: std::io::Result<Vec<u8>> =
                tokio::task::block_in_place(|| pty.read_available());

            match data {
                Ok(bytes) if !bytes.is_empty() => {
                    let len = bytes.len();
                    self.adaptive.record_bytes(len);

                    // unbounded send 는 실패하지 않는다 (receiver 가 drop 된 경우에만)
                    if tx.send(PtyEvent::Output(bytes)).is_err() {
                        info!("PtyEvent channel closed — PTY worker 종료");
                        break;
                    }
                }
                Ok(_) => {
                    // 빈 read — PTY 가 살아있는지 확인
                    if !pty.is_alive() {
                        let exit_code = 0; // portable-pty 에서 exit code 추출 (추후 구현)
                        let _ = tx.send(PtyEvent::ProcessExit(exit_code));
                        break;
                    }
                    // 10ms tick
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    info!(error = %e, "PTY read 에러 — worker 종료");
                    let _ = tx.send(PtyEvent::ProcessExit(1));
                    break;
                }
            }
        }
    }
}

impl Default for PtyWorker {
    fn default() -> Self {
        Self::new()
    }
}
