//! Sparkline history view for TRUST 5 quality dashboard (SPEC-V3-017 MS-3)
//!
//! Renders a line chart showing a single dimension's score trend over time.
//! Uses GPUI canvas for custom drawing of the sparkline.
//!
//! REQ-QD-020~021: History sparkline with time-series rendering.

use gpui::{
    Context, IntoElement, ParentElement, Pixels, Render, Styled, Window, canvas, div, point, px,
    rgb,
};
use moai_studio_agent::quality::QualityHistory;

use crate::design::tokens as tok;

/// Full dimension names in T, R, U, S, K order.
const DIMENSION_NAMES: [&str; 5] = ["Tested", "Readable", "Unified", "Secured", "Trackable"];

/// Default sparkline height in pixels.
const DEFAULT_SPARKLINE_HEIGHT: f32 = 80.0;

/// GPUI component that renders a sparkline chart for a single TRUST 5 dimension.
///
/// Displays the score history as a line chart with the newest snapshot on the right
/// and the oldest on the left. The Y axis spans [0.0, 1.0].
///
/// REQ-QD-020: Sparkline chart rendering.
/// REQ-QD-021: Dimension trend extraction.
pub struct HistoryView {
    /// Quality score history ring buffer.
    pub history: QualityHistory,
    /// Currently selected dimension index (0-4 for T, R, U, S, K).
    pub selected_dimension: usize,
    /// Sparkline height in pixels.
    pub height: f32,
}

impl HistoryView {
    /// Create a new HistoryView with an empty history and default dimension (Tested).
    pub fn new(history: QualityHistory) -> Self {
        Self {
            history,
            selected_dimension: 0,
            height: DEFAULT_SPARKLINE_HEIGHT,
        }
    }

    /// Select a dimension to display (0-4).
    pub fn select_dimension(&mut self, idx: usize) {
        self.selected_dimension = idx.min(4);
    }

    /// Get the label for a dimension index.
    pub fn dimension_label(idx: usize) -> &'static str {
        DIMENSION_NAMES[idx.min(4)]
    }

    /// Get the current score for the selected dimension from the latest snapshot.
    fn current_score(&self) -> f32 {
        self.history
            .latest()
            .map(|s| s.score.as_slice()[self.selected_dimension.min(4)])
            .unwrap_or(0.0)
    }

    /// Get the trend data for the selected dimension.
    fn trend(&self) -> Vec<f32> {
        self.history.dimension_trend(self.selected_dimension)
    }
}

impl Default for HistoryView {
    fn default() -> Self {
        Self::new(QualityHistory::new())
    }
}

/// Helper: extract f32 from Pixels.
#[inline]
fn pf(p: Pixels) -> f32 {
    f32::from(p)
}

/// Build a stroke path from line segments connecting points.
fn stroke_path_from_points(
    points: &[(f32, f32)],
    ox: f32,
    oy: f32,
    line_width: f32,
) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::stroke(px(line_width));
    if let Some(&(fx, fy)) = points.first() {
        builder.move_to(point(px(ox + fx), px(oy + fy)));
        for &(x, y) in &points[1..] {
            builder.line_to(point(px(ox + x), px(oy + y)));
        }
    }
    builder
        .build()
        .unwrap_or_else(|_| gpui::PathBuilder::stroke(px(line_width)).build().unwrap())
}

impl Render for HistoryView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let trend = self.trend();
        let current = self.current_score();
        let dim_label = Self::dimension_label(self.selected_dimension);
        let height = self.height;
        let padding: f32 = 16.0;
        let chart_h = height - padding;

        // Build line chart points: x is evenly spaced, y is score mapped to chart area.
        // Newest data point is on the right.
        let points: Vec<(f32, f32)> = if trend.len() >= 2 {
            let step = 260.0 / (trend.len() - 1).max(1) as f32;
            trend
                .iter()
                .enumerate()
                .map(|(i, &score)| {
                    let x = padding + i as f32 * step;
                    // Invert Y: score 1.0 at top (y=0), score 0.0 at bottom (y=chart_h)
                    let y = chart_h * (1.0 - score);
                    (x, y + padding / 2.0)
                })
                .collect()
        } else {
            vec![]
        };

        let dim_color = if current >= 0.75 {
            tok::semantic::SUCCESS
        } else {
            tok::semantic::DANGER
        };

        // Canvas for drawing the sparkline
        let sparkline_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let ox = pf(bounds.origin.x);
                let oy = pf(bounds.origin.y);

                if points.len() >= 2 {
                    // Draw the sparkline
                    let path = stroke_path_from_points(&points, ox, oy, 2.0);
                    window.paint_path(path, gpui::rgb(tok::ACCENT));
                }
            },
        )
        .size_full();

        // Container with header + canvas
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(4.))
            .child(
                // Header: dimension label + current score
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(8.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(dim_label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(dim_color))
                            .child(format!("{:.2}", current)),
                    ),
            )
            .child(
                // Sparkline canvas area
                div()
                    .relative()
                    .w_full()
                    .h(px(height))
                    .bg(rgb(tok::BG_PANEL))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .child(sparkline_canvas),
            )
    }
}

// ============================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::quality::{QualityHistory, QualitySnapshot, Trust5Score};

    /// REQ-QD-020: dimension_label returns correct names.
    #[test]
    fn history_view_dimension_label() {
        assert_eq!(HistoryView::dimension_label(0), "Tested");
        assert_eq!(HistoryView::dimension_label(1), "Readable");
        assert_eq!(HistoryView::dimension_label(2), "Unified");
        assert_eq!(HistoryView::dimension_label(3), "Secured");
        assert_eq!(HistoryView::dimension_label(4), "Trackable");
    }

    /// REQ-QD-021: Default selected dimension is 0 (Tested).
    #[test]
    fn history_view_default_selected_dimension() {
        let view = HistoryView::default();
        assert_eq!(view.selected_dimension, 0);
        assert_eq!(view.height, DEFAULT_SPARKLINE_HEIGHT);
    }

    /// select_dimension clamps to valid range.
    #[test]
    fn history_view_select_dimension_clamps() {
        let mut view = HistoryView::default();
        view.select_dimension(3);
        assert_eq!(view.selected_dimension, 3);
        view.select_dimension(10); // Should clamp to 4
        assert_eq!(view.selected_dimension, 4);
    }

    /// trend returns empty when no history.
    #[test]
    fn history_view_trend_empty() {
        let view = HistoryView::default();
        assert!(view.trend().is_empty());
    }

    /// current_score returns 0 when history is empty.
    #[test]
    fn history_view_current_score_no_history() {
        let view = HistoryView::default();
        assert_eq!(view.current_score(), 0.0);
    }

    /// current_score returns latest dimension score.
    #[test]
    fn history_view_current_score_with_data() {
        let mut history = QualityHistory::new();
        history.push(QualitySnapshot::new(Trust5Score::new_with_scores(
            0.8, 0.7, 0.6, 0.5, 0.4,
        )));
        let view = HistoryView::new(history);
        assert!((view.current_score() - 0.8).abs() < f32::EPSILON);
    }
}
