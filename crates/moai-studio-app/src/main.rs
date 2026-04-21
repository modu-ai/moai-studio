//! MoAI Studio v3 — 메인 바이너리 엔트리.
//!
//! Phase 1.1 (SPEC-V3-001 RG-V3-2): GPUI 윈도우 오픈.
//!
//! 빌드:
//! ```bash
//! cargo run --bin moai-studio             # GPUI 윈도우 (기본)
//! cargo run --bin moai-studio -- --scaffold  # 스캐폴드 로그만 (GPUI 없이)
//! ```

use tracing::info;

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
}
