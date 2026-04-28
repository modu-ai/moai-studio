// WebViewSurface GPUI Entity for SPEC-V3-007 MS-1/MS-2
//
// This module implements the GPUI Entity that renders the web browser chrome
// (URL bar + status bar + webview placeholder area).
//
// REQ-WB-001: WebViewSurface is a GPUI Entity with impl Render
// REQ-WB-005: If backend unavailable, render placeholder (no panic)
// MS-2: NavigationHistory integration, URL validation, DevTools support

use crate::design::tokens as tok;
use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Div, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px,
    rgb,
};

use super::history::NavigationHistory;
use super::url::validate_url;
use super::bridge::BridgeRouter;

/// WebView state enumeration (REQ-WB-006)
///
/// Represents the current state of the webview.
#[derive(Debug, Clone, PartialEq)]
pub enum WebViewState {
    /// Page is currently loading
    Loading,
    /// Page is ready and interactive
    Ready,
    /// Page encountered an error
    Error(String),
    /// Page has crashed (webview process terminated)
    Crashed,
}

/// WebViewSurface GPUI Entity
///
/// Renders a web browser chrome with URL bar, navigation buttons,
/// status bar, and webview content area. Gracefully degrades when
/// wry backend is not available (REQ-WB-005).
///
/// # Fields (MS-3 updated)
/// * `url_bar_text` - Current URL bar content
/// * `status_message` - Status bar text (e.g., "Loading...", "Ready")
/// * `backend_available` - Whether wry backend is available
/// * `history` - Navigation history with cursor tracking
/// * `devtools_open` - Whether DevTools panel is open
/// * `workspace_id` - Optional workspace ID for sandbox data directory
/// * `last_error` - Last validation or navigation error
/// * `state` - WebView state (Loading, Ready, Error, Crashed) - MS-3 REQ-WB-006
/// * `bridge` - Optional JS bridge router - MS-3
pub struct WebViewSurface {
    /// Current URL bar content
    url_bar_text: String,
    /// Status bar text showing loading state or errors
    status_message: String,
    /// Whether wry backend is available (feature flag check)
    backend_available: bool,
    /// Navigation history (MS-2: replaced Vec<String> with NavigationHistory)
    history: NavigationHistory,
    /// DevTools open state (MS-2)
    devtools_open: bool,
    /// Optional workspace ID for sandbox data directory (MS-2)
    workspace_id: Option<String>,
    /// Last error message (MS-2)
    last_error: Option<String>,
    /// WebView state (MS-3: REQ-WB-006)
    state: WebViewState,
    /// JS bridge router (MS-3)
    bridge: Option<BridgeRouter>,
}

