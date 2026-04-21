//! T-016: workspace별 endpoint → HTTP POST → EventBus 발행 검증.
//!
//! NFR AC-22: hook POST P95 latency < 10ms (EventBus 발행까지).

use moai_hook_http::HookEndpoint;
use tokio::sync::broadcast;

async fn post_hook(
    base: &str,
    token: &str,
    event: &str,
    body: serde_json::Value,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{base}/hooks/{event}"))
        .header("X-Auth-Token", token)
        .json(&body)
        .send()
        .await
        .expect("POST 실패")
}

fn sample_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "sess-1",
        "cwd": "/tmp",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    })
}

#[tokio::test]
async fn endpoint_rejects_invalid_token() {
    let (tx, _rx) = broadcast::channel(8);
    let ep = HookEndpoint::spawn(1, tx).await.expect("spawn");
    let resp = post_hook(&ep.base_url(), "wrong-token", "PreToolUse", sample_body()).await;
    assert_eq!(resp.status(), 401, "잘못된 토큰은 401");
}

#[tokio::test]
async fn endpoint_publishes_to_event_bus_on_valid_hook() {
    let (tx, mut rx) = broadcast::channel(8);
    let ep = HookEndpoint::spawn(99, tx).await.expect("spawn");
    let resp = post_hook(&ep.base_url(), &ep.auth_token, "PreToolUse", sample_body()).await;
    assert_eq!(resp.status(), 200);

    // PreToolUse 응답은 permissionDecision=allow 포함
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        v["hookSpecificOutput"]["permissionDecision"], "allow",
        "PreToolUse 응답 누락: {v}"
    );

    // EventBus 수신
    let payload = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("recv timeout")
        .expect("recv fail");
    let p: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(p["kind"], "hook");
    assert_eq!(p["workspace_id"], 99);
    assert_eq!(p["event"], "PreToolUse");
    assert_eq!(p["session_id"], "sess-1");
}

#[tokio::test]
async fn unique_tokens_per_workspace() {
    let (tx, _) = broadcast::channel(8);
    let a = HookEndpoint::spawn(1, tx.clone()).await.unwrap();
    let b = HookEndpoint::spawn(2, tx).await.unwrap();
    assert_ne!(a.auth_token, b.auth_token);
    assert_eq!(a.auth_token.len(), 64, "hex 64자 토큰");
}

/// NFR AC-22: 100회 POST 반복하여 P95 < 10ms.
/// CI 불안정성을 감안해 `#[ignore]` 처리.
#[tokio::test]
#[ignore = "NFR benchmark — run manually with `cargo test -- --ignored`"]
async fn hook_post_p95_under_10ms() {
    let (tx, mut rx) = broadcast::channel(256);
    let ep = HookEndpoint::spawn(0, tx).await.unwrap();
    let client = reqwest::Client::new();

    const N: usize = 100;
    let mut samples = Vec::with_capacity(N);
    for _ in 0..N {
        let t = std::time::Instant::now();
        let resp = client
            .post(format!("{}/hooks/PreToolUse", ep.base_url()))
            .header("X-Auth-Token", &ep.auth_token)
            .json(&sample_body())
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        samples.push(t.elapsed());
    }
    samples.sort();
    let p95 = samples[(N * 95) / 100];
    println!("hook POST P95 = {p95:?} (N={N})");

    // EventBus에 모두 도착했는지 확인
    let mut got = 0;
    while rx.try_recv().is_ok() {
        got += 1;
    }
    assert_eq!(got, N);

    assert!(
        p95 < std::time::Duration::from_millis(10),
        "NFR AC-22 P95 < 10ms 위반: {p95:?}"
    );
}
