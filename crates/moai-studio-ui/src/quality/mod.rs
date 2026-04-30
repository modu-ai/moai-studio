//! TRUST 5 Quality Dashboard UI components (SPEC-V3-017 MS-2/MS-3)
//!
//! Provides GPUI views for the TRUST 5 quality dashboard:
//! - `RadarChartView`: 5-axis radar chart for dimension scores
//! - `QualityGateView`: Horizontal gate indicator bar
//! - `HistoryView`: Sparkline chart for dimension score trends
//! - `DimensionDetailView`: Metric detail table for a selected dimension
//! - `QualityDashboardView`: Combined container
//!
//! REQ-QD-010~014: Radar chart rendering.
//! REQ-QD-015~017: Quality gate display.
//! REQ-QD-018~021: Quality history ring buffer (sparkline).
//! REQ-QD-025~028: Dimension detail view.

pub mod dimension_detail_view;
pub mod history_view;
pub mod quality_gate_view;
pub mod radar_chart_view;

// Re-export main types
pub use dimension_detail_view::{DimensionDetailView, DimensionMetrics, MetricItem};
pub use history_view::HistoryView;
pub use quality_gate_view::{DimensionGate, GateStatus, QualityGateView};
pub use radar_chart_view::RadarChartView;

use gpui::{
    Context, IntoElement, ParentElement, Pixels, Render, Styled, Window, canvas, div, point,
    prelude::FluentBuilder, px, rgb,
};
use moai_studio_agent::quality::{QualityHistory, Trust5Score};

use crate::design::tokens as tok;

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
        builder.close();
    }
    builder
        .build()
        .unwrap_or_else(|_| gpui::PathBuilder::stroke(px(line_width)).build().unwrap())
}

/// Build a fill path from line segments connecting points.
fn fill_path_from_points(points: &[(f32, f32)], ox: f32, oy: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::fill();
    if let Some(&(fx, fy)) = points.first() {
        builder.move_to(point(px(ox + fx), px(oy + fy)));
        for &(x, y) in &points[1..] {
            builder.line_to(point(px(ox + x), px(oy + y)));
        }
        builder.close();
    }
    builder
        .build()
        .unwrap_or_else(|_| gpui::PathBuilder::fill().build().unwrap())
}

/// Build a stroke line between two points.
fn line_path(x1: f32, y1: f32, x2: f32, y2: f32, width: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::stroke(px(width));
    builder.move_to(point(px(x1), px(y1)));
    builder.line_to(point(px(x2), px(y2)));
    builder
        .build()
        .unwrap_or_else(|_| gpui::PathBuilder::stroke(px(width)).build().unwrap())
}

/// Build an approximate circle path (8-segment polygon, filled).
fn circle_path(cx: f32, cy: f32, r: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::fill();
    for j in 0..8 {
        let angle = (j as f32 / 8.0) * 2.0 * std::f32::consts::PI;
        let px_val = cx + r * angle.cos();
        let py_val = cy + r * angle.sin();
        if j == 0 {
            builder.move_to(point(px(px_val), px(py_val)));
        } else {
            builder.line_to(point(px(px_val), px(py_val)));
        }
    }
    builder.close();
    builder
        .build()
        .unwrap_or_else(|_| gpui::PathBuilder::fill().build().unwrap())
}

/// Combined container for the TRUST 5 quality dashboard.
///
/// Renders the radar chart, dimension detail, history sparkline, and gate
/// indicators together. Click on a radar axis to select a dimension, which
/// updates the history sparkline and dimension detail panel.
///
/// Layout (REQ-QD-022~024):
/// ```text
/// +--------------------------------------------+
/// | [Radar Chart]     | [Dimension Detail]     |
/// | (300x300)         | (metric table)         |
/// +--------------------------------------------+
/// | [History Sparkline]                        |
/// | (selected dimension trend)                 |
/// +--------------------------------------------+
/// | [Quality Gate Bar] (5 indicators)          |
/// +--------------------------------------------+
/// ```
pub struct QualityDashboardView {
    /// Radar chart component.
    pub radar: RadarChartView,
    /// Quality gate bar component.
    pub gate: QualityGateView,
    /// Quality score history for sparkline.
    pub history: QualityHistory,
    /// Currently selected dimension index (None = no selection).
    pub selected_dimension: Option<usize>,
}

