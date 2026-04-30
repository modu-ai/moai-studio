//! Radar chart GPUI view for TRUST 5 quality dashboard (SPEC-V3-017 MS-2)
//!
//! Renders a 5-axis radar chart using GPUI canvas for custom drawing.
//! Axes: Tested (T), Readable (R), Unified (U), Secured (S), Trackable (K).
//!
//! REQ-QD-010~014: Radar chart with score polygon, threshold reference, and axis labels.

use gpui::{
    Context, IntoElement, ParentElement, Pixels, Render, Styled, Window, canvas, div, point, px,
    rgb,
};
use moai_studio_agent::quality::Trust5Score;

use crate::design::tokens as tok;

/// Dimension labels displayed around the radar chart (T, R, U, S, K).
pub const DIMENSION_LABELS: [&str; 5] = ["T", "R", "U", "S", "K"];

/// Full dimension names for tooltip/label context.
pub const DIMENSION_NAMES: [&str; 5] = ["Tested", "Readable", "Unified", "Secured", "Trackable"];

/// Default threshold value for the quality gate (0.75).
pub const DEFAULT_THRESHOLD: f32 = 0.75;

/// Default chart size in pixels.
pub const DEFAULT_CHART_SIZE: f32 = 300.0;

// @MX:ANCHOR: [AUTO] radar-chart-axis-geometry
// @MX:REASON: [AUTO] SPEC-V3-017 REQ-QD-010~014. Axis geometry is the single source of truth
//   for radar chart rendering. fan_in >= 3: axis_position, polygon_vertices, render.
//   5 axes at 72-degree intervals starting from top (-90 degrees).

/// Angle offset for each axis in radians (5 axes at 72-degree intervals from top).
/// Axis 0 (T) starts at -PI/2 (top), rotating clockwise.
pub const AXIS_ANGLES: [f32; 5] = [
    -std::f32::consts::FRAC_PI_2,             // -90 deg (top)
    -std::f32::consts::FRAC_PI_2 + 1.2566371, // -18 deg (top-right)
    -std::f32::consts::FRAC_PI_2 + 2.5132741, //  54 deg (bottom-right)
    -std::f32::consts::FRAC_PI_2 + 3.7699112, // 126 deg (bottom-left)
    -std::f32::consts::FRAC_PI_2 + 5.0265482, // 198 deg (top-left)
];

/// GPUI component that renders a 5-axis radar chart for TRUST 5 scores.
///
/// The chart shows:
/// - Score polygon connecting the 5 dimension scores
/// - Threshold reference polygon (distinct outline)
/// - Axis lines from center to perimeter
/// - Axis labels with score values
///
/// REQ-QD-010: 5-axis radar chart rendering.
/// REQ-QD-011: Axis labels with dimension name and numeric score.
/// REQ-QD-013: Threshold reference polygon.
pub struct RadarChartView {
    /// Current TRUST 5 score to display.
    pub score: Trust5Score,
    /// Gate threshold (e.g., 0.75). Drawn as a reference polygon.
    pub threshold: f32,
    /// Chart size in pixels (diameter).
    pub size: f32,
    /// Currently hovered axis index (0-4), used for highlight.
    pub hovered_axis: Option<usize>,
}

impl RadarChartView {
    /// Create a new RadarChartView with default settings and zero scores.
    pub fn new() -> Self {
        Self {
            score: Trust5Score::new(),
            threshold: DEFAULT_THRESHOLD,
            size: DEFAULT_CHART_SIZE,
            hovered_axis: None,
        }
    }

    /// Create a RadarChartView with the given score and default threshold/size.
    pub fn with_score(score: Trust5Score) -> Self {
        Self {
            score,
            threshold: DEFAULT_THRESHOLD,
            size: DEFAULT_CHART_SIZE,
            hovered_axis: None,
        }
    }

    /// Update the score.
    pub fn set_score(&mut self, score: Trust5Score) {
        self.score = score;
    }

    /// Update the threshold (clamped to [0.0, 1.0]).
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }
}

