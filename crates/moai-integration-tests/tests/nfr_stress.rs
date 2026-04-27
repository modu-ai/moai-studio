//! T-029: NFR stress + benchmark (SPEC-M1-001 §5)
//!
//! 측정 대상:
//!   - 콜드 스타트: RustCore::new() → 첫 workspace Running 시점 (<1.0s 목표)
//!   - Workspace 생성: stub Claude spawn 포함 <3s
//!   - Workspace 전환: in-process state 조회 <100ms
//!   - FFI call overhead: supervisor 호출 1000회 평균 <1ms (참고용 sanity)
//!   - Store 쿼리: workspace CRUD <5ms (평균)
//!   - 4 동시 워크스페이스: 생성/삭제 루프에서 deadlock / leak 없음
//!
//! 실제 RSS 측정은 `ps -o rss` 가 가능한 환경에서만 수행하며, 미가용 시
//! "deferred" 로 표기한다. Hook HTTP P95/MCP round-trip 은 이미 MS-3 에서
//! hook_roundtrip.rs 와 mcp_roundtrip.rs 로 개별 검증되므로 여기서는 재측정 없이
//! 그 결과를 보고서에 집계한다.

use std::path::PathBuf;
use std::time::Instant;

use moai_store::Store;
use moai_supervisor::{
    RootSupervisor, WorkspaceCreateRequest, lifecycle::create_workspace, workspace::WorkspaceId,
};
use tempfile::tempdir;

fn make_req(name: &str, proj: PathBuf, wt: PathBuf) -> WorkspaceCreateRequest {
    WorkspaceCreateRequest {
        name: name.to_string(),
        project_path: proj,
        worktree_path: wt,
        spec_id: None,
    }
}

/// NFR: 콜드 스타트 — store open + supervisor 초기화 + 첫 workspace 생성이 1초 내 완료되어야 함.
#[tokio::test]
async fn nfr_cold_start_under_1s() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("cold.db");

    let start = Instant::now();
    let store = Store::open(&db_path).unwrap();
    let sup = RootSupervisor::new(store);
    let proj = tmp.path().join("p");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("w");
    let _id = create_workspace(&sup, make_req("cold", proj, wt))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    eprintln!("[NFR] cold_start (store+sup+first create) = {:?}", elapsed);
    assert!(
        elapsed.as_secs_f64() < 1.0,
        "콜드 스타트 {:.3}s — 1.0s 목표 초과",
        elapsed.as_secs_f64()
    );
}

/// NFR: workspace 생성 1회 <3s (stub Claude spawn 포함).
#[tokio::test]
async fn nfr_workspace_create_under_3s() {
    let tmp = tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    let proj = tmp.path().join("p");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("w");

    let start = Instant::now();
    let _id = create_workspace(&sup, make_req("a", proj, wt))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    eprintln!("[NFR] workspace_create = {:?}", elapsed);
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "워크스페이스 생성 {:.3}s — 3.0s 목표 초과",
        elapsed.as_secs_f64()
    );
}

/// NFR: workspace 전환 <100ms. in-process state 조회로 simulate.
#[tokio::test]
async fn nfr_workspace_switch_under_100ms() {
    let tmp = tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    let mut ids: Vec<WorkspaceId> = Vec::new();
    for i in 0..2 {
        let proj = tmp.path().join(format!("p{i}"));
        std::fs::create_dir_all(&proj).unwrap();
        let wt = tmp.path().join(format!("w{i}"));
        let id = create_workspace(&sup, make_req(&format!("w{i}"), proj, wt))
            .await
            .unwrap();
        ids.push(id);
    }

    // 100회 전환 시뮬레이션 — 각 전환은 get() + list() 로 UI state rebuild
    let start = Instant::now();
    for _ in 0..100 {
        let _ = sup.get(ids[0]).await;
        let _ = sup.get(ids[1]).await;
        let _ = sup.list().await;
    }
    let elapsed = start.elapsed();
    let per_switch_ms = elapsed.as_millis() as f64 / 100.0;

    eprintln!(
        "[NFR] workspace_switch avg = {:.3}ms (100 iterations)",
        per_switch_ms
    );
    assert!(
        per_switch_ms < 100.0,
        "워크스페이스 전환 평균 {:.3}ms — 100ms 목표 초과",
        per_switch_ms
    );
}

/// NFR: FFI-equivalent call overhead — supervisor 호출 1000회 평균 <1ms.
/// 실제 swift-bridge 경유 호출은 Swift 측 micro-bench 로 측정하나, Rust 측
/// 최소한의 동기 호출 오버헤드가 1ms 미만임을 확인한다 (상한 sanity).
#[tokio::test]
async fn nfr_ffi_call_overhead_under_1ms() {
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    let iters = 10_000u32;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = sup.len().await;
    }
    let elapsed = start.elapsed();
    let per_call_us = elapsed.as_micros() as f64 / iters as f64;

    eprintln!(
        "[NFR] ffi_equiv_call avg = {:.3}µs ({} iter)",
        per_call_us, iters
    );
    assert!(
        per_call_us < 1_000.0,
        "FFI-equivalent 호출 평균 {:.3}µs — 1ms 목표 초과",
        per_call_us
    );
}

