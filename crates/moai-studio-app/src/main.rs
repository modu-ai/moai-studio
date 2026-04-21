//! MoAI Studio v3 — 메인 바이너리 엔트리.
//!
//! Phase 0.4 스캐폴드 단계. GPUI 통합은 Phase 1 (SPEC-V3-001 RG-V3-2) 에서 수행.
//!
//! 빌드:
//! ```bash
//! cargo run --bin moai-studio
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

    info!("MoAI Studio v3 scaffold starting…");
    info!(
        "Build: {} / Rust edition 2024 / Target: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );

    // Phase 0: 스캐폴드 — 각 내부 crate 의 hello() 로 연결 확인
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

    info!("Scaffold OK. Phase 1 (GPUI integration) pending — see SPEC-V3-001.");
}
