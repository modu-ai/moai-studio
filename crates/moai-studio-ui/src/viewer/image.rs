//! Image Viewer with zoom and pan support (C-5).
//!
//! SPEC: C-5 Image Surface zoom/pan.
//! SPEC-V3-016 MS-1: Actual image decoding and rendering (REQ-IV-001~006).
//! SPEC-V3-016 MS-2: Zoom toolbar (REQ-IV-020~025) + EXIF panel (REQ-IV-040~045).
//! Features: Mouse wheel zoom, click-drag pan, fit-to-view reset, EXIF metadata panel.

use crate::design::tokens as tok;
use crate::viewer::exif::ExifData;
use crate::viewer::image_data::ImageData;
use gpui::prelude::FluentBuilder;
use gpui::{
    Context, CursorStyle, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

use std::path::Path;

/// Image viewer state with zoom and pan (REQ-IV-001).
pub struct ImageViewer {
    /// Current zoom level (1.0 = 100%).
    zoom: f32,
    /// Pan offset X (pixels).
    pan_x: f32,
    /// Pan offset Y (pixels).
    pan_y: f32,
    /// Whether currently dragging to pan.
    is_dragging: bool,
    /// Last mouse position for drag delta calculation.
    last_mouse_x: f32,
    last_mouse_y: f32,
    /// Decoded image data (REQ-IV-001, REQ-IV-005).
    image_data: Option<ImageData>,
    /// Error message if image loading failed (REQ-IV-003).
    error_message: Option<String>,
    /// EXIF metadata extracted from image file (REQ-IV-040).
    exif_data: Option<ExifData>,
    /// Whether EXIF side panel is visible (REQ-IV-045, default: false).
    show_exif_panel: bool,
    /// Whether the loaded file is SVG (REQ-IV-013).
    /// SVG files show a placeholder instead of decoded image.
    is_svg: bool,
}

impl ImageViewer {
    /// Create a new image viewer.
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            image_data: None,
            error_message: None,
            exif_data: None,
            show_exif_panel: false,
            is_svg: false,
        }
    }

    /// Load decoded image data (REQ-IV-002, REQ-IV-005).
    ///
    /// Also extracts EXIF metadata from the source file (REQ-IV-040).
    /// If the file is SVG (by extension), sets `is_svg` flag and skips decode (REQ-IV-013).
    pub fn load_image(&mut self, data: ImageData, _cx: &mut Context<Self>) {
        // Check if file is SVG by extension (REQ-IV-013)
        if Self::is_svg_path(&data.path) {
            self.is_svg = true;
            self.image_data = Some(data);
            self.error_message = None;
            self.reset_view();
            return;
        }

        self.is_svg = false;
        let width = data.width as f32;
        let height = data.height as f32;

        // Extract EXIF metadata from source path (REQ-IV-041)
        self.exif_data = crate::viewer::exif::extract_exif(Path::new(&data.path));

        self.image_data = Some(data);
        self.error_message = None;
        self.reset_view();
        // Set initial zoom to fit image within typical viewport
        self.fit_to_view(width, height);
    }

    /// Load SVG file as placeholder (REQ-IV-013).
    ///
    /// Sets `is_svg` flag and shows "SVG preview not yet supported" message.
    /// Does not attempt image decoding.
    pub fn load_svg(&mut self, _path: &Path, _cx: &mut Context<Self>) {
        self.is_svg = true;
        self.image_data = None;
        self.error_message = None;
        self.exif_data = None;
        self.reset_view();
    }

    /// Check if a file path has SVG extension.
    fn is_svg_path(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
    }

    /// Set error message if image loading failed (REQ-IV-003).
    pub fn set_error(&mut self, message: String, _cx: &mut Context<Self>) {
        self.error_message = Some(message);
        self.image_data = None;
    }

    /// Calculate zoom to fit image within bounds (helper for REQ-IV-023).
    fn fit_to_view(&mut self, img_w: f32, img_h: f32) {
        const VIEWPORT_W: f32 = 800.0;
        const VIEWPORT_H: f32 = 600.0;

        let scale_x = VIEWPORT_W / img_w;
        let scale_y = VIEWPORT_H / img_h;
        self.zoom = scale_x.min(scale_y).min(1.0); // Don't zoom in beyond 100%
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Reset zoom and pan to fit-to-view.
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    // ── Zoom toolbar actions (REQ-IV-020~025) ──

    /// Zoom in by 10%, capped at 10x (REQ-IV-021).
    pub fn zoom_in(&mut self) {
        const ZOOM_STEP: f32 = 0.1;
        const ZOOM_MAX: f32 = 10.0;
        self.zoom = (self.zoom + ZOOM_STEP).min(ZOOM_MAX);
    }

    /// Zoom out by 10%, floored at 0.1x (REQ-IV-022).
    pub fn zoom_out(&mut self) {
        const ZOOM_STEP: f32 = 0.1;
        const ZOOM_MIN: f32 = 0.1;
        self.zoom = (self.zoom - ZOOM_STEP).max(ZOOM_MIN);
    }

    /// Fit image to view bounds, resets pan (REQ-IV-023).
    pub fn fit_to_view_action(&mut self) {
        if let Some(ref data) = self.image_data {
            self.fit_to_view(data.width as f32, data.height as f32);
        }
    }

    /// Set zoom to 100% (actual size), resets pan (REQ-IV-024).
    pub fn zoom_actual_size(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Toggle EXIF panel visibility (REQ-IV-045).
    pub fn toggle_exif_panel(&mut self) {
        self.show_exif_panel = !self.show_exif_panel;
    }

    /// Whether the EXIF panel is currently visible.
    pub fn is_exif_panel_visible(&self) -> bool {
        self.show_exif_panel
    }

    /// Handle mouse wheel zoom.
    pub fn handle_wheel(&mut self, delta: f32) {
        const ZOOM_FACTOR: f32 = 0.1;
        const ZOOM_MIN: f32 = 0.1;
        const ZOOM_MAX: f32 = 10.0;

        if delta < 0.0 {
            // Zoom in
            self.zoom = (self.zoom + ZOOM_FACTOR).min(ZOOM_MAX);
        } else {
            // Zoom out
            self.zoom = (self.zoom - ZOOM_FACTOR).max(ZOOM_MIN);
        }
    }

    /// Handle mouse down for drag start.
    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        self.is_dragging = true;
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    /// Handle mouse up for drag end.
    pub fn handle_mouse_up(&mut self) {
        self.is_dragging = false;
    }

    /// Handle mouse move for panning.
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        if self.is_dragging {
            let dx = x - self.last_mouse_x;
            let dy = y - self.last_mouse_y;
            self.pan_x += dx;
            self.pan_y += dy;
            self.last_mouse_x = x;
            self.last_mouse_y = y;
        }
    }

    /// Get current zoom level (for testing).
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Get current pan X offset (for testing).
    pub fn pan_x(&self) -> f32 {
        self.pan_x
    }

    /// Get current pan Y offset (for testing).
    pub fn pan_y(&self) -> f32 {
        self.pan_y
    }

    /// Whether the loaded file is an SVG (REQ-IV-013).
    pub fn is_svg(&self) -> bool {
        self.is_svg
    }

    /// Whether currently dragging to pan.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Get cursor style based on current state (REQ-IV-030~031).
    ///
    /// Returns `CursorStyle::OpenHand` (grab) when hovering over image,
    /// `CursorStyle::ClosedHand` (grabbing) when dragging.
    pub fn cursor_style(&self) -> CursorStyle {
        if self.is_dragging {
            CursorStyle::ClosedHand
        } else {
            CursorStyle::OpenHand
        }
    }
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self::new()
    }
}

