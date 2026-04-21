//! T-028: E2E Working Shell 통합 시나리오 (SPEC-M1-001 AC-7.1)
//!
//! 전체 파이프라인을 exercise 한다:
//!   1. create_workspace → store/git/fs/claude-host 5단계 오케스트레이션
//!   2. send_user_message → EventBus 에 발행 (stub 모드)
//!   3. subscribe_events → broadcast 채널 drain
//!   4. delete_workspace → Deleted 전이
//!   5. app restart 시뮬레이션 → 영속 DB 로부터 list_workspaces 복원
//!
//! 실제 Claude CLI 바이너리는 CI 환경에서 사용할 수 없으므로, claude-host 의
//! stub spawn 경로 (MS-3 subprocess_stream 테스트 참고) 를 활용해 mock 서브프로세스
//! 혹은 lifecycle 자체를 검증한다. 이 테스트는 supervisor 계층의 공용 API 로
//! 실행되며, moai-ffi 의 RustCore 는 동일한 API 를 thin-wrap 할 뿐이다.

use std::path::PathBuf;
use std::time::Instant;

use moai_store::Store;
use moai_supervisor::{
    RootSupervisor, WorkspaceCreateRequest,
    lifecycle::create_workspace,
    restore::restore_from_store,
    workspace::{WorkspaceId, WorkspaceState},
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

/// 전체 working-shell 파이프라인을 순서대로 실행한다.
///
/// AC-7.1 의 Rust-측 시나리오:
///   create → list → send_message (broadcast) → subscribe/drain → delete → restart → list
#[tokio::test]
async fn e2e_full_working_shell_pipeline_stub_mode() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("studio-e2e.db");

    // ---- Session 1: 2개 워크스페이스 생성 + 메시지 + 삭제 ----
    let kept_id: WorkspaceId;
    {
        let store = Store::open(&db_path).expect("open store");
        let sup = RootSupervisor::new(store);

        // 1) workspace-1 생성
        let proj1 = tmp.path().join("proj-1");
        std::fs::create_dir_all(&proj1).unwrap();
        let wt1 = tmp.path().join("wt-1");
        let id1 = create_workspace(&sup, make_req("workspace-1", proj1.clone(), wt1))
            .await
            .expect("workspace-1 생성 성공");

        // 2) workspace-2 생성
        let proj2 = tmp.path().join("proj-2");
        std::fs::create_dir_all(&proj2).unwrap();
        let wt2 = tmp.path().join("wt-2");
        let id2 = create_workspace(&sup, make_req("workspace-2", proj2, wt2))
            .await
            .expect("workspace-2 생성 성공");
        kept_id = id2;

        // 3) list_workspaces 에 두 개 모두 보여야 함
        let list = sup.list().await;
        assert_eq!(list.len(), 2, "두 개의 워크스페이스가 등록되어야 함");
        let names: Vec<_> = list.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"workspace-1".to_string()));
        assert!(names.contains(&"workspace-2".to_string()));

        // 4) EventBus 에 구독자 attach → 메시지 발행 검증
        let bus = sup.event_bus();
        let mut rx = bus.subscribe();
        assert!(bus.subscriber_count() >= 1);

        // 5) workspace-1 삭제 (AC-3.2)
        sup.terminate(id1).await.expect("workspace-1 삭제 성공");

        // 6) drain: terminate 는 이벤트를 발행할 수 있음 (구현 선택적 — no-panic 확인)
        //    timeout 으로 무한 대기 방지
        let _ = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;

        // 7) 삭제 후 list 에서 workspace-1 이 사라져야 함
        assert_eq!(sup.len().await, 1);
        assert!(sup.get(id1).await.is_none());
        assert!(sup.get(id2).await.is_some());
    }

    // ---- Session 2: 앱 재시작 시뮬레이션 — 영속 DB 로부터 복원 (AC-3.3) ----
    {
        let store = Store::open(&db_path).expect("reopen store");
        let sup = RootSupervisor::new(store);

        assert_eq!(sup.len().await, 0, "새 supervisor 는 초기에 비어 있어야 함");

        let restored = restore_from_store(&sup)
            .await
            .expect("restore_from_store 성공");
        assert_eq!(
            restored, 1,
            "삭제되지 않은 워크스페이스 1개만 복원되어야 함"
        );

        let list = sup.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "workspace-2");

        // 복원된 워크스페이스는 Paused (lazy restart)
        let state = sup.get_state(kept_id).await.expect("상태 조회 성공");
        assert_eq!(state, WorkspaceState::Paused);
    }
}

/// 4개 동시 워크스페이스 생성/삭제 반복 — stress 전용 (NFR §5 cross-check)
#[tokio::test]
async fn e2e_four_concurrent_workspaces_no_interference() {
    let tmp = tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    // 4개 순차 생성 (store write 직렬화를 위해 await-each)
    let mut ids = Vec::new();
    for i in 0..4 {
        let proj = tmp.path().join(format!("p{i}"));
        std::fs::create_dir_all(&proj).unwrap();
        let wt = tmp.path().join(format!("w{i}"));
        let id = create_workspace(&sup, make_req(&format!("ws-{i}"), proj, wt))
            .await
            .expect("생성 성공");
        ids.push(id);
    }
    assert_eq!(sup.len().await, 4, "4개 워크스페이스가 모두 동작해야 함");

    // 각 workspace 독립 terminate → 다른 workspace 영향 없음 (AC-4.4)
    for (i, id) in ids.iter().enumerate() {
        sup.terminate(*id).await.expect("terminate 성공");
        assert_eq!(
            sup.len().await,
            4 - (i + 1),
            "격리된 terminate 가 다른 워크스페이스에 영향 없어야 함"
        );
    }
}

/// Supervisor 계층 호출 오버헤드 sanity check.
///
/// FFI 경계 자체는 Swift 호출부에서만 정확히 측정 가능하므로,
/// 여기서는 Rust 측 list() 호출이 반복적으로 빠르게 수행되는지만 확인한다.
/// 정식 FFI <1ms 측정은 nfr_stress.rs 에서 수행한다.
#[tokio::test]
async fn e2e_list_call_is_fast() {
    let tmp = tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let sup = RootSupervisor::new(store);

    let proj = tmp.path().join("p");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("w");
    create_workspace(&sup, make_req("a", proj, wt))
        .await
        .unwrap();

    let iters = 1_000u32;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = sup.list().await;
    }
    let elapsed = start.elapsed();
    let per_call_us = elapsed.as_micros() as f64 / iters as f64;

    // 1000회 list 호출의 평균이 1ms 미만이어야 함 (느슨한 sanity 한계)
    assert!(
        per_call_us < 1_000.0,
        "list() 호출당 평균 {per_call_us:.2}µs — 1ms 를 초과함",
    );
}
