//! Toolbar GPUI Component — main app action buttons (F-3)
//!
//! SPEC-V0-1-2-MENUS-001 REQ-F3-001: 7 action buttons in main toolbar.
//! 각 버튼은 기존 Action type에 연결되어 menu/keyboard와 동일한 dispatch 경로 사용.

use gpui::{div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window};

use crate::design::tokens as tok;

// 기존 action type import
use crate::{
    NewWorkspace, ToggleSidebar, OpenSettings, OpenCommandPalette, ToggleFind,
    NewTerminalSurface, OpenDocumentation
};

/// Toolbar Entity — 7 action buttons for main app
// @MX:NOTE: [AUTO] toolbar entity — main action 단일 진입점
// @MX:REASON: [AUTO] main UI action 집합. fan_in = 1 (RootView만 사용).
// SPEC: SPEC-V0-1-2-MENUS-001 F-3
pub struct Toolbar {
    /// 현재 sidebar visible 상태 (toggle button label용)
    sidebar_visible: bool,
}

impl Toolbar {
    /// 새 toolbar 생성
    pub fn new(sidebar_visible: bool) -> Self {
        Self { sidebar_visible }
    }

    /// sidebar visible 상태 갱신
    pub fn set_sidebar_visible(&mut self, visible: bool) {
        self.sidebar_visible = visible;
    }
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // toolbar button 공통 스타일
        let toolbar_btn = || {
            div()
                .px(px(12.))
                .py(px(6.))
                .rounded_md()
                .bg(rgb(tok::BG_ELEVATED))
                .text_color(rgb(tok::FG_SECONDARY))
                .text_sm()
                .hover(|s| s.bg(rgb(tok::BG_ELEVATED)))
                .cursor_pointer()
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .py(px(6.))
            .px(px(12.))
            .bg(rgb(tok::BG_SURFACE))
            .border_b_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            // 1. New Workspace (Cmd+N)
            .child(
                toolbar_btn()
                    .id("toolbar-new-workspace")
                    .on_action(cx.listener(|_this, _: &NewWorkspace, _window, _cx| {
                        // NewWorkspace action dispatch (handled by RootView)
                    }))
                    .child("New Workspace"),
            )
            // 2. Toggle Sidebar (Cmd+B)
            .child(
                toolbar_btn()
                    .id("toolbar-toggle-sidebar")
                    .on_action(cx.listener(|_this, _: &ToggleSidebar, _window, _cx| {
                        // ToggleSidebar action dispatch (handled by RootView)
                    }))
                    .child(if self.sidebar_visible { "Hide Sidebar" } else { "Show Sidebar" }),
            )
            // 3. Settings (Cmd+,)
            .child(
                toolbar_btn()
                    .id("toolbar-settings")
                    .on_action(cx.listener(|_this, _: &OpenSettings, _window, _cx| {
                        // OpenSettings action dispatch (handled by RootView)
                    }))
                    .child("Settings"),
            )
            // 4. Command Palette (Cmd+K)
            .child(
                toolbar_btn()
                    .id("toolbar-command-palette")
                    .on_action(cx.listener(|_this, _: &OpenCommandPalette, _window, _cx| {
                        // OpenCommandPalette action dispatch (handled by RootView)
                    }))
                    .child("Command Palette"),
            )
            // 5. New Terminal
            .child(
                toolbar_btn()
                    .id("toolbar-new-terminal")
                    .on_action(cx.listener(|_this, _: &NewTerminalSurface, _window, _cx| {
                        // NewTerminalSurface action dispatch (handled by RootView)
                    }))
                    .child("New Terminal"),
            )
            // 6. Find (Cmd+F)
            .child(
                toolbar_btn()
                    .id("toolbar-find")
                    .on_action(cx.listener(|_this, _: &ToggleFind, _window, _cx| {
                        // ToggleFind action dispatch (handled by RootView)
                    }))
                    .child("Find"),
            )
            // 7. Documentation
            .child(
                toolbar_btn()
                    .id("toolbar-documentation")
                    .on_action(cx.listener(|_this, _: &OpenDocumentation, _window, _cx| {
                        // OpenDocumentation action dispatch (handled by RootView)
                    }))
                    .child("Documentation"),
            )
    }
}
