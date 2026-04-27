//! Image Viewer with zoom and pan support (C-5).
//!
//! SPEC: C-5 Image Surface zoom/pan.
//! Features: Mouse wheel zoom, click-drag pan, fit-to-view reset.

use gpui::{div, px, rgb, Context, IntoElement, ParentElement, Render, Styled, Window};
use crate::design::tokens as tok;

/// Image viewer state with zoom and pan.
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
    /// Image dimensions (width, height).
    image_size: Option<(f32, f32)>,
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
            image_size: None,
        }
    }

    /// Set image dimensions.
    pub fn set_image_size(&mut self, width: f32, height: f32) {
        self.image_size = Some((width, height));
    }

    /// Reset zoom and pan to fit-to-view.
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
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
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ImageViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // If no image loaded, show placeholder
        if self.image_size.is_none() {
            return self.render_placeholder().into_any_element();
        }

        let (img_w, img_h) = self.image_size.unwrap();
        let display_w = img_w * self.zoom;
        let display_h = img_h * self.zoom;

        div()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .relative()
            // Image container
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
                    // Zoom info overlay
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
                            .child(format!("Zoom: {:.0}%", self.zoom * 100.0))
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
                            .child("Scroll to zoom • Drag to pan")
                    )
            )
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
}
