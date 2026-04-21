#![allow(clippy::unnecessary_cast)] // swift-bridge 0.1 매크로가 생성하는 동일 타입 포인터 캐스트 허용

//! moai-ffi: Rust ↔ Swift FFI 경계
//!
//! M1 부터는 수동 C ABI 대신 `swift-bridge` 매크로로 Swift 바인딩을 자동 생성한다.
//! 모든 FFI 경계는 아래 `#[swift_bridge::bridge] mod ffi` 블록을 통해서만 노출된다.
//!
//! ## 비동기/콜백 규약
//!
//! swift-bridge 의 `#[swift_bridge(async)]` 는 Swift 6 Structured Concurrency 와
//! 조합 시 수명/스레드 안전성 이슈가 있어 본 크레이트는 사용하지 않는다.
//! 이벤트 스트림은 아래 **sync FFI + 폴링 기반 콜백** 패턴으로 구현된다.
//!
//! 1. Rust: `subscribe_events(workspace_id)` → 내부 `tokio::broadcast` 채널 구독 시작
//! 2. Rust: 이벤트 발생 시 workspace 별 VecDeque 에 저장
//! 3. Swift: `DispatchSource.timer` 로 `poll_event(workspace_id)` 를 고빈도 호출
//!    → FFI 호출 오버헤드 <1ms (micro-benchmark 로 검증)
//! 4. Swift: 수신한 JSON payload 를 `DispatchQueue.main.async` 로 UI 에 전달

// @MX:NOTE: [AUTO] FFI 표면은 반드시 이 bridge 블록을 통해서만 노출된다.

mod events;
mod filetree;
mod pane;
mod surface;
mod workspace;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

use crate::workspace::WorkspaceRegistry;

/// 프로세스 전역 tokio 런타임. FFI 호출은 sync 이지만 내부 비동기 작업은
/// 이 런타임 위에서 `Runtime::spawn` 으로 실행된다.
// @MX:ANCHOR: [AUTO] 프로세스 단일 tokio 런타임
// @MX:REASON: [AUTO] 중복 초기화 방지 + Swift 측에서 여러 RustCore 인스턴스 생성 시 런타임 공유 (fan_in>=3)
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("tokio multi-thread runtime 초기화 실패"))
}

/// Rust 코어의 불투명 핸들. Swift 에서는 `RustCore` 클래스로 노출된다.
pub struct RustCore {
    workspaces: WorkspaceRegistry,
}

impl RustCore {
    // @MX:ANCHOR: [AUTO] Swift → Rust 최초 진입점
    // @MX:REASON: [AUTO] 모든 워크스페이스/이벤트 FFI 의 단일 게이트웨이 (fan_in>=5)
    pub fn new() -> Self {
        // 런타임 초기화를 미리 트리거
        let _ = runtime();
        Self {
            workspaces: WorkspaceRegistry::new(),
        }
    }

    /// moai-core 버전 문자열 반환.
    pub fn version(&self) -> String {
        moai_core::version()
    }

    /// 새 워크스페이스를 생성하고 UUID 를 반환한다.
    pub fn create_workspace(&self, name: String, project_path: String) -> String {
        self.workspaces.create(name, project_path, runtime())
    }

    /// 지정된 워크스페이스를 삭제한다. 존재하지 않으면 `false`.
    pub fn delete_workspace(&self, workspace_id: String) -> bool {
        self.workspaces.delete(&workspace_id, runtime())
    }

    /// 현재 등록된 워크스페이스 목록을 반환한다.
    pub fn list_workspaces(&self) -> Vec<ffi::WorkspaceInfo> {
        self.workspaces.list(runtime())
    }

    /// 사용자 메시지를 지정 워크스페이스로 전달한다.
    ///
    /// M1 후속 태스크(T-013) 에서 Claude subprocess 의 stdin 전송으로 확장된다.
    /// 현재는 브로드캐스트 채널에 `user_message` 이벤트로 발행한다.
    pub fn send_user_message(&self, workspace_id: String, message: String) -> bool {
        self.workspaces
            .send_message(&workspace_id, message, runtime())
    }

    /// 이벤트 스트림 구독을 활성화한다.
    ///
    /// 이후 `poll_event(workspace_id)` 로 큐에서 하나씩 꺼내 Swift 측 콜백에
    /// 전달한다. 이미 구독 중이면 no-op.
    pub fn subscribe_events(&self, workspace_id: String) -> bool {
        self.workspaces.subscribe(&workspace_id, runtime())
    }

