//! MoAI Studio UI 컴포넌트 라이브러리.
//!
//! Phase 1.2 (SPEC-V3-001 RG-V3-2) — 4 영역 레이아웃 (TitleBar/Sidebar/Body/StatusBar).
//!
//! ## 설계
//! - `run_app()` 이 유일한 엔트리. `moai-studio-app` 바이너리가 호출.
//! - 윈도우 크기 1600×1000 (`system.md` §8 기본 크기)
//! - 4 영역:
//!   - TitleBar 44pt (상단)
//!   - Sidebar 260pt (좌측) + Body (가변, 우측) (중앙)
//!   - StatusBar 28pt (하단)
//! - 디자인 토큰 (`system.md` §4 색상) 직접 인라인
//! - Phase 1.3 에서 Empty State CTA 본격 컨텐츠 추가

use gpui::{
    App, Application, Context, IntoElement, ParentElement, Render, Styled, Window, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use tracing::info;

// ============================================================
// Design tokens — `system.md` §4 dark primary.
// ============================================================

pub mod tokens {
    /// 기본 배경 (윈도우 전체)
    pub const BG_BASE: u32 = 0x0a0a0b;
    /// 1차 surface (TitleBar, Sidebar, StatusBar, 카드)
    pub const BG_SURFACE: u32 = 0x131315;
    /// 2차 surface (hover, selected row)
    pub const BG_SURFACE_2: u32 = 0x1b1b1e;
    /// 3차 surface (active row)
    pub const BG_SURFACE_3: u32 = 0x232327;

    /// 제목 / 강조 텍스트
    pub const FG_PRIMARY: u32 = 0xf4f4f5;
    /// 본문
    pub const FG_SECONDARY: u32 = 0xb5b5bb;
    /// 메타 / 캡션 / 힌트
    pub const FG_MUTED: u32 = 0x6b6b73;
    /// 비활성
    pub const FG_DIM: u32 = 0x3f3f46;

    /// 기본 경계선
    pub const BORDER_SUBTLE: u32 = 0x2a2a2e;
    /// 강조 경계 (modal 등)
    pub const BORDER_STRONG: u32 = 0x3a3a40;

    /// MoAI 브랜드 오렌지
    pub const ACCENT_MOAI: u32 = 0xff6a3d;

    /// macOS traffic lights
    pub const TRAFFIC_RED: u32 = 0xff5f57;
    pub const TRAFFIC_YELLOW: u32 = 0xfebc2e;
    pub const TRAFFIC_GREEN: u32 = 0x28c840;
}

// ============================================================
// Root view — 4 영역 레이아웃 컨테이너
// ============================================================

pub struct RootView;

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tokens::BG_BASE))
            .child(title_bar())
            .child(main_body())
            .child(status_bar())
    }
}

// ============================================================
// 1) TitleBar — 44pt 상단
// ============================================================

fn title_bar() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(44.))
        .px_4()
        .gap_3()
        .bg(rgb(tokens::BG_SURFACE))
        .border_b_1()
        .border_color(rgb(tokens::BORDER_SUBTLE))
        // 좌측 — traffic lights placeholder (native 윈도우 chrome 사용 시 숨김 가능)
        .child(traffic_lights())
        // 프로젝트 이름 (현재 활성 워크스페이스)
        .child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_PRIMARY))
                .child("MoAI Studio"),
        )
        // 구분자
        .child(div().text_sm().text_color(rgb(tokens::FG_DIM)).child("/"))
        // 활성 워크스페이스 이름 (empty state 시에는 placeholder)
        .child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_SECONDARY))
                .child("no workspace"),
        )
}

/// macOS 전용 traffic lights (red/yellow/green). GPUI 자체 타이틀바 사용 시 생략 가능하나
/// 브랜드 일관성을 위해 인라인 렌더링.
fn traffic_lights() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(tokens::TRAFFIC_RED)),
        )
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(tokens::TRAFFIC_YELLOW)),
        )
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(tokens::TRAFFIC_GREEN)),
        )
}

// ============================================================
// 2) Main Body — Sidebar 260pt + 컨텐츠 영역
// ============================================================

fn main_body() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_grow()
        .w_full()
        .child(sidebar())
        .child(content_area())
}

