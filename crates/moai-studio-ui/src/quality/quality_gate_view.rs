//! Quality gate indicator bar for TRUST 5 dashboard (SPEC-V3-017 MS-2)
//!
//! Horizontal bar with 5 gate indicators (one per dimension).
//! Each indicator shows PASS (green) or FAIL (red) based on score vs threshold.
//!
//! REQ-QD-015~017: Quality gate display with overall pass/fail status.

use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
    rgb,
};
use moai_studio_agent::quality::Trust5Score;

use crate::design::tokens as tok;

/// Default threshold for the quality gate (0.75).
const DEFAULT_GATE_THRESHOLD: f32 = 0.75;

/// Gate result for a single dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    /// Score meets or exceeds threshold.
    Pass,
    /// Score is below threshold.
    Fail,
}

/// Per-dimension gate result.
#[derive(Debug, Clone, Copy)]
pub struct DimensionGate {
    /// Dimension abbreviation (T, R, U, S, K).
    pub label: &'static str,
    /// Dimension full name.
    pub name: &'static str,
    /// Dimension score value.
    pub score: f32,
    /// Threshold value.
    pub threshold: f32,
    /// Pass or fail status.
    pub status: GateStatus,
}

/// GPUI component that renders a horizontal quality gate bar.
///
/// Shows 5 gate indicators (T, R, U, S, K) with PASS/FAIL status.
/// If any gate fails, shows an overall "GATE: FAIL" badge.
///
/// REQ-QD-015: Quality gate indicator display.
/// REQ-QD-016: Overall gate status with failing dimension names.
/// REQ-QD-017: Gate status uses design tokens for colors.
pub struct QualityGateView {
    /// Current TRUST 5 score.
    pub score: Trust5Score,
    /// Gate threshold (0.75 by default).
    pub threshold: f32,
}

impl QualityGateView {
    /// Create a new QualityGateView with default threshold and zero scores.
    pub fn new() -> Self {
        Self {
            score: Trust5Score::new(),
            threshold: DEFAULT_GATE_THRESHOLD,
        }
    }

    /// Create a QualityGateView with the given score and threshold.
    pub fn with_score_and_threshold(score: Trust5Score, threshold: f32) -> Self {
        Self {
            score,
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Evaluate gate status for a single dimension.
    pub fn gate_status(&self, dimension_score: f32) -> GateStatus {
        if dimension_score >= self.threshold {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        }
    }

    /// Compute per-dimension gate results.
    pub fn dimension_gates(&self) -> Vec<DimensionGate> {
        let scores = self.score.as_slice();
        let labels = ["T", "R", "U", "S", "K"];
        let names = ["Tested", "Readable", "Unified", "Secured", "Trackable"];

        scores
            .iter()
            .zip(labels.iter().zip(names.iter()))
            .map(|(&score, (&label, &name))| {
                let status = self.gate_status(score);
                DimensionGate {
                    label,
                    name,
                    score,
                    threshold: self.threshold,
                    status,
                }
            })
            .collect()
    }

    /// Check if all gates pass.
    pub fn all_pass(&self) -> bool {
        self.score
            .as_slice()
            .iter()
            .all(|&s| self.gate_status(s) == GateStatus::Pass)
    }

    /// Get the list of failing dimension labels.
    pub fn failing_dimensions(&self) -> Vec<&'static str> {
        let gates = self.dimension_gates();
        gates
            .into_iter()
            .filter(|g| g.status == GateStatus::Fail)
            .map(|g| g.label)
            .collect()
    }
}

impl Default for QualityGateView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for QualityGateView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let gates = self.dimension_gates();
        let all_pass = self.all_pass();
        let failing = self.failing_dimensions();

        // Overall gate badge
        let (badge_text, badge_bg) = if all_pass {
            ("GATE: PASS", tok::semantic::SUCCESS)
        } else {
            ("GATE: FAIL", tok::semantic::DANGER)
        };

