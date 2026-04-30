//! Dimension detail view for TRUST 5 quality dashboard (SPEC-V3-017 MS-3)
//!
//! Shows contributing metrics for a selected TRUST 5 dimension.
//! Displays a table-like layout with metric name, value, threshold, and pass/fail status.
//!
//! REQ-QD-025~028: Dimension detail panel with metric items.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};

use crate::design::tokens as tok;

/// Maximum number of metric items displayed per dimension (REQ-QD-027).
pub const MAX_METRICS_PER_DIMENSION: usize = 10;

/// A single contributing metric item for a dimension.
#[derive(Debug, Clone)]
pub struct MetricItem {
    /// Human-readable metric name.
    pub name: String,
    /// Current metric value as a display string.
    pub value: String,
    /// Threshold value as a display string (if applicable).
    pub threshold: Option<String>,
    /// Whether this metric passes its threshold.
    pub pass: bool,
}

impl MetricItem {
    /// Create a new MetricItem.
    pub fn new(name: impl Into<String>, value: impl Into<String>, pass: bool) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            threshold: None,
            pass,
        }
    }

    /// Create a MetricItem with a threshold.
    pub fn with_threshold(
        name: impl Into<String>,
        value: impl Into<String>,
        threshold: impl Into<String>,
        pass: bool,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            threshold: Some(threshold.into()),
            pass,
        }
    }
}

/// Contributing metrics for a TRUST 5 dimension.
#[derive(Debug, Clone, Default)]
pub struct DimensionMetrics {
    /// Metric items for this dimension.
    pub items: Vec<MetricItem>,
}

impl DimensionMetrics {
    /// Create empty dimension metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a metric item, respecting MAX_METRICS_PER_DIMENSION.
    pub fn add(&mut self, item: MetricItem) {
        if self.items.len() < MAX_METRICS_PER_DIMENSION {
            self.items.push(item);
        }
    }

    /// Create from a Vec of items, truncating to MAX_METRICS_PER_DIMENSION.
    pub fn from_items(items: Vec<MetricItem>) -> Self {
        Self {
            items: items.into_iter().take(MAX_METRICS_PER_DIMENSION).collect(),
        }
    }
}

/// GPUI component showing contributing metrics for a selected dimension.
///
/// Displays a table-like layout with columns: name | value | threshold | status.
/// Shows placeholder text when no dimension is selected.
///
/// REQ-QD-025: Dimension detail panel.
/// REQ-QD-026: Metric item display with name, value, threshold, status.
/// REQ-QD-027: Max 10 metrics per dimension.
/// REQ-QD-028: Placeholder when no dimension selected.
pub struct DimensionDetailView {
    /// Currently selected dimension index (None = no selection).
    pub dimension_idx: Option<usize>,
    /// Contributing metrics for the selected dimension.
    pub metrics: Option<DimensionMetrics>,
}

impl DimensionDetailView {
    /// Create a new DimensionDetailView with no selection.
    pub fn new() -> Self {
        Self {
            dimension_idx: None,
            metrics: None,
        }
    }

    /// Create a view with the given dimension and metrics.
    pub fn with_metrics(dimension_idx: usize, metrics: DimensionMetrics) -> Self {
        Self {
            dimension_idx: Some(dimension_idx),
            metrics: Some(metrics),
        }
    }

    /// Select a dimension (clears metrics until set explicitly).
    pub fn select(&mut self, idx: usize) {
        self.dimension_idx = Some(idx.min(4));
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.dimension_idx = None;
        self.metrics = None;
    }

    /// Get the dimension label for the current selection.
    fn dimension_label(&self) -> &'static str {
        match self.dimension_idx {
            Some(0) => "Tested",
            Some(1) => "Readable",
            Some(2) => "Unified",
            Some(3) => "Secured",
            Some(4) => "Trackable",
            _ => "N/A",
        }
    }
}

