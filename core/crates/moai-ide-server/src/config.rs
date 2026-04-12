//! MCP 설정 JSON 생성 모듈

/// Claude `--mcp-config` 에 전달할 JSON 설정을 생성한다.
///
/// # 인수
/// * `port` - MCP 서버가 바인딩된 포트
/// * `server_name` - mcpServers 맵에서 사용할 서버 이름
pub fn generate_mcp_config(port: u16, server_name: &str) -> serde_json::Value {
    serde_json::json!({
        "mcpServers": {
            server_name: {
                "type": "sse",
                "url": format!("http://127.0.0.1:{}/sse", port)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 생성된 JSON이 올바른 mcpServers 구조를 갖는지 검증한다
    #[test]
    fn test_generate_mcp_config_structure() {
        let config = generate_mcp_config(8080, "moai");

        assert!(
            config["mcpServers"].is_object(),
            "mcpServers 필드가 객체여야 한다"
        );
        assert_eq!(
            config["mcpServers"]["moai"]["type"], "sse",
            "전송 타입은 sse여야 한다"
        );
        assert_eq!(
            config["mcpServers"]["moai"]["url"],
            "http://127.0.0.1:8080/sse",
            "URL은 포트를 포함한 SSE 엔드포인트여야 한다"
        );
    }

    /// 커스텀 서버 이름과 포트가 올바르게 반영되는지 검증한다
    #[test]
    fn test_generate_mcp_config_custom_values() {
        let config = generate_mcp_config(12345, "my-server");

        assert!(
            config["mcpServers"]["my-server"].is_object(),
            "커스텀 서버 이름이 키로 사용되어야 한다"
        );
        assert_eq!(
            config["mcpServers"]["my-server"]["url"],
            "http://127.0.0.1:12345/sse"
        );
    }
}
