//! MoAI Studio UI 컴포넌트 라이브러리.
//!
//! Phase 1 (SPEC-V3-001 RG-V3-2) — GPUI 기반 윈도우 + 4 영역 레이아웃.
//!
//! ## 설계
//! - `run_app()` 이 유일한 엔트리. `moai-studio-app` 바이너리가 호출.
//! - 윈도우 크기 1600×1000 (`system.md` §8 기본 크기)
//! - 디자인 토큰 (`system.md` §4 색상) 직접 인라인 (GPUI `rgb(0x...)`)
//! - Phase 1.2 에서 4 영역 (TitleBar 44pt / Sidebar 260pt / Body / StatusBar 28pt) 확장

use gpui::{
    App, Application, Context, IntoElement, ParentElement, Render, Styled, Window, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use tracing::info;

/// Design tokens — `system.md` §4 dark primary.
pub mod tokens {
    pub const BG_BASE: u32 = 0x0a0a0b;
    pub const BG_SURFACE: u32 = 0x131315;
    pub const BG_SURFACE_2: u32 = 0x1b1b1e;
    pub const FG_PRIMARY: u32 = 0xf4f4f5;
    pub const FG_SECONDARY: u32 = 0xb5b5bb;
    pub const FG_MUTED: u32 = 0x6b6b73;
    pub const BORDER_SUBTLE: u32 = 0x2a2a2e;
    pub const ACCENT_MOAI: u32 = 0xff6a3d;
}

/// Root view — Phase 1.1 "Hello World". Phase 1.2 에서 4 영역 레이아웃으로 확장.
pub struct RootView;

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tokens::BG_BASE))
            .justify_center()
            .items_center()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .text_color(rgb(tokens::FG_PRIMARY))
                    .child("MoAI Studio"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child("v3 Phase 1.1 — GPUI scaffold"),
            )
            .child(
                div()
                    .mt_8()
                    .px_6()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(tokens::ACCENT_MOAI))
                    .text_color(rgb(0xffffff))
                    .child("Create First Workspace"),
            )
    }
}

/// 엔트리 — GPUI Application 생성 + 윈도우 오픈.
pub fn run_app() {
    info!("moai-studio-ui: GPUI Application 시작 (Phase 1.1)");

    Application::new().run(|cx: &mut App| {
        let bounds = gpui::Bounds::centered(None, size(px(1600.), px(1000.)), cx);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("MoAI Studio".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| cx.new(|_cx| RootView))
            .expect("GPUI 윈도우 생성 실패");

        cx.activate(true);
        info!("moai-studio-ui: RootView 렌더 등록 완료");
    });
}

/// 스캐폴드 hello 유지 (scaffold 단계 바이너리에서 GPUI 없는 경로로 호출 가능)
pub fn hello() {
    info!("moai-studio-ui: scaffold entry. GPUI 엔트리는 run_app()");
}
