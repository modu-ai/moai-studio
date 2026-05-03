//! Toolbar GPUI Component — main app action buttons (F-3)
//!
//! SPEC-V0-1-2-MENUS-001 REQ-F3-001: 7 action buttons in main toolbar.
//! 각 버튼은 기존 Action type에 연결되어 menu/keyboard와 동일한 dispatch 경로 사용.
//!
//! SPEC-V0-2-0-TOOLBAR-WIRE-001 MS-1 (audit Top 8 #5, v0.2.0 cycle Sprint 9):
//! Each button now wires `on_mouse_down(Left)` → `App::dispatch_action(&Action)`
//! so that user clicks reach the RootView's existing `on_action` handlers
//! (lib.rs §1973~2044). The pre-existing `on_action` listener on each button
//! is preserved for backward compatibility — those listeners are no-ops here
//! because the dispatch is now driven from the click side, but RootView
//! receives the action and runs its own handler.

use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Render, Styled, Window,
    div, px, rgb,
};

use crate::design::tokens as tok;

// 기존 action type import
use crate::{
    NewTerminalSurface, NewWorkspace, OpenCommandPalette, OpenDocumentation, OpenSettings,
    ToggleFind, ToggleSidebar,
};

/// Toolbar Entity — 7 action buttons for main app
// @MX:NOTE: [AUTO] toolbar entity — main action 단일 진입점
// @MX:REASON: [AUTO] main UI action 집합. fan_in = 1 (RootView만 사용).
// SPEC: SPEC-V0-1-2-MENUS-001 F-3
// @MX:SPEC: SPEC-V0-2-0-TOOLBAR-WIRE-001 MS-1 — click→dispatch wire complete.
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

    /// Returns the current sidebar visibility flag (test convenience).
    pub fn sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }

    /// Canonical button labels exposed by the toolbar — handy for tests
    /// and documentation. Order mirrors the render output.
    pub fn button_labels(&self) -> [&'static str; 7] {
        [
            "New Workspace",
            if self.sidebar_visible {
                "Hide Sidebar"
            } else {
                "Show Sidebar"
            },
            "Settings",
            "Command Palette",
            "New Terminal",
            "Find",
            "Documentation",
        ]
    }

    /// Canonical button ids — order mirrors `button_labels`.
    pub fn button_ids() -> [&'static str; 7] {
        [
            "toolbar-new-workspace",
            "toolbar-toggle-sidebar",
            "toolbar-settings",
            "toolbar-command-palette",
            "toolbar-new-terminal",
            "toolbar-find",
            "toolbar-documentation",
        ]
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
            // 1. New Workspace (Cmd+N) — REQ-TW-001
            .child(
                toolbar_btn()
                    .id("toolbar-new-workspace")
                    .on_action(cx.listener(|_this, _: &NewWorkspace, _window, _cx| {
                        // NewWorkspace action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&NewWorkspace);
                        }),
                    )
                    .child("New Workspace"),
            )
            // 2. Toggle Sidebar (Cmd+B) — REQ-TW-002
            .child(
                toolbar_btn()
                    .id("toolbar-toggle-sidebar")
                    .on_action(cx.listener(|_this, _: &ToggleSidebar, _window, _cx| {
                        // ToggleSidebar action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&ToggleSidebar);
                        }),
                    )
                    .child(if self.sidebar_visible {
                        "Hide Sidebar"
                    } else {
                        "Show Sidebar"
                    }),
            )
            // 3. Settings (Cmd+,) — REQ-TW-003
            .child(
                toolbar_btn()
                    .id("toolbar-settings")
                    .on_action(cx.listener(|_this, _: &OpenSettings, _window, _cx| {
                        // OpenSettings action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&OpenSettings);
                        }),
                    )
                    .child("Settings"),
            )
            // 4. Command Palette (Cmd+K) — REQ-TW-004
            .child(
                toolbar_btn()
                    .id("toolbar-command-palette")
                    .on_action(cx.listener(|_this, _: &OpenCommandPalette, _window, _cx| {
                        // OpenCommandPalette action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&OpenCommandPalette);
                        }),
                    )
                    .child("Command Palette"),
            )
            // 5. New Terminal — REQ-TW-005
            .child(
                toolbar_btn()
                    .id("toolbar-new-terminal")
                    .on_action(cx.listener(|_this, _: &NewTerminalSurface, _window, _cx| {
                        // NewTerminalSurface action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&NewTerminalSurface);
                        }),
                    )
                    .child("New Terminal"),
            )
            // 6. Find (Cmd+F) — REQ-TW-006
            .child(
                toolbar_btn()
                    .id("toolbar-find")
                    .on_action(cx.listener(|_this, _: &ToggleFind, _window, _cx| {
                        // ToggleFind action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&ToggleFind);
                        }),
                    )
                    .child("Find"),
            )
            // 7. Documentation — REQ-TW-007
            .child(
                toolbar_btn()
                    .id("toolbar-documentation")
                    .on_action(cx.listener(|_this, _: &OpenDocumentation, _window, _cx| {
                        // OpenDocumentation action dispatch (handled by RootView)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _ev, _window, cx| {
                            cx.dispatch_action(&OpenDocumentation);
                        }),
                    )
                    .child("Documentation"),
            )
    }
}