/// Sidebar 260pt — WORKSPACE + GIT WORKTREES + SPECs 섹션.
fn sidebar() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w(px(260.))
        .h_full()
        .bg(rgb(tokens::BG_SURFACE))
        .border_r_1()
        .border_color(rgb(tokens::BORDER_SUBTLE))
        .px_3()
        .py_4()
        .gap_4()
        .child(sidebar_section(
            "WORKSPACE",
            vec![("No workspace yet", tokens::FG_MUTED)],
        ))
        .child(sidebar_section(
            "GIT WORKTREES",
            vec![("—", tokens::FG_DIM)],
        ))
        .child(sidebar_section("SPECS", vec![("—", tokens::FG_DIM)]))
        // 하단 "+" New Workspace 버튼 (Phase 1.4 에서 실동작)
        .child(div().flex_grow()) // 채움
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_2()
                .py_2()
                .rounded_md()
                .bg(rgb(tokens::BG_SURFACE_2))
                .text_color(rgb(tokens::FG_SECONDARY))
                .text_sm()
                .child("+ New Workspace"),
        )
}

/// Sidebar 내부 섹션 (ALL-CAPS 라벨 + 항목 리스트).
fn sidebar_section(label: &'static str, items: Vec<(&'static str, u32)>) -> impl IntoElement {
    let mut section = div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .text_color(rgb(tokens::FG_MUTED))
            .child(label),
    );
    for (text, color) in items {
        section = section.child(
            div()
                .text_sm()
                .text_color(rgb(color))
                .px_2()
                .py_1()
                .child(text),
        );
    }
    section
}

/// 컨텐츠 영역 — Phase 1.3 Empty State CTA (Create First / Start Sample / Open Recent).
fn content_area() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_grow()
        .h_full()
        .bg(rgb(tokens::BG_BASE))
        .justify_center()
        .items_center()
        .gap_4()
        .px_12()
        .child(empty_state_hero())
        .child(empty_state_primary_cta())
        .child(empty_state_secondary_cta_row())
        .child(empty_state_tip())
}

/// Hero: 큰 환영 제목 + 서브타이틀.
fn empty_state_hero() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_3xl()
                .text_color(rgb(tokens::FG_PRIMARY))
                .child("Welcome to MoAI Studio"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_MUTED))
                .child("SPEC-first native shell for Claude Code agents"),
        )
}

/// Primary CTA — `+ Create First Workspace` (MoAI 오렌지).
fn empty_state_primary_cta() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .mt_4()
        .px_6()
        .py_3()
        .rounded_lg()
        .bg(rgb(tokens::ACCENT_MOAI))
        .text_color(rgb(0xffffff))
        .text_sm()
        .child("+ Create First Workspace")
}

/// Secondary CTA 2 개 — Start Sample + Open Recent (가로 배치).
fn empty_state_secondary_cta_row() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .gap_3()
        .mt_2()
        .child(secondary_btn("Start Sample", "Guided tour"))
        .child(secondary_btn("Open Recent", "Last used workspace"))
}

fn secondary_btn(label: &'static str, subtitle: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_5()
        .py_3()
        .rounded_lg()
        .bg(rgb(tokens::BG_SURFACE))
        .border_1()
        .border_color(rgb(tokens::BORDER_SUBTLE))
        .child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_PRIMARY))
                .child(label),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(tokens::FG_MUTED))
                .child(subtitle),
        )
}

/// Bottom tip — Command Palette 발견성 힌트.
fn empty_state_tip() -> impl IntoElement {
    div()
        .mt_8()
        .text_xs()
        .text_color(rgb(tokens::FG_MUTED))
        .child("Tip: ⌘K opens Command Palette anytime")
}

// ============================================================
// 3) StatusBar — 28pt 하단
// ============================================================

fn status_bar() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(28.))
        .px_3()
        .gap_3()
        .bg(rgb(tokens::BG_SURFACE_2))
        .border_t_1()
        .border_color(rgb(tokens::BORDER_SUBTLE))
        .child(
            div()
                .text_xs()
                .text_color(rgb(tokens::FG_MUTED))
                .child("no git"),
        )
        .child(div().text_xs().text_color(rgb(tokens::FG_DIM)).child("·"))
        .child(
            div()
                .text_xs()
                .text_color(rgb(tokens::FG_MUTED))
                .child("moai-studio v0.1.0"),
        )
        .child(div().flex_grow())
        // 우측 — ⌘K 힌트 (Command Palette 발견성)
        .child(
            div()
                .text_xs()
                .text_color(rgb(tokens::FG_MUTED))
                .child("⌘K to search"),
        )
}

// ============================================================
// 앱 엔트리
// ============================================================

pub fn run_app() {
    info!("moai-studio-ui: GPUI Application 시작 (Phase 1.2 — 4 영역 레이아웃)");

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
        info!("moai-studio-ui: RootView 렌더 등록 완료 (TitleBar/Sidebar/Body/StatusBar)");
    });
}

/// 스캐폴드 hello 유지 (non-GPUI 경로용).
pub fn hello() {
    info!("moai-studio-ui: scaffold entry. GPUI 엔트리는 run_app()");
}