impl Default for DimensionDetailView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for DimensionDetailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let dim_label = self.dimension_label().to_string();
        let has_selection = self.dimension_idx.is_some();
        let items = self
            .metrics
            .as_ref()
            .map(|m| m.items.clone())
            .unwrap_or_default();

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

        // Header with dimension name
        container = container.child(div().text_sm().text_color(rgb(tok::FG_PRIMARY)).child(
            if has_selection {
                format!("{} Metrics", dim_label)
            } else {
                "Dimension Metrics".to_string()
            },
        ));

        if !has_selection {
            // Placeholder when no dimension is selected (REQ-QD-028)
            container = container.child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .py(px(16.))
                    .child("Select a dimension to view metrics"),
            );
        } else if items.is_empty() {
            container = container.child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .py(px(8.))
                    .child("No metrics available for this dimension"),
            );
        } else {
            // Table header
            let mut table_header = div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .pb(px(4.))
                .border_b_1()
                .border_color(rgb(tok::BORDER_SUBTLE));

            for label in ["Metric", "Value", "Threshold", "Status"] {
                table_header = table_header.child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(rgb(tok::FG_MUTED))
                        .child(label.to_string()),
                );
            }
            container = container.child(table_header);

            // Metric rows
            for item in &items {
                let status_text = if item.pass { "PASS" } else { "FAIL" };
                let status_color = if item.pass {
                    tok::semantic::SUCCESS
                } else {
                    tok::semantic::DANGER
                };

                let row = div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .py(px(2.))
                    .child(
                        // Metric name
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(item.name.clone()),
                    )
                    .child(
                        // Value
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(item.value.clone()),
                    )
                    .child(
                        // Threshold
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(tok::FG_MUTED))
                            .child(item.threshold.clone().unwrap_or_else(|| "-".to_string())),
                    )
                    .child(
                        // Status
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(rgb(status_color))
                            .child(status_text.to_string()),
                    );

                container = container.child(row);
            }
        }

        container
    }
}

// ============================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-QD-028: Default view has no selection.
    #[test]
    fn detail_view_default_no_selection() {
        let view = DimensionDetailView::default();
        assert!(view.dimension_idx.is_none());
        assert!(view.metrics.is_none());
    }

    /// REQ-QD-026: MetricItem pass/fail states.
    #[test]
    fn detail_view_metric_item_pass_fail() {
        let pass_item = MetricItem::new("Coverage", "85%", true);
        assert!(pass_item.pass);
        assert_eq!(pass_item.name, "Coverage");
        assert_eq!(pass_item.value, "85%");

        let fail_item = MetricItem::with_threshold("Lint Errors", "5", "0", false);
        assert!(!fail_item.pass);
        assert_eq!(fail_item.threshold.as_deref(), Some("0"));
    }

    /// REQ-QD-027: Max 10 metrics per dimension enforced.
    #[test]
    fn detail_view_max_10_metrics() {
        let mut metrics = DimensionMetrics::new();
        for i in 0..15 {
            metrics.add(MetricItem::new(
                format!("Metric {}", i),
                format!("{}", i),
                true,
            ));
        }
        assert_eq!(metrics.items.len(), MAX_METRICS_PER_DIMENSION);
    }

    /// REQ-QD-027: from_items truncates to max.
    #[test]
    fn detail_view_from_items_truncates() {
        let items: Vec<MetricItem> = (0..20)
            .map(|i| MetricItem::new(format!("M{}", i), format!("{}", i), true))
            .collect();
        let metrics = DimensionMetrics::from_items(items);
        assert_eq!(metrics.items.len(), MAX_METRICS_PER_DIMENSION);
    }

    /// select sets dimension_idx.
    #[test]
    fn detail_view_select() {
        let mut view = DimensionDetailView::new();
        view.select(2);
        assert_eq!(view.dimension_idx, Some(2));
    }

    /// clear resets selection and metrics.
    #[test]
    fn detail_view_clear() {
        let mut view = DimensionDetailView::with_metrics(
            1,
            DimensionMetrics::from_items(vec![MetricItem::new("test", "1", true)]),
        );
        view.clear();
        assert!(view.dimension_idx.is_none());
        assert!(view.metrics.is_none());
    }
}
