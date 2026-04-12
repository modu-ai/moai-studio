//! 설정 생성 통합 테스트
//!
//! moai-ide-server의 MCP 설정 생성 및 인증 토큰 생성 흐름을 검증한다.
//! generate_mcp_config()와 generate_auth_token()의 크로스 크레이트 통합을 테스트한다.

use moai_ide_server::{auth::generate_auth_token, config::generate_mcp_config};

/// generate_mcp_config()가 올바른 JSON 구조를 반환하는지 검증
#[test]
fn test_generate_mcp_config_valid_json_structure() {
    let port = 8080u16;
    let server_name = "moai";

    let config = generate_mcp_config(port, server_name);

    // JSON 최상위 구조 검증
    assert!(config.is_object(), "설정은 JSON 객체여야 한다");
    assert!(
        config["mcpServers"].is_object(),
        "mcpServers 필드가 객체여야 한다"
    );
}

/// 올바른 서버 이름이 키로 사용되는지 검증
#[test]
fn test_generate_mcp_config_server_name_as_key() {
    let config = generate_mcp_config(9000, "my-mcp-server");

    assert!(
        config["mcpServers"]["my-mcp-server"].is_object(),
        "서버 이름이 mcpServers의 키로 사용되어야 한다"
    );
}

/// 생성된 URL에 올바른 포트가 포함되는지 검증
#[test]
fn test_generate_mcp_config_url_contains_correct_port() {
    let port = 12345u16;
    let config = generate_mcp_config(port, "test-server");

    let url = config["mcpServers"]["test-server"]["url"]
        .as_str()
        .expect("url 필드가 문자열이어야 한다");

    assert!(
        url.contains("12345"),
        "URL에 포트 번호 12345가 포함되어야 한다. 실제 URL: {}",
        url
    );
    assert!(
        url.starts_with("http://127.0.0.1:"),
        "URL은 http://127.0.0.1:으로 시작해야 한다. 실제 URL: {}",
        url
    );
}

/// 서버 타입 필드가 올바른지 검증
#[test]
fn test_generate_mcp_config_type_field() {
    let config = generate_mcp_config(3000, "moai");

    // 현재 구현은 "sse" 타입을 사용
    let server_type = config["mcpServers"]["moai"]["type"]
        .as_str()
        .expect("type 필드가 문자열이어야 한다");

    assert_eq!(server_type, "sse", "MCP 서버 타입은 'sse'여야 한다");
}

/// generate_auth_token()이 64자 hex 문자열을 반환하는지 검증
#[test]
fn test_generate_auth_token_is_64_hex_chars() {
    let token = generate_auth_token();

    assert_eq!(
        token.len(),
        64,
        "인증 토큰은 64자여야 한다 (32바이트 hex 인코딩)"
    );
    assert!(
        token.chars().all(|c| c.is_ascii_hexdigit()),
        "인증 토큰은 hex 문자(0-9, a-f)만 포함해야 한다. 실제 토큰: {}",
        token
    );
}

/// generate_auth_token() 연속 호출 시 서로 다른 토큰이 생성되는지 검증
#[test]
fn test_generate_auth_token_is_unique() {
    let token1 = generate_auth_token();
    let token2 = generate_auth_token();
    let token3 = generate_auth_token();

    assert_ne!(
        token1, token2,
        "연속 호출 시 서로 다른 토큰이 생성되어야 한다"
    );
    assert_ne!(
        token2, token3,
        "연속 호출 시 서로 다른 토큰이 생성되어야 한다"
    );
    assert_ne!(
        token1, token3,
        "연속 호출 시 서로 다른 토큰이 생성되어야 한다"
    );
}

/// 설정과 토큰을 함께 생성하는 전체 흐름 검증
#[test]
fn test_config_and_token_generation_full_flow() {
    // 1단계: 인증 토큰 생성
    let auth_token = generate_auth_token();
    assert_eq!(auth_token.len(), 64, "토큰은 64자여야 한다");

    // 2단계: MCP 설정 생성 (임의 포트 시뮬레이션)
    let simulated_port = 54321u16;
    let config = generate_mcp_config(simulated_port, "moai-studio");

    // 3단계: 설정 JSON이 유효하고 완전한지 검증
    let url = config["mcpServers"]["moai-studio"]["url"]
        .as_str()
        .expect("url 필드가 존재해야 한다");

    assert!(url.contains("54321"), "설정 URL에 포트가 포함되어야 한다");

    // 4단계: 설정을 JSON 문자열로 직렬화 가능한지 확인
    let config_str = serde_json::to_string(&config).expect("설정이 JSON으로 직렬화 가능해야 한다");
    assert!(
        !config_str.is_empty(),
        "직렬화된 설정이 비어있지 않아야 한다"
    );
    assert!(
        config_str.contains("mcpServers"),
        "직렬화된 설정에 mcpServers 키가 포함되어야 한다"
    );
}

/// 다양한 포트 범위에서 설정이 올바르게 생성되는지 검증
#[test]
fn test_generate_mcp_config_various_ports() {
    // 최솟값 유효 포트
    let config_min = generate_mcp_config(1024, "server");
    let url_min = config_min["mcpServers"]["server"]["url"].as_str().unwrap();
    assert!(url_min.contains("1024"));

    // 최댓값 포트
    let config_max = generate_mcp_config(65535, "server");
    let url_max = config_max["mcpServers"]["server"]["url"].as_str().unwrap();
    assert!(url_max.contains("65535"));

    // 일반적인 개발 포트
    let config_dev = generate_mcp_config(3000, "server");
    let url_dev = config_dev["mcpServers"]["server"]["url"].as_str().unwrap();
    assert!(url_dev.contains("3000"));
}
