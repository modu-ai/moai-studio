// Wry-based WebView backend implementation for SPEC-V3-007
//
// This module implements WebViewBackend trait using wry (Tauri's webview library).
// Wry provides cross-platform WebView embedding using native webview controls:
// - macOS: WKWebView (WebKit)
// - Windows: WebView2 (Edge Chromium)
// - Linux: WebKit2GTK

// wry re-exports raw_window_handle and dpi crate
use wry::{raw_window_handle, WebView, WebViewBuilder, Rect};

use super::WebViewBackend;

/// Wry-based WebView backend implementation
///
/// Wraps a wry::WebView and provides the WebViewBackend trait interface.
/// The WebView is created as a child window inside a parent GPUI window.
pub struct WryBackend {
    /// The underlying wry WebView
    webview: WebView,
}

impl WryBackend {
    /// Create a new WebView as a child of the given parent window
    ///
    /// # Arguments
    /// * `parent_window` - A reference to a type implementing HasWindowHandle (e.g., GPUI's PlatformWindow)
    /// * `bounds` - The position and size of the WebView within the parent window
    ///
    /// # Platform Compatibility
    /// - macOS: Fully supported via WKWebView
    /// - Windows: Fully supported via WebView2 (requires Edge Chromium)
    /// - Linux: Supported on X11 only (requires WebKit2GTK). Use GTK-specific APIs for Wayland.
    ///
    /// # Example
    /// ```no_run
    /// use wry::{dpi::LogicalPosition, dpi::LogicalSize, raw_window_handle::HasWindowHandle, Rect};
    /// use moai_studio_ui::web::WryBackend;
    ///
    /// // Assume we have a GPUI window accessible via window.platform_window()
    /// let bounds = Rect {
    ///     position: LogicalPosition::new(0, 0).into(),
    ///     size: LogicalSize::new(800, 600).into(),
    /// };
    /// // Note: This is a compilation test - parent_window would be a GPUI Window
    /// // that implements HasWindowHandle, which is verified by the generic constraint
    /// ```
    pub fn new_as_child<W>(parent_window: &W, bounds: Rect) -> Result<Self, Box<dyn std::error::Error>>
    where
        W: raw_window_handle::HasWindowHandle,
    {
        // @MX:ANCHOR - Critical integration point between GPUI and wry
        // @MX:REASON - This is the single point where GPUI and wry touch.
        // The parent_window must implement HasWindowHandle trait, which GPUI's PlatformWindow does.
        // This allows wry to access the native window handle (NSView on macOS, HWND on Windows).

        let webview = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_url("about:blank")
            .build_as_child(parent_window)?;

        Ok(Self { webview })
    }
}

impl WebViewBackend for WryBackend {
    fn navigate(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.webview.load_url(url);
        Ok(())
    }

    fn eval(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }

    fn current_url(&self) -> Option<String> {
        // Wry doesn't expose a direct current_url() method in the public API
        // This would need to be implemented via JavaScript bridge or custom protocol
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_wry_backend_compile() {
        // This test verifies that WryBackend can be compiled successfully
        // Actual integration tests require a running GPUI window context
    }
}