impl Default for RadarChartView {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Geometry helpers (pure functions, easily testable)
// ============================================================

/// Compute the (x, y) position of a point on a given axis.
///
/// - `center_x`, `center_y`: center of the radar chart in pixels
/// - `radius`: maximum radius in pixels
/// - `axis_index`: axis 0-4 (T, R, U, S, K)
/// - `value`: score value 0.0-1.0 (0 = center, 1 = perimeter)
///
/// Returns (x, y) pixel coordinates relative to chart origin.
pub fn axis_position(
    center_x: f32,
    center_y: f32,
    radius: f32,
    axis_index: usize,
    value: f32,
) -> (f32, f32) {
    let angle = AXIS_ANGLES[axis_index.min(4)];
    let r = radius * value.clamp(0.0, 1.0);
    let x = center_x + r * angle.cos();
    let y = center_y + r * angle.sin();
    (x, y)
}

/// Compute all 5 polygon vertices for a given score.
///
/// Returns a Vec of 5 (x, y) coordinate pairs, one per axis.
pub fn polygon_vertices(
    score: &Trust5Score,
    center_x: f32,
    center_y: f32,
    radius: f32,
) -> Vec<(f32, f32)> {
    score
        .as_slice()
        .iter()
        .enumerate()
        .map(|(i, &v)| axis_position(center_x, center_y, radius, i, v))
        .collect()
}

/// Compute the label position for a given axis (offset beyond the perimeter).
///
/// Labels are placed slightly outside the perimeter for readability.
pub fn label_position(center_x: f32, center_y: f32, radius: f32, axis_index: usize) -> (f32, f32) {
    const LABEL_OFFSET: f32 = 20.0;
    axis_position(center_x, center_y, radius + LABEL_OFFSET, axis_index, 1.0)
}

/// Format a score value for display (e.g., "0.85").
pub fn format_score(value: f32) -> String {
    format!("{:.2}", value)
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

impl Render for RadarChartView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.size;
        let center = size / 2.0;
        let radius = size / 2.0 - 30.0; // margin for labels
        let score = self.score;
        let threshold = self.threshold;

        // Precompute label positions for div overlays
        let label_data: Vec<(f32, f32, f32, u32)> = (0..5)
            .map(|i| {
                let (lx, ly) = label_position(center, center, radius, i);
                let dim_score = score.as_slice()[i];
                let label_color = if dim_score >= threshold {
                    tok::semantic::SUCCESS
                } else {
                    tok::semantic::DANGER
                };
                (lx, ly, dim_score, label_color)
            })
            .collect();

        // Canvas for drawing the radar chart
        let chart_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let ox = pf(bounds.origin.x);
                let oy = pf(bounds.origin.y);

                // Axis lines from center to perimeter
                for i in 0..5 {
                    let (ex, ey) = axis_position(center, center, radius, i, 1.0);
                    let path = line_path(ox + center, oy + center, ox + ex, oy + ey, 1.0);
                    window.paint_path(path, gpui::rgba(tok::FG_MUTED | 0x4d000000));
                }

                // Threshold reference polygon (stroke only)
                let threshold_points: Vec<(f32, f32)> = (0..5)
                    .map(|i| axis_position(center, center, radius, i, threshold))
                    .collect();
                let threshold_path = stroke_path_from_points(&threshold_points, ox, oy, 1.0);
                window.paint_path(threshold_path, gpui::rgba(tok::FG_MUTED | 0x60000000));

                // Score polygon: fill with semi-transparent accent
                let score_points = polygon_vertices(&score, center, center, radius);
                let fill_path = fill_path_from_points(&score_points, ox, oy);
                window.paint_path(fill_path, gpui::rgba(tok::ACCENT | 0x33000000));

                // Score polygon: stroke with accent
                let stroke_path = stroke_path_from_points(&score_points, ox, oy, 2.0);
                window.paint_path(stroke_path, gpui::rgb(tok::ACCENT));

                // Score dots at each vertex
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

        // Build the chart container with canvas + label overlays
        let mut chart = div()
            .relative()
            .w(px(size))
            .h(px(size))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .child(chart_canvas);

        // Axis labels as positioned div overlays
        for (i, &(lx, ly, dim_score, label_color)) in label_data.iter().enumerate() {
            chart = chart.child(
                div()
                    .absolute()
                    .left(px(lx - 16.0))
                    .top(px(ly - 10.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(label_color))
                            .child(DIMENSION_LABELS[i].to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format_score(dim_score)),
                    ),
            );
        }

        chart
    }
}

