//! MoAI Studio UI 컴포넌트 라이브러리.
//!
//! Phase 1.6 (SPEC-V3-001 RG-V3-2) — Sidebar workspace 리스트 + active 하이라이트.
//!
//! ## 설계
//! - `run_app(workspaces)` 이 유일한 엔트리. `moai-studio-app` 바이너리가 호출.
//! - 윈도우 크기 1600×1000 (`system.md` §8 기본 크기)
//! - 4 영역:
//!   - TitleBar 44pt (상단, 활성 워크스페이스 이름 표시)
//!   - Sidebar 260pt (좌측, workspace 리스트) + Body (가변, 우측)
//!   - StatusBar 28pt (하단)
//! - Empty state CTA 는 workspaces 가 비었을 때만 body 에 표시

use gpui::{
    App, Application, Context, IntoElement, MouseButton, ParentElement, Render, Styled, Window,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use moai_studio_workspace::{Workspace, WorkspacesStore};
use std::path::PathBuf;
use tracing::{error, info};

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

/// 앱 전역 상태 — Phase 1.7: workspace 리스트 + active id + storage path (버튼 클릭 시 재로드).
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
}

impl RootView {
    pub fn new(workspaces: Vec<Workspace>, storage_path: PathBuf) -> Self {
        // 가장 최근 활성 워크스페이스를 자동 선택 (last_active 최댓값).
        let active_id = workspaces
            .iter()
            .max_by_key(|w| w.last_active)
            .map(|w| w.id.clone());
        Self {
            workspaces,
            active_id,
            storage_path,
        }
    }

    /// 현재 활성 워크스페이스 레퍼런스.
    pub fn active(&self) -> Option<&Workspace> {
        let id = self.active_id.as_deref()?;
        self.workspaces.iter().find(|w| w.id == id)
    }

    /// TitleBar 에 표시할 워크스페이스 이름 (없으면 placeholder).
    pub fn title_label(&self) -> &str {
        self.active()
            .map(|w| w.name.as_str())
            .unwrap_or("no workspace")
    }

    /// 새 워크스페이스가 저장소에 추가된 이후 로컬 상태를 갱신.
    /// GPUI 이벤트 핸들러와 독립적으로 테스트 가능.
    pub fn apply_added_workspace(&mut self, added: &Workspace, all: Vec<Workspace>) {
        self.workspaces = all;
        self.active_id = Some(added.id.clone());
    }

    /// + New Workspace 버튼 클릭 처리 — store 재로드, 네이티브 picker, 상태 갱신.
    ///   사용자가 취소하거나 로드/저장이 실패하면 상태 유지.
    fn handle_add_workspace(&mut self, cx: &mut Context<Self>) {
        let mut store = match WorkspacesStore::load(&self.storage_path) {
            Ok(s) => s,
            Err(e) => {
                error!("WorkspacesStore::load 실패: {e}");
                return;
            }
        };
        match moai_studio_workspace::pick_and_save(&mut store) {
            Ok(Some(ws)) => {
                let all = store.list().to_vec();
                self.apply_added_workspace(&ws, all);
                cx.notify();
            }
            Ok(None) => info!("pick_and_save: 사용자 취소"),
            Err(e) => error!("pick_and_save 실패: {e}"),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let new_ws_btn = new_workspace_button().on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, cx| this.handle_add_workspace(cx)),
        );
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tokens::BG_BASE))
            .child(title_bar(self.title_label()))
            .child(main_body(
                &self.workspaces,
                self.active_id.as_deref(),
                new_ws_btn,
            ))
            .child(status_bar())
    }
}

/// 인터랙션 가능한 "+ New Workspace" 버튼 (id 필수 — StatefulInteractiveElement).
fn new_workspace_button() -> gpui::Stateful<gpui::Div> {
    div()
        .id("new-workspace-btn")
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
        .hover(|s| s.bg(rgb(tokens::BG_SURFACE_3)))
        .cursor_pointer()
        .child("+ New Workspace")
}

// ============================================================
// 1) TitleBar — 44pt 상단
// ============================================================

fn title_bar(active_label: &str) -> impl IntoElement {
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
                .child(active_label.to_string()),
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

fn main_body(
    workspaces: &[Workspace],
    active_id: Option<&str>,
    new_ws_btn: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_grow()
        .w_full()
        .child(sidebar(workspaces, active_id, new_ws_btn))
        .child(content_area(workspaces.is_empty()))
}

/// Sidebar 260pt — WORKSPACE + GIT WORKTREES + SPECs 섹션 + 하단 인터랙티브 "+ New Workspace".
fn sidebar(
    workspaces: &[Workspace],
    active_id: Option<&str>,
    new_ws_btn: impl IntoElement,
) -> impl IntoElement {
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
        .child(workspace_section(workspaces, active_id))
        .child(sidebar_section(
            "GIT WORKTREES",
            vec![("—", tokens::FG_DIM)],
        ))
        .child(sidebar_section("SPECS", vec![("—", tokens::FG_DIM)]))
        .child(div().flex_grow())
        .child(new_ws_btn)
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

/// WORKSPACE 섹션 — 비었으면 placeholder, 있으면 각 워크스페이스 row 렌더.
fn workspace_section(workspaces: &[Workspace], active_id: Option<&str>) -> impl IntoElement {
    let mut section = div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .text_color(rgb(tokens::FG_MUTED))
            .child("WORKSPACE"),
    );

    if workspaces.is_empty() {
        section = section.child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_MUTED))
                .px_2()
                .py_1()
                .child("No workspace yet"),
        );
    } else {
        for ws in workspaces {
            section = section.child(workspace_row(ws, active_id == Some(ws.id.as_str())));
        }
    }
    section
}