    /// 큐에 대기 중인 이벤트를 하나 꺼낸다. 비어있으면 `None`.
    // @MX:NOTE: [AUTO] Swift 는 DispatchSource.timer 로 고빈도 폴링. <1ms 오버헤드 목표.
    pub fn poll_event(&self, workspace_id: String) -> Option<String> {
        self.workspaces.poll_event(&workspace_id)
    }

    /// UUID 로 워크스페이스의 DB id (i64) 를 반환한다. 없으면 0.
    ///
    /// pane/surface FFI 는 i64 id 기반이므로 UUID → DB id 변환이 필요하다.
    pub fn get_workspace_db_id(&self, workspace_uuid: &str) -> i64 {
        self.workspaces.get_db_id(workspace_uuid)
    }

    // ── Pane FFI ────────────────────────────────────────────────────────────

    /// 새 pane 을 생성하고 id 를 반환한다. 오류 시 0.
    ///
    /// `parent_id == 0` 은 루트 pane 을 의미한다.
    pub fn create_pane(&self, workspace_id: i64, parent_id: i64, split: String, ratio: f64) -> i64 {
        let store = self.workspaces.store_handle();
        pane::create_pane(&store, workspace_id, parent_id, split, ratio)
    }

    /// 워크스페이스 내 pane 목록을 반환한다.
    pub fn list_panes(&self, workspace_id: i64) -> Vec<ffi::PaneInfo> {
        let store = self.workspaces.store_handle();
        pane::list_panes(&store, workspace_id)
    }

    /// pane ratio 를 업데이트한다. 성공 시 true.
    pub fn update_pane_ratio(&self, pane_id: i64, ratio: f64) -> bool {
        let store = self.workspaces.store_handle();
        pane::update_pane_ratio(&store, pane_id, ratio)
    }

    /// pane 을 삭제한다. 삭제되면 true.
    pub fn delete_pane(&self, pane_id: i64) -> bool {
        let store = self.workspaces.store_handle();
        pane::delete_pane(&store, pane_id)
    }

    // ── Surface FFI ──────────────────────────────────────────────────────────

    /// 새 surface 를 생성하고 id 를 반환한다. 오류 시 0.
    ///
    /// `state_json == ""` 은 빈 상태 (None) 를 의미한다.
    pub fn create_surface(
        &self,
        pane_id: i64,
        kind: String,
        state_json: String,
        tab_order: i64,
    ) -> i64 {
        let store = self.workspaces.store_handle();
        surface::create_surface(&store, pane_id, kind, state_json, tab_order)
    }

    /// pane 내 surface 목록을 tab_order 오름차순으로 반환한다.
    pub fn list_surfaces(&self, pane_id: i64) -> Vec<ffi::SurfaceInfo> {
        let store = self.workspaces.store_handle();
        surface::list_surfaces(&store, pane_id)
    }

    /// surface tab_order 를 업데이트한다. 성공 시 true.
    pub fn update_surface_tab_order(&self, surface_id: i64, tab_order: i64) -> bool {
        let store = self.workspaces.store_handle();
        surface::update_surface_tab_order(&store, surface_id, tab_order)
    }

    /// surface 를 삭제한다. 삭제되면 true.
    pub fn delete_surface(&self, surface_id: i64) -> bool {
        let store = self.workspaces.store_handle();
        surface::delete_surface(&store, surface_id)
    }

    // ── JSON 반환 FFI (swift-bridge Vectorizable 한계 우회) ──────────────────
    // @MX:NOTE: [AUTO] Vec<PaneInfo>/Vec<SurfaceInfo> 는 swift-bridge 0.1 Vectorizable 미생성 문제로
    //           직접 반환 불가. JSON 문자열로 직렬화하여 Swift 측에서 Codable 로 파싱한다.
    //           C-5 (Vectorizable workaround 제거) 해소 시 이 메서드를 deprecated 처리한다.

    /// 워크스페이스 내 pane 목록을 JSON 문자열로 반환한다.
    /// Swift 측 `PaneTreeModel.load()` 에서 `JSONDecoder` 로 파싱한다.
    pub fn list_panes_json(&self, workspace_id: i64) -> String {
        let store = self.workspaces.store_handle();
        pane::list_panes_json(&store, workspace_id)
    }

    /// pane 내 surface 목록을 JSON 문자열로 반환한다.
    pub fn list_surfaces_json(&self, pane_id: i64) -> String {
        let store = self.workspaces.store_handle();
        surface::list_surfaces_json(&store, pane_id)
    }

