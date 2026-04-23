//! PtyEvent enum — PTY worker 에서 GPUI main thread 로 전달되는 이벤트.
//!
//! SPEC-V3-002 RG-V3-002-3: unbounded_channel 기반 (backpressure drop 금지).

/// PTY worker 가 방출하는 이벤트.
///
/// tokio::sync::mpsc::unbounded_channel 로 전달된다.
/// 채널 drop 은 PTY 종료를 의미하므로 bounded channel 은 사용하지 않는다.
#[derive(Debug)]
pub enum PtyEvent {
    /// PTY stdout 출력 (VT parser 에 주입할 raw bytes)
    Output(Vec<u8>),
    /// shell 프로세스 종료 코드
    ProcessExit(i32),
    /// 터미널 크기 변경 요청 (worker → PTY 전파)
    Resize { rows: u16, cols: u16 },
}