// ============================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::quality::Trust5Score;

    const CENTER_X: f32 = 150.0;
    const CENTER_Y: f32 = 150.0;
    const RADIUS: f32 = 120.0;

    /// REQ-QD-010: Top axis (index 0, Tested) is at the top of the chart.
    #[test]
    fn axis_position_top_axis_is_at_top() {
        let (x, y) = axis_position(CENTER_X, CENTER_Y, RADIUS, 0, 1.0);
        assert!(
            (x - CENTER_X).abs() < 0.01,
            "Top axis x should be at center: got {x}"
        );
        assert!(
            (y - (CENTER_Y - RADIUS)).abs() < 0.01,
            "Top axis y should be above center: got {y}, expected {}",
            CENTER_Y - RADIUS
        );
    }

    /// REQ-QD-010: Five axes are evenly spaced at 72-degree intervals.
    #[test]
    fn axis_position_five_axes_are_evenly_spaced() {
        let points: Vec<(f32, f32)> = (0..5)
            .map(|i| axis_position(CENTER_X, CENTER_Y, RADIUS, i, 1.0))
            .collect();

        for (i, &(x, y)) in points.iter().enumerate() {
            let dist = ((x - CENTER_X).powi(2) + (y - CENTER_Y).powi(2)).sqrt();
            assert!(
                (dist - RADIUS).abs() < 0.1,
                "Axis {i} should be at radius distance: got {dist}"
            );
        }

        for i in 0..5 {
            let j = (i + 1) % 5;
            let (x1, y1) = points[i];
            let (x2, y2) = points[j];
            let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            let expected_edge = 2.0 * RADIUS * (std::f32::consts::FRAC_PI_2 / 2.5).sin();
            assert!(
                (dist - expected_edge).abs() < 0.1,
                "Edge {i}-{j} distance should be {expected_edge:.2}, got {dist:.2}"
            );
        }
    }

    /// REQ-QD-010: polygon_vertices produces 5 points from a score.
    #[test]
    fn polygon_vertices_from_score() {
        let score = Trust5Score::new_with_scores(0.8, 0.6, 0.4, 0.9, 0.7);
        let verts = polygon_vertices(&score, CENTER_X, CENTER_Y, RADIUS);
        assert_eq!(verts.len(), 5, "Should produce 5 vertices");
    }

    /// REQ-QD-010: When all scores are 0, all vertices are at center.
    #[test]
    fn polygon_vertices_all_zeros_at_center() {
        let score = Trust5Score::new();
        let verts = polygon_vertices(&score, CENTER_X, CENTER_Y, RADIUS);
        for (i, &(x, y)) in verts.iter().enumerate() {
            assert!(
                (x - CENTER_X).abs() < 0.01 && (y - CENTER_Y).abs() < 0.01,
                "Axis {i} with score 0 should be at center: ({x}, {y})"
            );
        }
    }

    /// REQ-QD-010: When all scores are 1, all vertices are at perimeter.
    #[test]
    fn polygon_vertices_all_ones_at_perimeter() {
        let score = Trust5Score::new_with_scores(1.0, 1.0, 1.0, 1.0, 1.0);
        let verts = polygon_vertices(&score, CENTER_X, CENTER_Y, RADIUS);
        for (i, &(x, y)) in verts.iter().enumerate() {
            let dist = ((x - CENTER_X).powi(2) + (y - CENTER_Y).powi(2)).sqrt();
            assert!(
                (dist - RADIUS).abs() < 0.1,
                "Axis {i} with score 1 should be at perimeter: dist = {dist}"
            );
        }
    }

    /// REQ-QD-013: Default threshold is 0.75.
    #[test]
    fn radar_chart_default_threshold() {
        let view = RadarChartView::new();
        assert!(
            (view.threshold - 0.75).abs() < f32::EPSILON,
            "Default threshold should be 0.75"
        );
    }

    /// Score update reflects in the view.
    #[test]
    fn radar_chart_score_update() {
        let mut view = RadarChartView::new();
        let score = Trust5Score::new_with_scores(0.9, 0.8, 0.7, 0.6, 0.5);
        view.set_score(score);
        assert!((view.score.tested - 0.9).abs() < f32::EPSILON);
        assert!((view.score.readable - 0.8).abs() < f32::EPSILON);
    }

    /// Default chart size is 300px.
    #[test]
    fn radar_chart_default_size() {
        let view = RadarChartView::new();
        assert!((view.size - 300.0).abs() < f32::EPSILON);
    }

    /// Threshold is clamped to [0.0, 1.0].
    #[test]
    fn radar_chart_threshold_clamped() {
        let mut view = RadarChartView::new();
        view.set_threshold(1.5);
        assert!((view.threshold - 1.0).abs() < f32::EPSILON);
        view.set_threshold(-0.5);
        assert!((view.threshold - 0.0).abs() < f32::EPSILON);
    }

    /// Label position is outside the perimeter.
    #[test]
    fn label_position_outside_perimeter() {
        let (lx, ly) = label_position(CENTER_X, CENTER_Y, RADIUS, 0);
        let (px, py) = axis_position(CENTER_X, CENTER_Y, RADIUS, 0, 1.0);
        let label_dist = ((lx - CENTER_X).powi(2) + (ly - CENTER_Y).powi(2)).sqrt();
        let perim_dist = ((px - CENTER_X).powi(2) + (py - CENTER_Y).powi(2)).sqrt();
        assert!(
            label_dist > perim_dist,
            "Label should be outside perimeter: label_dist={label_dist}, perim_dist={perim_dist}"
        );
    }

    /// format_score shows 2 decimal places.
    #[test]
    fn format_score_two_decimals() {
        assert_eq!(format_score(0.856), "0.86");
        assert_eq!(format_score(0.0), "0.00");
        assert_eq!(format_score(1.0), "1.00");
    }

    /// with_score constructor sets score correctly.
    #[test]
    fn radar_chart_with_score() {
        let score = Trust5Score::new_with_scores(0.5, 0.6, 0.7, 0.8, 0.9);
        let view = RadarChartView::with_score(score);
        assert!((view.score.overall() - 0.7).abs() < 0.01);
        assert!((view.threshold - DEFAULT_THRESHOLD).abs() < f32::EPSILON);
    }

    /// Axis position at score 0.5 should be at half radius.
    #[test]
    fn axis_position_half_score_at_half_radius() {
        let (x, y) = axis_position(CENTER_X, CENTER_Y, RADIUS, 0, 0.5);
        let dist = ((x - CENTER_X).powi(2) + (y - CENTER_Y).powi(2)).sqrt();
        assert!(
            (dist - RADIUS * 0.5).abs() < 0.1,
            "Half score should be at half radius: dist={dist}, expected={}",
            RADIUS * 0.5
        );
    }
}
