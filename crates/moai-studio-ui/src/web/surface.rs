// WebViewSurface GPUI Entity for SPEC-V3-007 MS-1
//
// This module implements the GPUI Entity that renders the web browser chrome
// (URL bar + status bar + webview placeholder area).
//
// REQ-WB-001: WebViewSurface is a GPUI Entity with impl Render
// REQ-WB-005: If backend unavailable, render placeholder (no panic)

use crate::design::tokens as tok;
use gpui::{
    Context, Div, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px,
    rgb,
};

/// WebViewSurface GPUI Entity
///
/// Renders a web browser chrome with URL bar, navigation buttons,
/// status bar, and webview content area. Gracefully degrades when
/// wry backend is not available (REQ-WB-005).
///
/// # Fields
/// * `url_bar_text` - Current URL bar content
/// * `status_message` - Status bar text (e.g., "Loading...", "Ready")
/// * `backend_available` - Whether wry backend is available
/// * `history` - Navigation history (max 100 entries)
/// * `history_index` - Current position in history
pub struct WebViewSurface {
    /// Current URL bar content
    url_bar_text: String,
    /// Status bar text showing loading state
    status_message: String,
    /// Whether wry backend is available (feature flag check)
    backend_available: bool,
    /// Navigation history (simple Vec for now, max 100)
    history: Vec<String>,
    /// Current position in history (None = no history)
    history_index: Option<usize>,
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
            history: vec![url_text],
            history_index: Some(0),
        }
    }

    /// Navigate to a new URL
    ///
    /// Updates URL bar, adds to history, and sets status to "Loading..."
    pub fn navigate(&mut self, url: impl Into<String>) {
        let url = url.into();
        self.url_bar_text = url.clone();
        self.status_message = "Loading...".to_string();

        // Add to history (max 100 entries)
        if self.history.len() >= 100 {
            self.history.remove(0);
        }
        self.history.push(url.clone());
        self.history_index = Some(self.history.len() - 1);
    }

    /// Go back in history
    ///
    /// Returns true if successful, false if already at beginning
    pub fn go_back(&mut self) -> bool {
        match self.history_index {
            Some(0) => false, // Already at beginning
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                if let Some(url) = self.history.get(idx - 1) {
                    self.url_bar_text = url.clone();
                    self.status_message = "Loading...".to_string();
                }
                true
            }
            _ => false,
        }
    }

    /// Go forward in history
    ///
    /// Returns true if successful, false if already at end
    pub fn go_forward(&mut self) -> bool {
        match self.history_index {
            Some(idx) if idx < self.history.len().saturating_sub(1) => {
                self.history_index = Some(idx + 1);
                if let Some(url) = self.history.get(idx + 1) {
                    self.url_bar_text = url.clone();
                    self.status_message = "Loading...".to_string();
                }
                true
            }
            _ => false,
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
                    // Back button
                    .child(nav_button("←", "Back"))
                    // Forward button
                    .child(nav_button("→", "Forward"))
                    // Reload button
                    .child(nav_button("⟳", "Reload"))
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
            // Content area: WebView placeholder (MS-1 - actual WebView integration in MS-2)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .justify_center()
                    .items_center()
                    .bg(rgb(tok::BG_PANEL))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_MUTED))
                            .child("WebView will render here (MS-2 integration)"),
                    ),
            )
            // Bottom status bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_3()
                    .py_1()
                    .bg(rgb(tok::BG_SURFACE))
                    .border_t_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .child(self.status_message.clone()),
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

/// Navigation button styling helper
fn nav_button(label: &'static str, _tooltip: &'static str) -> Div {
    use gpui::StyleRefinement;
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(28.))
        .rounded_md()
        .text_sm()
        .text_color(rgb(tok::FG_SECONDARY))
        .hover(|s: StyleRefinement| s.bg(rgb(tok::BG_ELEVATED)))
        .cursor_pointer()
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
        assert_eq!(surface.history_index, Some(0));
    }

    #[test]
    fn webview_surface_navigate_updates_url_and_history() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");

        assert_eq!(surface.url_bar_text, "https://modu.ai");
        assert_eq!(surface.status_message, "Loading...");
        assert_eq!(surface.history.len(), 2);
        assert_eq!(surface.history_index, Some(1));
        assert_eq!(surface.history[1], "https://modu.ai");
    }

    #[test]
    fn webview_surface_go_back_works() {
        let mut surface = WebViewSurface::new("https://example.com");
        surface.navigate("https://modu.ai");
        surface.navigate("https://github.com");

        let success = surface.go_back();
        assert!(success);
        assert_eq!(surface.url_bar_text, "https://modu.ai");
        assert_eq!(surface.history_index, Some(1));
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
        assert_eq!(surface.history_index, Some(1));
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
        // First entry should be removed
        assert_eq!(surface.history[0], "https://example.com/1");
    }

    #[test]
    fn webview_surface_backend_available_reflects_feature_flag() {
        let surface = WebViewSurface::new("https://example.com");
        // backend_available should match cfg!(feature = "web")
        assert_eq!(surface.backend_available, cfg!(feature = "web"));
    }
}