impl QualityDashboardView {
    /// Create a new dashboard with default settings.
    pub fn new() -> Self {
        Self {
            radar: RadarChartView::new(),
            gate: QualityGateView::new(),
            history: QualityHistory::new(),
            selected_dimension: None,
        }
    }

    /// Create a dashboard with the given score and default threshold.
    pub fn with_score(score: Trust5Score) -> Self {
        Self {
            radar: RadarChartView::with_score(score),
            gate: QualityGateView::with_score_and_threshold(score, 0.75),
            history: QualityHistory::new(),
            selected_dimension: None,
        }
    }

    /// Update both views with a new score.
    pub fn set_score(&mut self, score: Trust5Score) {
        self.radar.set_score(score);
        self.gate.score = score;
    }

    /// Update threshold for both views.
    pub fn set_threshold(&mut self, threshold: f32) {
        self.radar.set_threshold(threshold);
        self.gate.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Select a dimension for detail view and sparkline.
    pub fn select_dimension(&mut self, idx: usize) {
        self.selected_dimension = Some(idx.min(4));
        self.radar.hovered_axis = self.selected_dimension;
    }

    /// Clear dimension selection.
    pub fn clear_selection(&mut self) {
        self.selected_dimension = None;
        self.radar.hovered_axis = None;
    }
}

impl Default for QualityDashboardView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for QualityDashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.radar.size;
        let center = size / 2.0;
        let radius = size / 2.0 - 30.0;
        let score = self.radar.score;
        let threshold = self.radar.threshold;
        let selected_dim = self.selected_dimension;

        // Precompute label positions for div overlays
        let label_data: Vec<(f32, f32, f32, u32)> = (0..5)
            .map(|i| {
                let (lx, ly) = radar_chart_view::label_position(center, center, radius, i);
                let dim_score = score.as_slice()[i];
                let label_color = if dim_score >= threshold {
                    tok::semantic::SUCCESS
                } else {
                    tok::semantic::DANGER
                };
                (lx, ly, dim_score, label_color)
            })
            .collect();

        // Canvas for drawing radar chart
        let chart_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let ox = pf(bounds.origin.x);
                let oy = pf(bounds.origin.y);

                // Axis lines from center to perimeter
                for i in 0..5 {
                    let (ex, ey) = radar_chart_view::axis_position(center, center, radius, i, 1.0);
                    let path = line_path(ox + center, oy + center, ox + ex, oy + ey, 1.0);
                    window.paint_path(path, gpui::rgba(tok::FG_MUTED | 0x4d000000));
                }

                // Threshold reference polygon
                let threshold_points: Vec<(f32, f32)> = (0..5)
                    .map(|i| radar_chart_view::axis_position(center, center, radius, i, threshold))
                    .collect();
                let threshold_path = stroke_path_from_points(&threshold_points, ox, oy, 1.0);
                window.paint_path(threshold_path, gpui::rgba(tok::FG_MUTED | 0x60000000));

                // Score polygon: fill
                let score_points =
                    radar_chart_view::polygon_vertices(&score, center, center, radius);
                let fill_p = fill_path_from_points(&score_points, ox, oy);
                window.paint_path(fill_p, gpui::rgba(tok::ACCENT | 0x33000000));

                // Score polygon: stroke
                let stroke_p = stroke_path_from_points(&score_points, ox, oy, 2.0);
                window.paint_path(stroke_p, gpui::rgb(tok::ACCENT));

                // Score dots
                for (i, &(sx, sy)) in score_points.iter().enumerate() {
                    let dim_score = score.as_slice()[i];
                    let dot_color = if dim_score >= threshold {
                        gpui::rgb(tok::semantic::SUCCESS)
                    } else {
                        gpui::rgb(tok::semantic::DANGER)
                    };
                    let path = circle_path(ox + sx, oy + sy, 4.0);
                    window.paint_path(path, dot_color);
                }