impl WebViewSurface {
    /// Create a new WebViewSurface
    ///
    /// # Arguments
    /// * `url` - Initial URL to display (default: "https://example.com")
    ///
    /// # Example
    /// ```
    /// use moai_studio_ui::web::WebViewSurface;
    ///
    /// let surface = WebViewSurface::new("https://example.com");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        let url_text = url.into();
        Self {
            url_bar_text: url_text.clone(),
            status_message: "Ready".to_string(),
            backend_available: cfg!(feature = "web"),
            history: NavigationHistory::new(url_text),
            devtools_open: false,
            workspace_id: None,
            last_error: None,
            state: WebViewState::Ready,
            bridge: None,
        }
    }

    /// Navigate to a new URL (MS-2: with validation)
    ///
    /// Validates URL first. On error, sets status_message to "Blocked: {reason}"
    /// and returns without navigating. On success, updates history and status.
    pub fn navigate(&mut self, url: impl Into<String>) {
        let url = url.into();

        // MS-2: Validate URL before navigation
        match validate_url(&url) {
            Ok(sanitized) => {
                self.url_bar_text = sanitized.clone();
                self.status_message = "Loading...".to_string();
                self.last_error = None;

                // MS-2: Use NavigationHistory::navigate
                self.history.navigate(sanitized);
            }
            Err(err) => {
                // MS-2: Show validation error
                self.status_message = format!("Blocked: {}", err);
                self.last_error = Some(err.to_string());
            }
        }
    }

    /// Go back in history (MS-2: delegates to NavigationHistory)
    ///
    /// Returns true if successful, false if already at beginning
    pub fn go_back(&mut self) -> bool {
        if self.history.go_back() {
            self.url_bar_text = self.history.current().url.clone();
            self.status_message = "Loading...".to_string();
            true
        } else {
            false
        }
    }

    /// Go forward in history (MS-2: delegates to NavigationHistory)
    ///
    /// Returns true if successful, false if already at end
    pub fn go_forward(&mut self) -> bool {
        if self.history.go_forward() {
            self.url_bar_text = self.history.current().url.clone();
            self.status_message = "Loading...".to_string();
            true
        } else {
            false
        }
    }

    /// Reload current page
    pub fn reload(&mut self) {
        self.status_message = "Reloading...".to_string();
    }

    /// Set status message (e.g., "Loaded", "Error", "Loading...")
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    /// Toggle DevTools panel (MS-2)
    pub fn toggle_devtools(&mut self) {
        self.devtools_open = !self.devtools_open;
        self.status_message = if self.devtools_open {
            "DevTools: Open".to_string()
        } else {
            "DevTools: Closed".to_string()
        };
    }

    /// Check if DevTools is open (MS-2)
    pub fn devtools_is_open(&self) -> bool {
        self.devtools_open
    }

    /// Set workspace ID for sandbox data directory (MS-2)
    pub fn set_workspace_id(&mut self, id: String) {
        self.workspace_id = Some(id);
    }

    /// Report a webview crash (MS-3: REQ-WB-006)
    ///
    /// Transitions the state to Crashed, which will render
    /// a "Page crashed. Reload?" message in the webview area.
    pub fn report_crash(&mut self) {
        self.state = WebViewState::Crashed;
        self.status_message = "Page crashed".to_string();
    }

    /// Recover from a crash by reloading (MS-3: REQ-WB-006)
    ///
    /// Transitions state back to Loading and attempts to reload
    /// the current URL.
    pub fn recover_from_crash(&mut self) {
        if self.state == WebViewState::Crashed {
            self.state = WebViewState::Loading;
            self.status_message = "Reloading...".to_string();
            // In a full implementation, this would trigger backend.reload()
        }
    }

    /// Get the current webview state (MS-3)
    pub fn state(&self) -> &WebViewState {
        &self.state
    }

    /// Set the webview state (MS-3)
    pub fn set_state(&mut self, state: WebViewState) {
        self.state = state;
        match &self.state {
            WebViewState::Loading => {
                self.status_message = "Loading...".to_string();
            }
            WebViewState::Ready => {
                self.status_message = "Ready".to_string();
            }
            WebViewState::Error(msg) => {
                self.status_message = format!("Error: {}", msg);
            }
            WebViewState::Crashed => {
                self.status_message = "Page crashed".to_string();
            }
        }
    }

    /// Get the bridge router (MS-3)
    pub fn bridge(&self) -> Option<&BridgeRouter> {
        self.bridge.as_ref()
    }

    /// Set the bridge router (MS-3)
    pub fn set_bridge(&mut self, bridge: BridgeRouter) {
        self.bridge = Some(bridge);
    }
}

impl Render for WebViewSurface {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // REQ-WB-005: Graceful degradation when backend unavailable
        if !self.backend_available {
            return self.render_unavailable();
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_APP))
            // Top bar: URL bar + navigation buttons
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .bg(rgb(tok::BG_SURFACE))
                    .border_b_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    // MS-2: Back button (enabled/disabled based on can_go_back)
                    .child(nav_button("←", "Back", self.history.can_go_back()))
                    // MS-2: Forward button (enabled/disabled based on can_go_forward)
                    .child(nav_button("→", "Forward", self.history.can_go_forward()))
                    // Reload button
                    .child(nav_button("⟳", "Reload", true))
                    // URL text input placeholder
                    .child(
                        div()
                            .flex_grow()
                            .px_3()
                            .py_1()
                            .bg(rgb(tok::BG_PANEL))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(self.url_bar_text.clone()),
                    ),
            )
            // Content area: WebView placeholder or crash recovery (MS-3: REQ-WB-006)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .justify_center()
                    .items_center()
                    .bg(rgb(tok::BG_PANEL))
                    .when(self.state == WebViewState::Crashed, |this| {
                        // MS-3: REQ-WB-006 - Show crash recovery UI
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .items_center()
                                .px_6()
                                .py_4()
                                .bg(rgb(tok::BG_SURFACE))
                                .rounded_lg()
                                .border_1()
                                .border_color(rgb(tok::semantic::DANGER))
                                .child(
                                    div()
                                        .text_lg()
                                        .text_color(rgb(tok::semantic::DANGER))
                                        .child("Page crashed"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(tok::FG_SECONDARY))
                                        .child("The webview process has terminated unexpectedly."),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(tok::FG_PRIMARY))
                                        .child("Reload?"),
                                ),
                        )
                    })
                    .when(self.state != WebViewState::Crashed, |this| {
                        // Normal webview placeholder
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(tok::FG_MUTED))
                                .child("WebView will render here (MS-2 integration)"),
                        )
                    }),
            )
            // MS-2: Bottom status bar with error display and DevTools indicator
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .bg(rgb(tok::BG_SURFACE))
                    .border_t_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .text_xs()
                    // MS-2: Show status message (or default if error shown)
                    .child(
                        div()
                            .text_color(if self.last_error.is_some() {
                                rgb(tok::FG_MUTED)
                            } else {
                                rgb(tok::FG_SECONDARY)
                            })
                            .child(self.status_message.clone()),
                    )
                    // MS-2: Show error in red if present
                    .when_some(self.last_error.clone(), |this, error| {
                        this.child(div().text_color(rgb(tok::semantic::DANGER)).child(error))
                    })
                    // MS-2: DevTools indicator
                    .when(self.devtools_open, |this| {
                        this.child(
                            div()
                                .text_color(rgb(tok::semantic::INFO))
                                .child("DevTools: Open"),
                        )
                    }),
            )
    }
}