        // Determine text color for badge - use light text on dark background
        let badge_text_color = tok::theme::dark::text::ON_PRIMARY;

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE));

        // Overall gate status badge
        container = container.child(
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
                    // Show failing dimension names
                    let fail_text = failing.join(", ");
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::semantic::DANGER))
                            .child(format!("({})", fail_text)),
                    )
                }),
        );

        // Individual gate indicators
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
                        // Dimension label
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(gate.label.to_string()),
                    )
                    .child(
                        // Score value
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", gate.score)),
                    )
                    .child(
                        // Pass/Fail badge
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

        container = container.child(indicators);
        container
    }
}

// ============================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::quality::Trust5Score;

    /// REQ-QD-015: Gate passes when score is above threshold.
    #[test]
    fn gate_pass_when_above_threshold() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.8, 0.0, 0.0, 0.0, 0.0),
            0.75,
        );
        assert_eq!(view.gate_status(0.8), GateStatus::Pass);
    }

    /// REQ-QD-015: Gate fails when score is below threshold.
    #[test]
    fn gate_fail_when_below_threshold() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.0, 0.0, 0.0, 0.0, 0.0),
            0.75,
        );
        assert_eq!(view.gate_status(0.5), GateStatus::Fail);
    }

    /// REQ-QD-015: All gates pass when all scores meet threshold.
    #[test]
    fn gate_all_pass() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.9, 0.85, 0.80, 0.78, 0.95),
            0.75,
        );
        assert!(view.all_pass(), "All scores above 0.75 should pass");
        assert!(view.failing_dimensions().is_empty());
    }

    /// REQ-QD-015: All gates fail when all scores are below threshold.
    #[test]
    fn gate_all_fail() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.1, 0.2, 0.3, 0.4, 0.5),
            0.75,
        );
        assert!(!view.all_pass(), "All scores below 0.75 should fail");
        let failing = view.failing_dimensions();
        assert_eq!(failing.len(), 5, "All 5 dimensions should fail");
    }

    /// REQ-QD-016: Mixed results correctly identify failing dimensions.
    #[test]
    fn gate_mixed_results() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.9, 0.5, 0.8, 0.3, 0.6),
            0.75,
        );
        assert!(!view.all_pass(), "Not all pass");
        let failing = view.failing_dimensions();
        // R=0.5 < 0.75, S=0.3 < 0.75, K=0.6 < 0.75
        assert_eq!(
            failing,
            vec!["R", "S", "K"],
            "Readable, Secured, Trackable should fail"
        );
    }

    /// REQ-QD-016: Failing dimensions list shows correct labels.
    #[test]
    fn gate_fail_shows_failing_dimensions() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.8, 0.8, 0.5, 0.8, 0.8),
            0.75,
        );
        let failing = view.failing_dimensions();
        assert_eq!(failing, vec!["U"], "Only Unified should fail");
    }

    /// Gate at exact threshold value should pass.
    #[test]
    fn gate_pass_at_exact_threshold() {
        let view = QualityGateView::with_score_and_threshold(
            Trust5Score::new_with_scores(0.75, 0.75, 0.75, 0.75, 0.75),
            0.75,
        );
        assert!(view.all_pass(), "Score at exact threshold should pass");
    }

    /// dimension_gates returns 5 results.
    #[test]
    fn dimension_gates_count() {
        let view = QualityGateView::new();
        let gates = view.dimension_gates();
        assert_eq!(gates.len(), 5, "Should have 5 dimension gates");
    }

    /// dimension_gates labels are in correct order.
    #[test]
    fn dimension_gates_labels_order() {
        let view = QualityGateView::new();
        let gates = view.dimension_gates();
        let labels: Vec<&str> = gates.iter().map(|g| g.label).collect();
        assert_eq!(labels, vec!["T", "R", "U", "S", "K"]);
    }

    /// Default threshold is 0.75.
    #[test]
    fn default_threshold_is_075() {
        let view = QualityGateView::new();
        assert!((view.threshold - 0.75).abs() < f32::EPSILON);
    }

    /// Threshold is clamped to [0.0, 1.0].
    #[test]
    fn threshold_clamped() {
        let view = QualityGateView::with_score_and_threshold(Trust5Score::new(), 1.5);
        assert!((view.threshold - 1.0).abs() < f32::EPSILON);

        let view = QualityGateView::with_score_and_threshold(Trust5Score::new(), -0.5);
        assert!((view.threshold - 0.0).abs() < f32::EPSILON);
    }
}
