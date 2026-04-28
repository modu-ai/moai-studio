// WebView backend abstraction layer for SPEC-V3-007
//
// This module provides platform-agnostic WebView embedding capabilities
// using wry (Tauri's webview library) as the backend.
//
// # Spike 0 Validation (SPEC-V3-007 MS-1)
// - GPUI Window → PlatformWindow trait → HasWindowHandle (raw-window-handle)
// - wry WebViewBuilder::build_as_child() accepts any HasWindowHandle implementer
// - Compatibility: FULLY COMPATIBLE via raw_window_handle::HasWindowHandle trait

/// WebView backend trait abstracting over platform-specific WebView implementations
///
/// This trait allows different WebView backends (wry, webview2, WKWebView directly)
/// to be used interchangeably. The primary implementation is WryBackend.
///
/// # Thread Safety
/// WebView implementations are typically tied to the main/UI thread and are not
/// Send + Sync. This trait does not require Send + Sync bounds to accommodate
/// platform-specific WebView implementations.
pub trait WebViewBackend {
    /// Navigate the WebView to the specified URL
    fn navigate(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Execute JavaScript code in the WebView
    fn eval(&mut self, script: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the current URL of the WebView
    fn current_url(&self) -> Option<String>;
}

#[cfg(feature = "web")]
pub mod surface;

#[cfg(feature = "web")]
pub mod wry_backend;

#[cfg(feature = "web")]
pub use surface::WebViewSurface;

#[cfg(feature = "web")]
pub use wry_backend::WryBackend;