impl WebViewSurface {
    /// Render placeholder when WebView backend is unavailable (REQ-WB-005)
    fn render_unavailable(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_APP))
            .justify_center()
            .items_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .items_center()
                    .px_6()
                    .py_4()
                    .bg(rgb(tok::BG_SURFACE))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child("WebView unavailable"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_MUTED))
                            .child("Install webkit2gtk for WebView support"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_DISABLED))
                            .child("Enable with: cargo build --features web"),
                    ),
            )
    }
}

/// Navigation button styling helper (MS-2: added enabled parameter)
fn nav_button(label: &'static str, _tooltip: &'static str, enabled: bool) -> Div {
    use gpui::StyleRefinement;
    let color = if enabled {
        rgb(tok::FG_PRIMARY)
    } else {
        rgb(tok::FG_MUTED)
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(28.))
        .rounded_md()
        .text_sm()
        .text_color(color)
        .when(enabled, |this| {
            this.hover(|s: StyleRefinement| s.bg(rgb(tok::BG_ELEVATED)))
                .cursor_pointer()
        })
        .child(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview_surface_new_creates_with_initial_url() {
        let surface = WebViewSurface::new("https://example.com");
        assert_eq!(surface.url_bar_text, "https://example.com");
        assert_eq!(surface.status_message, "Ready");
        assert_eq!(surface.history.len(), 1);
        assert_eq!(surface.history.cursor(), 0);
    }

    #[test]
    fn webview_surface_navigate_updates_url_and_history() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");

        assert_eq!(surface.url_bar_text, "https://modu.ai");
        assert_eq!(surface.status_message, "Loading...");
        assert_eq!(surface.history.len(), 2);
        assert_eq!(surface.history.cursor(), 1);
        assert_eq!(surface.history.current().url, "https://modu.ai");
    }

    #[test]
    fn webview_surface_navigate_validates_and_blocks_unsafe_urls() {
        let mut surface = WebViewSurface::new("https://example.com");

        // javascript: should be blocked
        surface.navigate("javascript:alert(1)");
        assert!(surface.last_error.is_some());
        assert!(surface.status_message.contains("Blocked"));

        // data: should be blocked
        surface.navigate("data:text/html,test");
        assert!(surface.last_error.is_some());

        // Empty should be blocked
        surface.navigate("");
        assert!(surface.last_error.is_some());
    }

    #[test]
    fn webview_surface_navigate_prepends_https_to_urls_without_scheme() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("example.com");

        assert_eq!(surface.url_bar_text, "https://example.com");
        assert_eq!(surface.history.current().url, "https://example.com");
        assert!(surface.last_error.is_none());
    }

    #[test]
    fn webview_surface_go_back_works() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");
        surface.navigate("https://github.com");

        let success = surface.go_back();
        assert!(success);
        assert_eq!(surface.url_bar_text, "https://modu.ai");
        assert_eq!(surface.history.cursor(), 1);
    }

    #[test]
    fn webview_surface_go_back_at_beginning_returns_false() {
        let mut surface = WebViewSurface::new("https://example.com");
        let success = surface.go_back();
        assert!(!success);
        assert_eq!(surface.url_bar_text, "https://example.com");
    }

    #[test]
    fn webview_surface_go_forward_works() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");
        surface.go_back();
        let success = surface.go_forward();

        assert!(success);
        assert_eq!(surface.url_bar_text, "https://modu.ai");
        assert_eq!(surface.history.cursor(), 1);
    }

    #[test]
    fn webview_surface_go_forward_at_end_returns_false() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");
        let success = surface.go_forward();
        assert!(!success);
    }

    #[test]
    fn webview_surface_reload_updates_status() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.reload();
        assert_eq!(surface.status_message, "Reloading...");
    }

    #[test]
    fn webview_surface_set_status_works() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.set_status("Loaded");
        assert_eq!(surface.status_message, "Loaded");
    }

    #[test]
    fn webview_surface_history_max_100_entries() {
        let mut surface = WebViewSurface::new("https://example.com");
        // Add 101 URLs
        for i in 0..=100 {
            surface.navigate(format!("https://example.com/{}", i));
        }
        // History should be capped at 100
        assert_eq!(surface.history.len(), 100);
    }

    #[test]
    fn webview_surface_backend_available_reflects_feature_flag() {
        let surface = WebViewSurface::new("https://example.com");
        // backend_available should match cfg!(feature = "web")
        assert_eq!(surface.backend_available, cfg!(feature = "web"));
    }

    // MS-2 tests

    #[test]
    fn webview_surface_toggle_devtools() {
        let mut surface = WebViewSurface::new("https://example.com");
        assert!(!surface.devtools_is_open());

        surface.toggle_devtools();
        assert!(surface.devtools_is_open());
        assert_eq!(surface.status_message, "DevTools: Open");

        surface.toggle_devtools();
        assert!(!surface.devtools_is_open());
        assert_eq!(surface.status_message, "DevTools: Closed");
    }

    #[test]
    fn webview_surface_set_workspace_id() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.set_workspace_id("test-workspace".to_string());
        assert_eq!(surface.workspace_id, Some("test-workspace".to_string()));
    }

    #[test]
    fn webview_surface_can_go_back_forward() {
        let mut surface = WebViewSurface::new("https://example.com");

        // Initially can't go back or forward
        assert!(!surface.history.can_go_back());
        assert!(!surface.history.can_go_forward());

        surface.navigate("https://modu.ai");
        // Can go back, but not forward
        assert!(surface.history.can_go_back());
        assert!(!surface.history.can_go_forward());

        surface.go_back();
        // Can go forward, but not back
        assert!(!surface.history.can_go_back());
        assert!(surface.history.can_go_forward());
    }

    // MS-3 tests for WebViewState and crash recovery (REQ-WB-006)

    #[test]
    fn webview_surface_initial_state_is_ready() {
        let surface = WebViewSurface::new("https://example.com");
        assert_eq!(surface.state(), &WebViewState::Ready);
    }

    #[test]
    fn webview_surface_report_crash_transitions_to_crashed() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.report_crash();

        assert_eq!(surface.state(), &WebViewState::Crashed);
        assert_eq!(surface.status_message, "Page crashed");
    }

    #[test]
    fn webview_surface_recover_from_crash_transitions_to_loading() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.report_crash();
        surface.recover_from_crash();

        assert_eq!(surface.state(), &WebViewState::Loading);
        assert_eq!(surface.status_message, "Reloading...");
    }

    #[test]
    fn webview_surface_set_state_updates_status_message() {
        let mut surface = WebViewSurface::new("https://example.com");

        surface.set_state(WebViewState::Loading);
        assert_eq!(surface.status_message, "Loading...");
        assert_eq!(surface.state(), &WebViewState::Loading);

        surface.set_state(WebViewState::Ready);
        assert_eq!(surface.status_message, "Ready");
        assert_eq!(surface.state(), &WebViewState::Ready);

        surface.set_state(WebViewState::Error("Test error".to_string()));
        assert_eq!(surface.status_message, "Error: Test error");
        assert_eq!(surface.state(), &WebViewState::Error("Test error".to_string()));
    }

    #[test]
    fn webview_crashed_state_matches_pattern() {
        let crashed = WebViewState::Crashed;
        let loading = WebViewState::Loading;

        assert_eq!(crashed, WebViewState::Crashed);
        assert_ne!(crashed, loading);
    }

    #[test]
    fn webview_surface_bridge_operations() {
        let mut surface = WebViewSurface::new("https://example.com");

        // Initially no bridge
        assert!(surface.bridge().is_none());

        // Set bridge
        let router = BridgeRouter::new();
        surface.set_bridge(router);

        // Now has bridge
        assert!(surface.bridge().is_some());
    }

    #[test]
    fn webview_state_error_variants_distinct() {
        let error1 = WebViewState::Error("Network error".to_string());
        let error2 = WebViewState::Error("Parse error".to_string());
        let ready = WebViewState::Ready;

        assert_ne!(error1, error2);
        assert_ne!(error1, ready);
        assert_ne!(error2, ready);
    }
}