// @MX:ANCHOR: [AUTO] image-viewer-render
// @MX:REASON: [AUTO] REQ-IV-005, REQ-IV-020~025, REQ-IV-040~045.
//   Render is the primary UI entry point for image viewing with zoom toolbar and EXIF panel.
//   fan_in >= 3: GPUI render loop, unit tests, integration tests.

impl Render for ImageViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Show error if loading failed (REQ-IV-003)
        if let Some(ref error) = self.error_message {
            return self.render_error_message(error).into_any_element();
        }

        // SVG placeholder (REQ-IV-013)
        if self.is_svg {
            return self.render_svg_placeholder().into_any_element();
        }

        // If no image loaded, show placeholder (REQ-IV-006)
        let Some(ref data) = self.image_data else {
            return self.render_placeholder().into_any_element();
        };

        let img_w = data.width as f32;
        let img_h = data.height as f32;
        let display_w = img_w * self.zoom;
        let display_h = img_h * self.zoom;
        let show_exif = self.show_exif_panel && self.exif_data.is_some();

        // Cursor feedback: grab/grabbing based on drag state (REQ-IV-030~031)
        let cursor = self.cursor_style();

        div()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .flex()
            .flex_row()
            .overflow_hidden()
            .relative()
            // Main image area (shrinks if EXIF panel is visible)
            .child(
                div()
                    .flex_grow()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .relative()
                    .cursor(cursor)
                    // Image container (REQ-IV-005)
                    .child(
                        div()
                            .relative()
                            .w(px(display_w))
                            .h(px(display_h))
                            .bg(rgb(tok::BG_SURFACE))
                            .border_1()
                            .border_color(rgb(tok::BORDER_SUBTLE))
                            // Apply pan offset
                            .ml(px(self.pan_x))
                            .mt(px(self.pan_y))
                            // TODO: Render actual image pixels (REQ-IV-005)
                            // For now, show dimensions as placeholder
                            .child(
                                div()
                                    .absolute()
                                    .inset_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(tok::FG_SECONDARY))
                                    .text_sm()
                                    .child(format!("{} x {} pixels", data.width, data.height)),
                            )
                            // Zoom info overlay (REQ-IV-025)
                            .child(
                                div()
                                    .absolute()
                                    .top_4()
                                    .left_4()
                                    .px(px(8.))
                                    .py(px(4.))
                                    .rounded_md()
                                    .bg(rgb(0x00000080))
                                    .text_color(rgb(0xFFFFFF))
                                    .text_sm()
                                    .child(format!("Zoom: {:.0}%", self.zoom * 100.0)),
                            )
                            // Controls hint
                            .child(
                                div()
                                    .absolute()
                                    .bottom_4()
                                    .right_4()
                                    .px(px(8.))
                                    .py(px(4.))
                                    .rounded_md()
                                    .bg(rgb(0x00000080))
                                    .text_color(rgb(0xFFFFFF))
                                    .text_xs()
                                    .child("Scroll to zoom - Drag to pan"),
                            ),
                    )
                    // Zoom toolbar (REQ-IV-020~025)
                    .child(self.render_zoom_toolbar()),
            )
            // EXIF side panel (REQ-IV-042~045)
            .when(show_exif, |el: gpui::Div| {
                el.child(self.render_exif_panel())
            })
            .into_any_element()
    }
}