/// 단일 워크스페이스 row — 컬러 dot + 이름. Active 시 하이라이트.
fn workspace_row(ws: &Workspace, is_active: bool) -> impl IntoElement {
    let bg = if is_active {
        tokens::BG_SURFACE_3
    } else {
        tokens::BG_SURFACE
    };
    let fg = if is_active {
        tokens::FG_PRIMARY
    } else {
        tokens::FG_SECONDARY
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgb(bg))
        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(rgb(ws.color)))
        .child(div().text_sm().text_color(rgb(fg)).child(ws.name.clone()))
}

/// 컨텐츠 영역 — workspace 가 비었을 때만 Empty State CTA 표시.
fn content_area(show_empty_state: bool) -> impl IntoElement {
    let mut area = div()
        .flex()
        .flex_col()
        .flex_grow()
        .h_full()
        .bg(rgb(tokens::BG_BASE))
        .justify_center()
        .items_center()
        .gap_4()
        .px_12();
    if show_empty_state {
        area = area
            .child(empty_state_hero())
            .child(empty_state_primary_cta())
            .child(empty_state_secondary_cta_row())
            .child(empty_state_tip());
    } else {
        // 워크스페이스 선택된 상태의 플레이스홀더 — Phase 2+ 에서 터미널/에디터로 대체.
        area = area.child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_MUTED))
                .child("Workspace selected — terminal/editor coming in Phase 2"),
        );
    }
    area
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

pub fn run_app(workspaces: Vec<Workspace>, storage_path: PathBuf) {
    info!(
        "moai-studio-ui: GPUI Application 시작 (Phase 1.7 — workspaces={}, store={})",
        workspaces.len(),
        storage_path.display()
    );

    Application::new().run(move |cx: &mut App| {
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

        let ws = workspaces.clone();
        let path = storage_path.clone();
        cx.open_window(options, move |_window, cx| {
            cx.new(|_cx| RootView::new(ws, path))
        })
        .expect("GPUI 윈도우 생성 실패");

        cx.activate(true);
        info!("moai-studio-ui: RootView 렌더 등록 완료 (+ New Workspace 버튼 배선)");
    });
}

/// 스캐폴드 hello 유지 (non-GPUI 경로용).
pub fn hello() {
    info!("moai-studio-ui: scaffold entry. GPUI 엔트리는 run_app(workspaces)");
}

// ============================================================
// 유닛 테스트 — RootView 상태 로직 (GPUI 렌더 제외)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_ws(name: &str, id_suffix: &str, last_active: u64) -> Workspace {
        Workspace {
            id: format!("ws-{}", id_suffix),
            name: name.to_string(),
            project_path: PathBuf::from(format!("/tmp/{}", name)),
            moai_config: PathBuf::from(".moai"),
            color: 0xff6a3d,
            last_active,
        }
    }

    fn dummy_path() -> PathBuf {
        PathBuf::from("/tmp/moai-studio-ui-tests-workspaces.json")
    }

    #[test]
    fn root_view_empty_workspaces_has_no_active_and_placeholder_label() {
        let view = RootView::new(vec![], dummy_path());
        assert!(view.active_id.is_none());
        assert!(view.active().is_none());
        assert_eq!(view.title_label(), "no workspace");
    }

    #[test]
    fn root_view_picks_most_recently_active_workspace_as_active() {
        let older = make_ws("alpha", "1", 1_000);
        let newer = make_ws("beta", "2", 9_000);
        let view = RootView::new(vec![older, newer.clone()], dummy_path());
        assert_eq!(view.active_id.as_deref(), Some(newer.id.as_str()));
        assert_eq!(view.title_label(), "beta");
        assert_eq!(view.active().map(|w| w.name.as_str()), Some("beta"));
    }

    #[test]
    fn root_view_active_returns_none_when_active_id_missing() {
        let mut view = RootView::new(vec![make_ws("alpha", "1", 1_000)], dummy_path());
        view.active_id = Some("ws-does-not-exist".to_string());
        assert!(view.active().is_none());
        assert_eq!(view.title_label(), "no workspace");
    }

    #[test]
    fn apply_added_workspace_from_empty_sets_active_to_new() {
        let mut view = RootView::new(vec![], dummy_path());
        let added = make_ws("new-proj", "new1", 5_000);
        view.apply_added_workspace(&added, vec![added.clone()]);
        assert_eq!(view.workspaces.len(), 1);
        assert_eq!(view.active_id.as_deref(), Some(added.id.as_str()));
        assert_eq!(view.title_label(), "new-proj");
    }

    #[test]
    fn apply_added_workspace_switches_active_even_if_others_newer() {
        // last_active 가 더 오래된 항목을 추가해도 "방금 추가한 것" 을 active 로 강제.
        let existing = make_ws("alpha", "1", 9_999);
        let added = make_ws("new-proj", "new1", 1_000);
        let mut view = RootView::new(vec![existing.clone()], dummy_path());
        assert_eq!(view.active_id.as_deref(), Some(existing.id.as_str()));

        view.apply_added_workspace(&added, vec![existing, added.clone()]);
        assert_eq!(view.workspaces.len(), 2);
        assert_eq!(view.active_id.as_deref(), Some(added.id.as_str()));
        assert_eq!(view.title_label(), "new-proj");
    }
}