    // ── FileTree FFI (MS-4) ──────────────────────────────────────────────────
    // @MX:NOTE: [AUTO] 폴링 기반 갱신 채택 (500ms). MS-7+ 에서 notify-push 로 업그레이드 예정.

    /// 디렉토리 바로 아래 항목을 JSON 배열로 반환한다.
    ///
    /// `subpath` 가 빈 문자열이면 `workspace_path` 루트를 리스팅한다.
    /// `.git`, `target`, `node_modules`, `.build`, `build`, `.DS_Store` 는 제외된다.
    pub fn list_directory_json(&self, workspace_path: String, subpath: String) -> String {
        filetree::list_directory_json(workspace_path, subpath)
    }

    /// 워크스페이스 루트의 git status 맵을 JSON 객체로 반환한다.
    ///
    /// `{"src/main.rs": "modified", "new_file.txt": "untracked"}` 형식.
    /// git 저장소가 아니면 `{}` 반환.
    pub fn git_status_map_json(&self, workspace_path: String) -> String {
        filetree::git_status_map_json(workspace_path)
    }
}

impl Default for RustCore {
    fn default() -> Self {
        Self::new()
    }
}

// @MX:ANCHOR: [AUTO] Swift 바인딩 자동 생성 지점
// @MX:REASON: [AUTO] swift-bridge-build 가 이 블록만을 파싱해 .swift/.h 생성 (유일 FFI 경계)
#[swift_bridge::bridge]
mod ffi {
    // Swift 로 값 전달되는 워크스페이스 정보 스냅샷 (doc 속성 불가 — swift-bridge 제약)
    #[swift_bridge(swift_repr = "struct")]
    pub struct WorkspaceInfo {
        pub id: String,
        pub name: String,
        pub status: String,
    }

    // @MX:NOTE: [AUTO] pane binary tree 노드의 스냅샷. parent_id==0 은 루트 pane 을 의미
    #[swift_bridge(swift_repr = "struct")]
    pub struct PaneInfo {
        pub id: i64,
        pub workspace_id: i64,
        pub parent_id: i64,
        pub split: String,
        pub ratio: f64,
    }

    // @MX:NOTE: [AUTO] surface(탭) 스냅샷. state_json=="" 은 빈 상태를 의미
    #[swift_bridge(swift_repr = "struct")]
    pub struct SurfaceInfo {
        pub id: i64,
        pub pane_id: i64,
        pub kind: String,
        pub state_json: String,
        pub tab_order: i64,
    }

    extern "Rust" {
        type RustCore;

        #[swift_bridge(init)]
        fn new() -> RustCore;

        fn version(&self) -> String;

        fn create_workspace(&self, name: String, project_path: String) -> String;
        fn delete_workspace(&self, workspace_id: String) -> bool;
        fn list_workspaces(&self) -> Vec<WorkspaceInfo>;
        fn get_workspace_db_id(&self, workspace_uuid: &str) -> i64;

        fn send_user_message(&self, workspace_id: String, message: String) -> bool;
        fn subscribe_events(&self, workspace_id: String) -> bool;
        fn poll_event(&self, workspace_id: String) -> Option<String>;

        fn create_pane(&self, workspace_id: i64, parent_id: i64, split: String, ratio: f64) -> i64;
        fn list_panes(&self, workspace_id: i64) -> Vec<PaneInfo>;
        fn update_pane_ratio(&self, pane_id: i64, ratio: f64) -> bool;
        fn delete_pane(&self, pane_id: i64) -> bool;

        fn create_surface(
            &self,
            pane_id: i64,
            kind: String,
            state_json: String,
            tab_order: i64,
        ) -> i64;
        fn list_surfaces(&self, pane_id: i64) -> Vec<SurfaceInfo>;
        fn update_surface_tab_order(&self, surface_id: i64, tab_order: i64) -> bool;
        fn delete_surface(&self, surface_id: i64) -> bool;

        // JSON 반환 메서드 — swift-bridge Vectorizable 한계 우회
        fn list_panes_json(&self, workspace_id: i64) -> String;
        fn list_surfaces_json(&self, pane_id: i64) -> String;

        // FileTree FFI (MS-4)
        fn list_directory_json(&self, workspace_path: String, subpath: String) -> String;
        fn git_status_map_json(&self, workspace_path: String) -> String;
    }
}