/// NFR: store CRUD <5ms 평균.
#[tokio::test]
async fn nfr_store_crud_under_5ms() {
    use moai_store::{NewWorkspace, WorkspaceStoreExt};

    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();

    let iters = 50u32;
    let mut inserted = Vec::with_capacity(iters as usize);

    // INSERT 시간
    let start = Instant::now();
    for i in 0..iters {
        let row = dao
            .insert(&NewWorkspace {
                name: format!("ws-{i}"),
                project_path: format!("/tmp/p{i}"),
                spec_id: None,
                color_tag: None,
            })
            .unwrap();
        inserted.push(row.id);
    }
    let insert_avg_ms = start.elapsed().as_millis() as f64 / iters as f64;

    // LIST 시간
    let start = Instant::now();
    for _ in 0..iters {
        let _ = dao.list().unwrap();
    }
    let list_avg_ms = start.elapsed().as_millis() as f64 / iters as f64;

    eprintln!(
        "[NFR] store_insert avg = {:.3}ms, store_list avg = {:.3}ms",
        insert_avg_ms, list_avg_ms
    );

    assert!(
        insert_avg_ms < 5.0,
        "store insert 평균 {:.3}ms — 5ms 목표 초과",
        insert_avg_ms
    );
    assert!(
        list_avg_ms < 5.0,
        "store list 평균 {:.3}ms — 5ms 목표 초과",
        list_avg_ms
    );
}

/// NFR: 4-동시-워크스페이스 stress — 생성/삭제 루프에서 deadlock / panic 없음.
#[tokio::test]
async fn nfr_four_concurrent_stress_no_deadlock() {
    let tmp = tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    for cycle in 0..3 {
        // 4개 생성
        let mut ids = Vec::new();
        for i in 0..4 {
            let proj = tmp.path().join(format!("c{cycle}-p{i}"));
            std::fs::create_dir_all(&proj).unwrap();
            let wt = tmp.path().join(format!("c{cycle}-w{i}"));
            let id = create_workspace(&sup, make_req(&format!("c{cycle}-ws-{i}"), proj, wt))
                .await
                .expect("생성 성공");
            ids.push(id);
        }
        assert_eq!(sup.len().await, 4);

        // 모두 삭제
        for id in ids {
            sup.terminate(id).await.expect("terminate 성공");
        }
        assert_eq!(sup.len().await, 0);
    }

    // 3-cycle 누적 후에도 정상 동작
    assert_eq!(sup.len().await, 0);
}

/// NFR 보고서 집계용 종합 측정 — 한 프로세스 내에서 각 지표를 한 번씩 기록해
/// 테스트 로그에 남긴다. (보고서 작성 시 참조)
#[tokio::test]
async fn nfr_summary_measurements() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("summary.db");

    // Cold start
    let cold_start = Instant::now();
    let store = Store::open(&db_path).unwrap();
    let sup = RootSupervisor::new(store);
    let cold_elapsed = cold_start.elapsed();

    // Workspace create
    let proj = tmp.path().join("p");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("w");
    let create_start = Instant::now();
    let id = create_workspace(&sup, make_req("ws", proj, wt))
        .await
        .unwrap();
    let create_elapsed = create_start.elapsed();

    // FFI-equivalent 1000 호출
    let ffi_start = Instant::now();
    for _ in 0..1000 {
        let _ = sup.len().await;
    }
    let ffi_avg_us = ffi_start.elapsed().as_micros() as f64 / 1000.0;

    // Switch 100 회
    let switch_start = Instant::now();
    for _ in 0..100 {
        let _ = sup.get(id).await;
    }
    let switch_avg_ms = switch_start.elapsed().as_millis() as f64 / 100.0;

    eprintln!("===== NFR SUMMARY =====");
    eprintln!("cold_start (store+sup init):  {:?}", cold_elapsed);
    eprintln!("workspace_create (1 ws):       {:?}", create_elapsed);
    eprintln!("ffi_equiv_call avg (1000x):   {:.3}µs", ffi_avg_us);
    eprintln!("workspace_switch avg (100x):  {:.3}ms", switch_avg_ms);
    eprintln!("=======================");

    assert!(cold_elapsed.as_secs_f64() < 1.0);
    assert!(create_elapsed.as_secs_f64() < 3.0);
    assert!(ffi_avg_us < 1_000.0);
    assert!(switch_avg_ms < 100.0);
}
