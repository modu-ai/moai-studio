//! MoAI Studio v3 — 메인 바이너리 엔트리.
//!
//! Phase 1.1 (SPEC-V3-001 RG-V3-2): GPUI 윈도우 오픈.
//!
//! ## Persistence lifecycle hooks (T13, REQ-P-050~056)
//!
//! - `restore_panes_on_startup()`: 앱 초기화 시 pane 레이아웃 복원 (현재 stub, 미래 GPUI 통합 시 활성화)
//! - `save_panes_on_shutdown()`: 앱 종료 시 pane 레이아웃 저장 (현재 stub, 미래 WindowCloseEvent 핸들러에서 호출)
//!
//! 빌드:
//! ```bash
//! cargo run --bin moai-studio             # GPUI 윈도우 (기본)
//! cargo run --bin moai-studio -- --scaffold  # 스캐폴드 로그만 (GPUI 없이)
//! ```

use tracing::info;

// ============================================================
// @MX:ANCHOR: [AUTO] restore-on-startup
// @MX:REASON: session 시작 시 사용자 layout 복원 진입점.
//             fan_in 향후: main + persistence 모듈 + 통합 테스트 (≥ 3).
//             이 함수 시그니처 변경 시 모든 호출자 동시 업데이트 필요.
// ============================================================

/// 앱 시작 시 pane 레이아웃을 복원한다.
///
/// # @MX:TODO: [AUTO] GPUI WindowCloseEvent 연동 미구현
/// GPUI render layer (AC-P-4, render-layer iteration) 도입 이후
/// `WindowCloseEvent` 핸들러에서 `save_panes_on_shutdown()` 을 호출하고,
/// 이 함수는 `gpui::AppContext` 에서 `TabContainer` 를 직접 복원해야 한다.
/// 현재는 no-op stub — persistence 모듈 연결 지점만 확보.
///
/// 연관: `moai_studio_workspace::panes_convert::layout_v1_to_tab_inputs`
#[cfg(not(test))]
fn restore_panes_on_startup(workspace_id: &str) {
    use moai_studio_workspace::panes_convert::snapshot_path;
    use moai_studio_workspace::persistence::load_panes;

    let path = snapshot_path(workspace_id);
    match load_panes(&path) {
        Ok(layout) => {
            info!(
                "restore_panes_on_startup: {} 탭 복원 준비 완료 (GPUI 통합 대기)",
                layout.tabs.len()
            );
            // TODO(T14-render): layout_v1_to_tab_inputs(&layout) → TabContainer::restore(inputs)
        }
        Err(e) => {
            // 파일 없음 = 첫 실행 또는 저장 전 — 정상 케이스
            tracing::debug!("restore_panes_on_startup: {} (첫 실행 또는 파일 없음)", e);
        }
    }
}

/// 앱 종료 시 pane 레이아웃을 저장한다.
///
/// # @MX:TODO: [AUTO] GPUI WindowCloseEvent 연동 미구현
/// 현재 no-op stub. GPUI `WindowCloseEvent` 핸들러에서 `TabContainer` 의
/// 현재 상태를 `TabSnapshotInput` 으로 변환한 후 이 함수에 전달해야 한다.
///
/// 연관: `moai_studio_workspace::panes_convert::tab_container_to_layout_v1`
///       `moai_studio_workspace::persistence::save_panes`
#[cfg(not(test))]
fn save_panes_on_shutdown(workspace_id: &str) {
    use moai_studio_workspace::panes_convert::snapshot_path;

    let path = snapshot_path(workspace_id);
    tracing::debug!(
        "save_panes_on_shutdown: {} (GPUI WindowCloseEvent 핸들러 연동 대기, stub)",
        path.display()
    );
    // TODO(T14-render): tab_container_to_layout_v1(&current_tabs) → save_panes(&path, &layout)
}

fn main() {
    // 로깅 초기화
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!(
        "MoAI Studio v3 — Build {} / Target: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );

    // CLI 인자 처리 — `--scaffold` 시 GPUI 없이 스캐폴드 로그만 출력 (CI smoke 용)
    let args: Vec<String> = std::env::args().collect();
    let scaffold_only = args.iter().any(|a| a == "--scaffold");

    // 플러그인 초기화 (번들 플러그인은 컴파일 타임 feature flag)
    moai_studio_ui::hello();
    moai_studio_terminal::hello();
    moai_studio_workspace::hello();
    moai_studio_plugin_api::hello();

    #[cfg(feature = "moai-adk")]
    {
        info!("moai-adk plugin: bundled (feature=moai-adk)");
        moai_studio_plugin_moai_adk::hello();
    }

    #[cfg(not(feature = "moai-adk"))]
    info!("moai-adk plugin: disabled (feature=no-moai-adk)");

    if scaffold_only {
        info!("Scaffold OK — GPUI 윈도우 건너뜀 (--scaffold 모드)");
        return;
    }

    // Phase T13: pane 레이아웃 복원 stub (GPUI render layer 통합 전 연결 지점)
    // @MX:TODO: [AUTO] GPUI WindowCloseEvent 연동 전까지 workspace_id 를 실제 선택된 workspace 로 교체 필요.
    #[cfg(not(test))]
    restore_panes_on_startup("default");

    // Phase 1.7: 저장된 workspace 리스트 로드 + storage_path 전달 (버튼 클릭 시 재로드용).
    let (workspaces, storage_path) = match moai_studio_workspace::WorkspacesStore::load_default() {
        Ok(store) => {
            info!(
                "workspaces loaded: {} items from {}",
                store.list().len(),
                store.path().display()
            );
            (store.list().to_vec(), store.path().to_path_buf())
        }
        Err(e) => {
            tracing::warn!("workspace store load 실패, 빈 리스트로 fallback: {e}");
            let fallback = moai_studio_workspace::default_storage_path()
                .unwrap_or_else(|_| std::path::PathBuf::from("workspaces.json"));
            (Vec::new(), fallback)
        }
    };

    // GPUI 윈도우 오픈 (blocks until app 종료)
    moai_studio_ui::run_app(workspaces, storage_path);

    // Phase T13: 앱 종료 시 pane 레이아웃 저장 stub
    #[cfg(not(test))]
    save_panes_on_shutdown("default");
}