                // Center dot
                let center_dot = circle_path(ox + center, oy + center, 2.0);
                window.paint_path(center_dot, gpui::rgb(tok::FG_MUTED));
            },
        )
        .size_full();

        // Radar chart container
        let mut radar_chart = div()
            .relative()
            .w(px(size))
            .h(px(size))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .child(chart_canvas);

        for (i, &(lx, ly, dim_score, label_color)) in label_data.iter().enumerate() {
            let is_selected = selected_dim == Some(i);

            radar_chart = radar_chart.child(
                div()
                    .absolute()
                    .left(px(lx - 16.0))
                    .top(px(ly - 10.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .when(is_selected, |el| {
                        el.border_2()
                            .border_color(rgb(tok::ACCENT))
                            .rounded_md()
                            .px(px(2.))
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(label_color))
                            .child(radar_chart_view::DIMENSION_LABELS[i].to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", dim_score)),
                    ),
            );
        }

        // Dimension detail panel (rendered inline)
        let dim_names = ["Tested", "Readable", "Unified", "Secured", "Trackable"];
        let has_selection = selected_dim.is_some();
        let dim_header = if let Some(idx) = selected_dim {
            format!("{} Metrics", dim_names[idx])
        } else {
            "Dimension Metrics".to_string()
        };

        let mut detail_panel = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE));

        detail_panel = detail_panel.child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_PRIMARY))
                .child(dim_header),
        );

        if !has_selection {
            detail_panel = detail_panel.child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .py(px(16.))
                    .child("Select a dimension to view metrics"),
            );
        } else {
            // Placeholder content for selected dimension metrics
            let dim_score = score.as_slice()[selected_dim.unwrap()];
            let status = if dim_score >= threshold {
                "PASS"
            } else {
                "FAIL"
            };
            let status_color = if dim_score >= threshold {
                tok::semantic::SUCCESS
            } else {
                tok::semantic::DANGER
            };
            detail_panel = detail_panel.child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .py(px(2.))
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child("Score"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", dim_score)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_MUTED))
                            .child(format!("{:.2}", threshold)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(status_color))
                            .child(status),
                    ),
            );
        }

        // History sparkline -- render inline using canvas
        let history_ref = &self.history;
        let selected_dim_for_sparkline = selected_dim.unwrap_or(0);
        let trend: Vec<f32> = history_ref.dimension_trend(selected_dim_for_sparkline);
        let current_dim_score = history_ref
            .latest()
            .map(|s| s.score.as_slice()[selected_dim_for_sparkline])
            .unwrap_or(score.as_slice()[selected_dim_for_sparkline]);
        let sparkline_height: f32 = 80.0;

        let sparkline_label =
            history_view::HistoryView::dimension_label(selected_dim_for_sparkline).to_string();
        let sparkline_dim_color = if current_dim_score >= 0.75 {
            tok::semantic::SUCCESS
        } else {
            tok::semantic::DANGER
        };

        // Build sparkline points
        let sparkline_points: Vec<(f32, f32)> = if trend.len() >= 2 {
            let step = 260.0 / (trend.len() - 1).max(1) as f32;
            trend
                .iter()
                .enumerate()
                .map(|(i, &s)| {
                    let x = 16.0 + i as f32 * step;
                    let y = (sparkline_height - 16.0) * (1.0 - s) + 8.0;
                    (x, y)
                })
                .collect()
        } else {
            vec![]
        };

        let sparkline_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let ox = pf(bounds.origin.x);
                let oy = pf(bounds.origin.y);

                if sparkline_points.len() >= 2 {
                    let mut builder = gpui::PathBuilder::stroke(px(2.0));
                    let &(fx, fy) = sparkline_points.first().unwrap();
                    builder.move_to(point(px(ox + fx), px(oy + fy)));
                    for &(x, y) in &sparkline_points[1..] {
                        builder.line_to(point(px(ox + x), px(oy + y)));
                    }
                    if let Ok(path) = builder.build() {
                        window.paint_path(path, gpui::rgb(tok::ACCENT));
                    }
                }
            },
        )
        .size_full();

        let sparkline_section = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(4.))
            .child(
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
                            .child(sparkline_label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(sparkline_dim_color))
                            .child(format!("{:.2}", current_dim_score)),
                    ),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(sparkline_height))
                    .bg(rgb(tok::BG_PANEL))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .child(sparkline_canvas),
            );

        // Gate indicators
        let gates = self.gate.dimension_gates();
        let all_pass = self.gate.all_pass();
        let failing = self.gate.failing_dimensions();

        let (badge_text, badge_bg) = if all_pass {
            ("GATE: PASS", tok::semantic::SUCCESS)
        } else {
            ("GATE: FAIL", tok::semantic::DANGER)
        };
        let badge_text_color = tok::theme::dark::text::ON_PRIMARY;

        let mut gate_bar = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE));

        gate_bar = gate_bar.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .px(px(8.))
                        .py(px(2.))
                        .rounded_md()
                        .bg(rgb(badge_bg))
                        .text_xs()
                        .text_color(rgb(badge_text_color))
                        .child(badge_text.to_string()),
                )
                .when(!all_pass, |el: gpui::Div| {
                    let fail_text = failing.join(", ");
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::semantic::DANGER))
                            .child(format!("({})", fail_text)),
                    )
                }),
        );

        let mut indicators = div().flex().flex_row().gap(px(8.)).w_full();
        for gate in &gates {
            let (status_text, status_color) = match gate.status {
                GateStatus::Pass => ("PASS", tok::semantic::SUCCESS),
                GateStatus::Fail => ("FAIL", tok::semantic::DANGER),
            };
            indicators = indicators.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(2.))
                    .flex_grow()
                    .p(px(6.))
                    .rounded_md()
                    .bg(rgb(tok::BG_SURFACE))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(gate.label.to_string()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", gate.score)),
                    )
                    .child(
                        div()
                            .px(px(4.))
                            .py(px(1.))
                            .rounded_sm()
                            .bg(rgb(status_color))
                            .text_xs()
                            .text_color(rgb(badge_text_color))
                            .child(status_text.to_string()),
                    ),
            );
        }
        gate_bar = gate_bar.child(indicators);

        // Combined layout (REQ-QD-022~024)
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .bg(rgb(tok::BG_APP))
            // Row 1: Radar + Dimension Detail side by side
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.))
                    .child(radar_chart)
                    .child(div().flex_grow().child(detail_panel)),
            )
            // Row 2: History sparkline
            .child(sparkline_section)
            // Row 3: Gate bar
            .child(gate_bar)
    }
}

