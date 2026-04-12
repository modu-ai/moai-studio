//! moai-core: MoAI Studio Rust 코어 퍼사드
//! Swift UI가 import하는 단일 공개 API 진입점

// @MX:ANCHOR: 공개 API 경계 — Swift FFI가 호출하는 모든 Rust 함수는 여기서 정의됨
// @MX:REASON: [AUTO] fan_in >= 3 (moai-ffi, 테스트, 향후 CLI) 예상

/// MoAI Studio 버전 반환
///
/// CARGO_PKG_VERSION 환경변수에서 빌드 타임에 결정된 버전 문자열을 반환한다.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Workspace 초기화 설정값
pub struct WorkspaceConfig {
    /// 작업 디렉토리 절대 경로
    pub working_dir: String,
    /// Anthropic API 키
    pub api_key: String,
    /// MCP 설정 파일 경로 (선택)
    pub mcp_config_path: Option<String>,
}

/// 코어 초기화 핸들
///
/// M0 단계에서는 플레이스홀더. M1+에서 tokio 런타임과 supervisor가 추가된다.
pub struct CoreHandle {
    // M0: 현재는 빈 구조체 — 향후 런타임 핸들 필드 추가 예정
}

impl CoreHandle {
    /// 코어 핸들 초기화
    pub fn new() -> Self {
        Self {}
    }

    /// 버전 문자열 반환
    pub fn version(&self) -> String {
        version()
    }
}

impl Default for CoreHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED→GREEN: moai_core::version()이 비어 있지 않은 문자열을 반환해야 함
    #[test]
    fn test_moai_core_version_is_non_empty() {
        let v = version();
        assert!(!v.is_empty(), "version() 결과가 비어 있음");
    }

    // RED→GREEN: version()이 Cargo.toml에 정의된 패키지 버전과 일치해야 함
    #[test]
    fn test_moai_core_version_matches_cargo_pkg_version() {
        let v = version();
        assert_eq!(v, env!("CARGO_PKG_VERSION"), "버전이 CARGO_PKG_VERSION과 불일치");
    }

    // RED→GREEN: CoreHandle::new()가 패닉 없이 생성되어야 함
    #[test]
    fn test_core_handle_new_does_not_panic() {
        let _handle = CoreHandle::new();
    }

    // RED→GREEN: CoreHandle::version()이 전역 version()과 동일한 값을 반환해야 함
    #[test]
    fn test_core_handle_version_matches_global_version() {
        let handle = CoreHandle::new();
        assert_eq!(handle.version(), version());
    }

    // RED→GREEN: WorkspaceConfig 기본값 생성 — api_key와 working_dir 검증
    #[test]
    fn test_workspace_config_fields_accessible() {
        let cfg = WorkspaceConfig {
            working_dir: "/tmp/test".to_string(),
            api_key: "sk-test".to_string(),
            mcp_config_path: None,
        };
        assert_eq!(cfg.working_dir, "/tmp/test");
        assert_eq!(cfg.api_key, "sk-test");
        assert!(cfg.mcp_config_path.is_none());
    }

    // RED→GREEN: WorkspaceConfig의 mcp_config_path 옵션 설정 가능 여부
    #[test]
    fn test_workspace_config_with_mcp_path() {
        let cfg = WorkspaceConfig {
            working_dir: "/tmp".to_string(),
            api_key: "key".to_string(),
            mcp_config_path: Some("/tmp/mcp.json".to_string()),
        };
        assert_eq!(cfg.mcp_config_path, Some("/tmp/mcp.json".to_string()));
    }
}