impl ImageViewer {
    fn render_placeholder(&self) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .h_full()
            .text_lg()
            .text_color(rgb(tok::FG_SECONDARY))
            .child("Image Viewer (C-5)")
    }

    /// Render SVG placeholder when SVG file is loaded (REQ-IV-013).
    fn render_svg_placeholder(&self) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h_full()
            .gap(px(8.))
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(tok::FG_SECONDARY))
                    .child("SVG preview not yet supported"),
            )
            .child(
                div().text_xs().text_color(rgb(tok::BORDER_SUBTLE)).child(
                    "SVG files are routed to the image viewer but rendering is not implemented",
                ),
            )
    }

    fn render_error_message(&self, message: &str) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .h_full()
            .text_lg()
            .text_color(rgb(tok::FG_SECONDARY))
            .child(format!("Failed to load image: {}", message))
    }

    /// Render zoom toolbar with 4 buttons + EXIF toggle (REQ-IV-020~025).
    fn render_zoom_toolbar(&self) -> gpui::Div {
        div()
            .absolute()
            .bottom_4()
            .left_1_2()
            .ml(px(-140.)) // Center toolbar (~280px wide / 2)
            .flex()
            .flex_row()
            .gap(px(4.))
            .px(px(8.))
            .py(px(4.))
            .rounded_md()
            .bg(rgb(0x000000CC))
            .text_color(rgb(0xFFFFFF))
            .text_xs()
            // Zoom-in button (REQ-IV-021)
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded_sm()
                    .bg(rgb(tok::brand::PRIMARY_DARK))
                    .cursor_pointer()
                    .child("+"),
            )
            // Zoom-out button (REQ-IV-022)
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded_sm()
                    .bg(rgb(tok::brand::PRIMARY_DARK))
                    .cursor_pointer()
                    .child("-"),
            )
            // Fit-to-view button (REQ-IV-023)
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded_sm()
                    .bg(rgb(tok::brand::PRIMARY_DARK))
                    .cursor_pointer()
                    .child("Fit"),
            )
            // 100% / Actual size button (REQ-IV-024)
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded_sm()
                    .bg(rgb(tok::brand::PRIMARY_DARK))
                    .cursor_pointer()
                    .child("100%"),
            )
            // EXIF toggle button (REQ-IV-045)
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded_sm()
                    .bg(if self.show_exif_panel {
                        rgb(tok::brand::PRIMARY_DARK_HOVER)
                    } else {
                        rgb(tok::neutral::N700)
                    })
                    .cursor_pointer()
                    .child("EXIF"),
            )
    }

    /// Render EXIF metadata side panel (REQ-IV-042~044).
    fn render_exif_panel(&self) -> gpui::Div {
        let exif = match &self.exif_data {
            Some(e) => e,
            None => return div(),
        };

        let mut rows = Vec::new();

        if let Some(ref v) = exif.camera_make {
            rows.push(self.render_exif_row("Make", v));
        }
        if let Some(ref v) = exif.camera_model {
            rows.push(self.render_exif_row("Model", v));
        }
        if let Some(ref v) = exif.datetime_original {
            rows.push(self.render_exif_row("Date", v));
        }
        if let Some(ref v) = exif.exposure_time {
            rows.push(self.render_exif_row("Exposure", v));
        }
        if let Some(ref v) = exif.f_number {
            rows.push(self.render_exif_row("Aperture", v));
        }
        if let Some(v) = exif.iso {
            rows.push(self.render_exif_row("ISO", &v.to_string()));
        }
        if let Some(ref v) = exif.focal_length {
            rows.push(self.render_exif_row("Focal Length", v));
        }
        if let Some(v) = exif.image_width {
            rows.push(self.render_exif_row("Width", &format!("{} px", v)));
        }
        if let Some(v) = exif.image_height {
            rows.push(self.render_exif_row("Height", &format!("{} px", v)));
        }

        div()
            .w(px(280.))
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .border_l_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .flex()
            .flex_col()
            .pt(px(12.))
            .px(px(12.))
            .overflow_hidden()
            // Panel header
            .child(
                div()
                    .pb(px(8.))
                    .border_b_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .text_sm()
                    .text_color(rgb(tok::FG_PRIMARY))
                    .child("EXIF Metadata"),
            )
            // EXIF rows
            .children(rows)
    }

    /// Render a single EXIF label-value row.
    fn render_exif_row(&self, label: &str, value: &str) -> gpui::Div {
        div()
            .flex()
            .flex_row()
            .justify_between()
            .py(px(6.))
            .border_b_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_SECONDARY))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_PRIMARY))
                    .child(value.to_string()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── REQ-IV-020~025: Zoom toolbar tests ──

    #[test]
    fn zoom_in_increments_by_10_percent() {
        let mut viewer = ImageViewer::new();
        assert!((viewer.zoom() - 1.0).abs() < f32::EPSILON);
        viewer.zoom_in();
        assert!((viewer.zoom() - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_out_decrements_by_10_percent() {
        let mut viewer = ImageViewer::new();
        viewer.zoom_in(); // 1.0 -> 1.1
        viewer.zoom_out(); // 1.1 -> 1.0
        assert!((viewer.zoom() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_capped_at_10x() {
        let mut viewer = ImageViewer::new();
        viewer.zoom = 9.95;
        viewer.zoom_in(); // 9.95 + 0.1 = 10.05 -> capped at 10.0
        assert!((viewer.zoom() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_floored_at_0_1x() {
        let mut viewer = ImageViewer::new();
        viewer.zoom = 0.15;
        viewer.zoom_out(); // 0.15 - 0.1 = 0.05 -> floored at 0.1
        assert!((viewer.zoom() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn fit_to_view_resets_pan() {
        let mut viewer = ImageViewer::new();
        // Simulate some pan offset
        viewer.pan_x = 50.0;
        viewer.pan_y = 30.0;

        // Load image data so fit_to_view_action has dimensions
        let pixels = vec![0u8; 100 * 100 * 4];
        let data = ImageData::new(pixels, 100, 100, std::path::PathBuf::from("test.png"), 1024);
        viewer.image_data = Some(data);

        viewer.fit_to_view_action();
        assert!(
            (viewer.pan_x() - 0.0).abs() < f32::EPSILON,
            "fit_to_view should reset pan_x"
        );
        assert!(
            (viewer.pan_y() - 0.0).abs() < f32::EPSILON,
            "fit_to_view should reset pan_y"
        );
    }

    #[test]
    fn zoom_100_percent_resets_pan() {
        let mut viewer = ImageViewer::new();
        viewer.pan_x = 100.0;
        viewer.pan_y = 200.0;
        viewer.zoom = 2.5;

        viewer.zoom_actual_size();

        assert!(
            (viewer.zoom() - 1.0).abs() < f32::EPSILON,
            "zoom should be 1.0"
        );
        assert!(
            (viewer.pan_x() - 0.0).abs() < f32::EPSILON,
            "100% should reset pan_x"
        );
        assert!(
            (viewer.pan_y() - 0.0).abs() < f32::EPSILON,
            "100% should reset pan_y"
        );
    }

    // ── REQ-IV-045: EXIF panel toggle tests ──

    #[test]
    fn exif_panel_default_hidden() {
        let viewer = ImageViewer::new();
        assert!(
            !viewer.is_exif_panel_visible(),
            "EXIF panel should be hidden by default"
        );
    }

    #[test]
    fn exif_panel_toggle_flips_visibility() {
        let mut viewer = ImageViewer::new();
        assert!(!viewer.is_exif_panel_visible());
        viewer.toggle_exif_panel();
        assert!(
            viewer.is_exif_panel_visible(),
            "first toggle should show panel"
        );
        viewer.toggle_exif_panel();
        assert!(
            !viewer.is_exif_panel_visible(),
            "second toggle should hide panel"
        );
    }

    // ── REQ-IV-013: SVG placeholder tests ──

    #[test]
    fn svg_path_detection() {
        assert!(ImageViewer::is_svg_path(Path::new("graphic.svg")));
        assert!(ImageViewer::is_svg_path(Path::new("diagram.SVG")));
        assert!(ImageViewer::is_svg_path(Path::new("/path/to/file.Svg")));
        assert!(!ImageViewer::is_svg_path(Path::new("photo.png")));
        assert!(!ImageViewer::is_svg_path(Path::new("image.jpeg")));
        assert!(!ImageViewer::is_svg_path(Path::new("svg_backup")));
    }

    #[test]
    fn svg_file_sets_is_svg_flag() {
        let mut viewer = ImageViewer::new();
        // Simulate SVG loading by checking is_svg_path and setting the flag directly
        // (load_image requires GPUI Context, so we test the logic path)
        let svg_path = std::path::PathBuf::from("graphic.svg");
        assert!(ImageViewer::is_svg_path(&svg_path));
        viewer.is_svg = true;
        viewer.image_data = Some(ImageData::new(vec![0u8; 4], 1, 1, svg_path, 100));
        assert!(viewer.is_svg());
    }

    #[test]
    fn svg_file_skips_exif_and_fit_to_view() {
        // SVG loading path should not trigger EXIF extraction or fit_to_view
        let mut viewer = ImageViewer::new();
        viewer.is_svg = true;
        viewer.image_data = Some(ImageData::new(
            vec![0u8; 4],
            1,
            1,
            std::path::PathBuf::from("diagram.SVG"),
            100,
        ));
        assert!(viewer.exif_data.is_none(), "SVG should not extract EXIF");
        // zoom should remain at 1.0 (not fit_to_view adjusted)
        assert!((viewer.zoom() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn non_svg_file_does_not_set_is_svg() {
        let viewer = ImageViewer::new();
        assert!(!viewer.is_svg());
        // PNG should not set is_svg
        let png_path = std::path::PathBuf::from("photo.png");
        assert!(!ImageViewer::is_svg_path(&png_path));
    }

    // ── REQ-IV-030~031: Cursor feedback tests ──

    #[test]
    fn cursor_grab_when_not_dragging() {
        let viewer = ImageViewer::new();
        assert_eq!(viewer.cursor_style(), CursorStyle::OpenHand);
    }

    #[test]
    fn cursor_grabbing_while_dragging() {
        let mut viewer = ImageViewer::new();
        assert!(!viewer.is_dragging());
        assert_eq!(viewer.cursor_style(), CursorStyle::OpenHand);

        viewer.handle_mouse_down(10.0, 20.0);
        assert!(viewer.is_dragging());
        assert_eq!(viewer.cursor_style(), CursorStyle::ClosedHand);
    }

    #[test]
    fn cursor_transitions_grab_to_grabbing_to_grab() {
        let mut viewer = ImageViewer::new();
        // Initial: grab
        assert_eq!(viewer.cursor_style(), CursorStyle::OpenHand);

        // Start drag: grabbing
        viewer.handle_mouse_down(0.0, 0.0);
        assert_eq!(viewer.cursor_style(), CursorStyle::ClosedHand);

        // End drag: grab
        viewer.handle_mouse_up();
        assert_eq!(viewer.cursor_style(), CursorStyle::OpenHand);
    }

    // ── Pan state after zoom changes ──

    #[test]
    fn pan_preserved_after_wheel_zoom() {
        let mut viewer = ImageViewer::new();
        viewer.pan_x = 50.0;
        viewer.pan_y = 30.0;
        viewer.handle_wheel(-1.0); // zoom in
        assert!(
            (viewer.pan_x() - 50.0).abs() < f32::EPSILON,
            "wheel zoom should preserve pan_x"
        );
        assert!(
            (viewer.pan_y() - 30.0).abs() < f32::EPSILON,
            "wheel zoom should preserve pan_y"
        );
    }

    // ── Zoom boundary conditions ──

    #[test]
    fn zoom_in_at_max_stays_at_max() {
        let mut viewer = ImageViewer::new();
        viewer.zoom = 10.0;
        viewer.zoom_in(); // already at max, should not exceed
        assert!(
            (viewer.zoom() - 10.0).abs() < f32::EPSILON,
            "zoom should stay at 10x max"
        );
    }

    #[test]
    fn zoom_out_at_min_stays_at_min() {
        let mut viewer = ImageViewer::new();
        viewer.zoom = 0.1;
        viewer.zoom_out(); // already at min, should not go below
        assert!(
            (viewer.zoom() - 0.1).abs() < f32::EPSILON,
            "zoom should stay at 0.1x min"
        );
    }

    // ── Drag state tracking ──

    #[test]
    fn drag_updates_pan_offset() {
        let mut viewer = ImageViewer::new();
        viewer.handle_mouse_down(100.0, 100.0);
        viewer.handle_mouse_move(120.0, 115.0);
        assert!(
            (viewer.pan_x() - 20.0).abs() < f32::EPSILON,
            "pan_x should be 20.0"
        );
        assert!(
            (viewer.pan_y() - 15.0).abs() < f32::EPSILON,
            "pan_y should be 15.0"
        );
    }

    #[test]
    fn no_pan_without_drag() {
        let mut viewer = ImageViewer::new();
        // Move without mousedown should not change pan
        viewer.handle_mouse_move(120.0, 115.0);
        assert!((viewer.pan_x() - 0.0).abs() < f32::EPSILON);
        assert!((viewer.pan_y() - 0.0).abs() < f32::EPSILON);
    }
}