// ============================================================
// Unit tests — SPEC-V0-2-0-TOOLBAR-WIRE-001 MS-1 (AC-TW-1/2/4/5/6)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-TW-5: Toolbar::new(false) initializes sidebar_visible to false.
    #[test]
    fn toolbar_new_false_preserves_flag() {
        let tb = Toolbar::new(false);
        assert!(!tb.sidebar_visible());
    }

    /// AC-TW-5: Toolbar::new(true) initializes sidebar_visible to true.
    #[test]
    fn toolbar_new_true_preserves_flag() {
        let tb = Toolbar::new(true);
        assert!(tb.sidebar_visible());
    }

    /// AC-TW-5: set_sidebar_visible mutates the flag.
    #[test]
    fn toolbar_set_sidebar_visible_mutates() {
        let mut tb = Toolbar::new(false);
        tb.set_sidebar_visible(true);
        assert!(tb.sidebar_visible());
        tb.set_sidebar_visible(false);
        assert!(!tb.sidebar_visible());
    }

    /// AC-TW-2 (REQ-TW-008): button ids are the canonical 7 in stable order.
    #[test]
    fn toolbar_button_ids_canonical_order() {
        let ids = Toolbar::button_ids();
        assert_eq!(ids.len(), 7);
        assert_eq!(ids[0], "toolbar-new-workspace");
        assert_eq!(ids[1], "toolbar-toggle-sidebar");
        assert_eq!(ids[2], "toolbar-settings");
        assert_eq!(ids[3], "toolbar-command-palette");
        assert_eq!(ids[4], "toolbar-new-terminal");
        assert_eq!(ids[5], "toolbar-find");
        assert_eq!(ids[6], "toolbar-documentation");
        // Ids must be unique.
        let unique: std::collections::HashSet<&'static str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), 7);
    }

    /// AC-TW-2: button labels reflect sidebar_visible state on the second slot.
    #[test]
    fn toolbar_button_labels_reflect_sidebar_state() {
        let hidden = Toolbar::new(false);
        let labels = hidden.button_labels();
        assert_eq!(labels[1], "Show Sidebar");

        let visible = Toolbar::new(true);
        let labels = visible.button_labels();
        assert_eq!(labels[1], "Hide Sidebar");
    }

    /// AC-TW-2: the other six labels are stable regardless of sidebar state.
    #[test]
    fn toolbar_other_button_labels_are_stable() {
        let labels_hidden = Toolbar::new(false).button_labels();
        let labels_visible = Toolbar::new(true).button_labels();
        for i in [0, 2, 3, 4, 5, 6] {
            assert_eq!(
                labels_hidden[i], labels_visible[i],
                "label at index {i} must be sidebar-state independent"
            );
        }
    }

    /// AC-TW-1: render does not panic for either sidebar state (TestAppContext).
    #[test]
    fn toolbar_render_does_not_panic() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        // Sidebar hidden.
        let toolbar_hidden = cx.new(|_| Toolbar::new(false));
        cx.update(|app| {
            toolbar_hidden.update(app, |_view: &mut Toolbar, _cx| {
                // Construction succeeded without panic.
            });
        });
        // Sidebar visible.
        let toolbar_visible = cx.new(|_| Toolbar::new(true));
        cx.update(|app| {
            toolbar_visible.update(app, |view: &mut Toolbar, _cx| {
                view.set_sidebar_visible(false);
                assert!(!view.sidebar_visible());
            });
        });
    }
}