// ============================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::quality::Trust5Score;

    /// REQ-QD-022: Default dashboard has zero scores.
    #[test]
    fn dashboard_view_default_score() {
        let view = QualityDashboardView::default();
        assert_eq!(view.radar.score.tested, 0.0);
        assert_eq!(view.gate.score.tested, 0.0);
        assert!(view.selected_dimension.is_none());
        assert!(view.history.is_empty());
    }

    /// REQ-QD-023: select_dimension updates selected state.
    #[test]
    fn dashboard_view_select_dimension() {
        let mut view = QualityDashboardView::default();
        assert!(view.selected_dimension.is_none());

        view.select_dimension(2);
        assert_eq!(view.selected_dimension, Some(2));
        assert_eq!(view.radar.hovered_axis, Some(2));

        view.clear_selection();
        assert!(view.selected_dimension.is_none());
        assert!(view.radar.hovered_axis.is_none());
    }

    /// REQ-QD-024: with_score sets score across all sub-views.
    #[test]
    fn dashboard_view_with_score() {
        let score = Trust5Score::new_with_scores(0.9, 0.8, 0.7, 0.6, 0.5);
        let view = QualityDashboardView::with_score(score);
        assert!((view.radar.score.overall() - 0.7).abs() < 0.01);
        assert!((view.gate.score.overall() - 0.7).abs() < 0.01);
    }

    /// set_score propagates to radar and gate.
    #[test]
    fn dashboard_view_set_score() {
        let mut view = QualityDashboardView::new();
        let score = Trust5Score::new_with_scores(0.5, 0.6, 0.7, 0.8, 0.9);
        view.set_score(score);
        assert!((view.radar.score.tested - 0.5).abs() < f32::EPSILON);
        assert!((view.gate.score.tested - 0.5).abs() < f32::EPSILON);
    }

    /// select_dimension clamps to 4.
    #[test]
    fn dashboard_view_select_dimension_clamps() {
        let mut view = QualityDashboardView::default();
        view.select_dimension(10);
        assert_eq!(view.selected_dimension, Some(4));
    }
}
