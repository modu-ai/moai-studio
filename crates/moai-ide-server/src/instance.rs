//! T-015: workspace별 MCP 서버 인스턴스 관리.
//!
//! 워크스페이스마다 별도의 MCP 서버를 OS-assigned 포트(0-binding)에
//! 띄우고, 각 인스턴스는 독립적으로 종료될 수 있다.
//!
//! @MX:ANCHOR [AUTO] workspace별 MCP 인스턴스 스폰 진입점
//!   fan_in>=2 (supervisor/lifecycle + integration-tests)
//! @MX:NOTE [AUTO] 포트 할당은 OS에게 위임 (port=0). 병렬 스폰 간 충돌 없음.

use crate::server::{ServerHandle, start_server};

/// 워크스페이스별 MCP 서버 인스턴스 핸들.
///
/// `port`로 Claude `--mcp-config`에 주입할 URL을 구성하고,
/// `shutdown()` 으로 인스턴스를 즉시 종료할 수 있다.
pub struct WorkspaceInstance {
    /// 워크스페이스 ID (외부 상관관계 용도)
    pub workspace_id: u64,
    handle: ServerHandle,
}

impl WorkspaceInstance {
    /// 새 워크스페이스용 MCP 인스턴스를 스폰한다.
    pub async fn spawn(workspace_id: u64) -> anyhow::Result<Self> {
        let handle = start_server().await?;
        Ok(Self {
            workspace_id,
            handle,
        })
    }

    /// OS가 할당한 실제 바인딩 포트.
    pub fn port(&self) -> u16 {
        self.handle.port
    }

    /// MCP HTTP endpoint URL (`/mcp` 베이스 경로).
    pub fn mcp_url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.handle.port)
    }

    /// 인스턴스를 즉시 종료한다.
    pub fn shutdown(self) {
        self.handle.cancellation_token.cancel();
    }
}
