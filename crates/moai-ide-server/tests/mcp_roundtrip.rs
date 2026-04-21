//! T-015: workspace별 MCP 서버 인스턴스 + full round-trip + P95 benchmark.
//!
//! NFR AC-21: tools/call echo round-trip P95 < 50ms.

use moai_ide_server::WorkspaceInstance;

async fn mcp_init(client: &reqwest::Client, url: &str) {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "t15", "version": "1.0"}
        }
    });
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .expect("initialize 실패");
    assert_eq!(resp.status(), 200, "initialize 200 expected");
}

async fn mcp_call_echo(client: &reqwest::Client, url: &str, id: u64, msg: &str) {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": {"name": "echo", "arguments": {"msg": msg}}
    });
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .expect("tools/call 실패");
    assert_eq!(resp.status(), 200, "tools/call 200 expected");
    let v: serde_json::Value = resp.json().await.expect("json 파싱");
    let text = v["result"]["content"][0]["text"].as_str().unwrap_or("");
    assert_eq!(text, msg, "echo 결과 불일치: {v:?}");
}

#[tokio::test]
async fn workspace_instance_binds_unique_port_per_workspace() {
    let a = WorkspaceInstance::spawn(1).await.expect("ws1");
    let b = WorkspaceInstance::spawn(2).await.expect("ws2");
    assert_ne!(a.port(), b.port(), "워크스페이스별 포트 분리 실패");
    assert!(a.mcp_url().starts_with("http://127.0.0.1:"));
    a.shutdown();
    b.shutdown();
}

#[tokio::test]
async fn mcp_full_roundtrip_initialize_then_echo() {
    let inst = WorkspaceInstance::spawn(42).await.expect("spawn");
    let url = inst.mcp_url();
    let client = reqwest::Client::new();
    mcp_init(&client, &url).await;
    mcp_call_echo(&client, &url, 2, "hello-moai").await;
    inst.shutdown();
}

/// NFR AC-21: 100회 반복하여 P95 round-trip latency < 50ms 검증.
/// CI에서 불안정할 수 있으므로 `#[ignore]` 처리하여 로컬 벤치마크 용도로 둔다.
#[tokio::test]
#[ignore = "NFR benchmark — run manually with `cargo test -- --ignored`"]
async fn mcp_roundtrip_p95_under_50ms() {
    let inst = WorkspaceInstance::spawn(7).await.expect("spawn");
    let url = inst.mcp_url();
    let client = reqwest::Client::new();
    mcp_init(&client, &url).await;

    const N: usize = 100;
    let mut samples = Vec::with_capacity(N);
    for i in 0..N {
        let t = std::time::Instant::now();
        mcp_call_echo(&client, &url, (i as u64) + 10, "bench").await;
        samples.push(t.elapsed());
    }
    samples.sort();
    let p95 = samples[(N * 95) / 100];
    println!("MCP echo round-trip P95 = {:?} (N={N})", p95);
    assert!(
        p95 < std::time::Duration::from_millis(50),
        "NFR AC-21 P95 < 50ms 위반: {p95:?}"
    );
    inst.shutdown();
}
