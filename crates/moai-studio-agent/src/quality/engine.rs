//! Scoring engine trait and default heuristic implementation (SPEC-V3-017 RG-QD-2)
//!
//! REQ-QD-009: ScoringEngine trait for swappable scoring strategies.
//! REQ-QD-005 through REQ-QD-008: Default heuristic formulas.

use crate::quality::{
    Trust5Score,
    metrics::{GitMetrics, LspMetrics, SecurityMetrics, TestMetrics},
};

// @MX:ANCHOR: [AUTO] scoring-engine-trait
// @MX:REASON: [AUTO] ScoringEngine trait enables pluggable scoring strategies. fan_in >= 3:
//   DefaultHeuristicEngine, future ML-based engines, test doubles.
//   SPEC: SPEC-V3-017 REQ-QD-009

/// Trait for computing TRUST 5 scores from metric snapshots (REQ-QD-009).
///
/// Enables different scoring strategies (heuristic, ML-based, etc.)
/// via trait implementations.
pub trait ScoringEngine: Send + Sync {
    /// Compute a TRUST 5 score from available metrics.
    ///
    /// Missing metrics should be handled gracefully (typically with neutral scores).
    fn compute_score(
        &self,
        lsp: Option<&LspMetrics>,
        test: Option<&TestMetrics>,
        git: Option<&GitMetrics>,
        security: Option<&SecurityMetrics>,
    ) -> Trust5Score;
}

/// Default heuristic scoring engine (REQ-QD-005 through REQ-QD-008).
///
/// Uses simple formulas to convert raw metrics into 0.0-1.0 scores.
///
/// Scoring formulas:
/// - **Tested** (REQ-QD-006): coverage_percent / 100.0, clamped. No data → 0.5 (neutral)
/// - **Readable** (REQ-QD-005): 1.0 - (lint_errors / max(1, total_lines / 100)). No data → 0.5
/// - **Unified** (REQ-QD-005): 1.0 if fmt_check_pass, else 0.0. No data → 0.5
/// - **Secured** (REQ-QD-008): 0.0 if critical/high > 0, else 1.0 - (medium*0.1 + low*0.05). No data → 0.5
/// - **Trackable** (REQ-QD-007): (conv_commit_pct + spec_ref_pct + mx_tag_pct) / 300.0. No data → 0.5
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultHeuristicEngine;

impl ScoringEngine for DefaultHeuristicEngine {
    // @MX:ANCHOR: [AUTO] heuristic-scoring-compute
    // @MX:REASON: [AUTO] Primary scoring entry point. fan_in >= 3:
    //   QualityDashboardView, tests, future quality history components.
    //   SPEC: SPEC-V3-017 REQ-QD-005/006/007/008
    fn compute_score(
        &self,
        lsp: Option<&LspMetrics>,
        test: Option<&TestMetrics>,
        git: Option<&GitMetrics>,
        security: Option<&SecurityMetrics>,
    ) -> Trust5Score {
        Trust5Score {
            tested: compute_tested(test),
            readable: compute_readable(lsp),
            unified: compute_unified(lsp),
            secured: compute_secured(security),
            trackable: compute_trackable(git),
        }
    }
}

// ----------------------------------------------------------------
// Dimension scoring functions
// ----------------------------------------------------------------

/// Compute Tested score from test metrics (REQ-QD-006).
///
/// Formula: coverage_percent / 100.0
/// Clamp to [0.0, 1.0]
///
/// No data → 0.5 (neutral)
fn compute_tested(test: Option<&TestMetrics>) -> f32 {
    match test {
        None => 0.5,
        Some(metrics) => (metrics.coverage_percent / 100.0).clamp(0.0, 1.0),
    }
}

/// Compute Readable score from LSP metrics (REQ-QD-005).
///
/// Formula: 1.0 - (lint_errors / max(1, total_lines / 100))
/// Clamp to [0.0, 1.0]
///
/// No data → 0.5 (neutral)
fn compute_readable(lsp: Option<&LspMetrics>) -> f32 {
    match lsp {
        None => 0.5,
        Some(metrics) => {
            let denominator = (metrics.total_lines / 100).max(1) as f32;
            let penalty = metrics.lint_errors as f32 / denominator;
            (1.0 - penalty).clamp(0.0, 1.0)
        }
    }
}

/// Compute Unified score from LSP metrics (REQ-QD-005).
///
/// Formula: 1.0 if type_errors == 0, else 0.0
///
/// No data → 0.5 (neutral)
///
/// Note: LSPMetrics doesn't have a fmt_check_pass field yet,
/// so we use type_errors as a proxy. This may be refined in MS-2.
fn compute_unified(lsp: Option<&LspMetrics>) -> f32 {
    match lsp {
        None => 0.5,
        Some(metrics) => {
            if metrics.type_errors == 0 {
                1.0
            } else {
                0.0
            }
        }
    }
}

/// Compute Secured score from security metrics (REQ-QD-008).
///
/// Formula:
/// - If critical > 0 or high > 0 → 0.0
/// - Else → 1.0 - (medium * 0.1 + low * 0.05)
/// - Clamp to [0.0, 1.0]
///
/// No data → 0.5 (neutral)
fn compute_secured(security: Option<&SecurityMetrics>) -> f32 {
    match security {
        None => 0.5,
        Some(metrics) => {
            if metrics.critical_vulns > 0 || metrics.high_vulns > 0 {
                0.0
            } else {
                let penalty = metrics.medium_vulns as f32 * 0.1 + metrics.low_vulns as f32 * 0.05;
                (1.0 - penalty).clamp(0.0, 1.0)
            }
        }
    }
}

/// Compute Trackable score from git metrics (REQ-QD-007).
///
/// Formula: (conventional_commit_pct + spec_ref_pct + mx_tag_pct) / 300.0
/// Clamp to [0.0, 1.0]
///
/// No data → 0.5 (neutral)
fn compute_trackable(git: Option<&GitMetrics>) -> f32 {
    match git {
        None => 0.5,
        Some(metrics) => {
            let sum = metrics.conventional_commit_pct + metrics.spec_ref_pct + metrics.mx_tag_pct;
            (sum / 300.0).clamp(0.0, 1.0)
        }
    }
}

// ================================================================
// Tests
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create default metrics

    fn lsp_clean() -> LspMetrics {
        LspMetrics::new_with_counts(0, 0, 0, 1000, 50)
    }

    fn test_perfect() -> TestMetrics {
        TestMetrics::new_with_coverage(100.0, 100, 0)
    }

    fn git_perfect() -> GitMetrics {
        GitMetrics::new_with_percentages(100.0, 100.0, 100.0)
    }

    fn security_clean() -> SecurityMetrics {
        SecurityMetrics::new_with_vulns(0, 0, 0, 0, true)
    }

    // compute_tested tests (REQ-QD-006)

    #[test]
    fn tested_full_coverage() {
        let metrics = test_perfect();
        assert_eq!(compute_tested(Some(&metrics)), 1.0);
    }

    #[test]
    fn tested_half_coverage() {
        let metrics = TestMetrics::new_with_coverage(50.0, 10, 0);
        assert_eq!(compute_tested(Some(&metrics)), 0.5);
    }

    #[test]
    fn tested_no_data_neutral() {
        assert_eq!(compute_tested(None), 0.5);
    }

    #[test]
    fn tested_over_100_clamped() {
        let metrics = TestMetrics::new_with_coverage(150.0, 10, 0);
        assert_eq!(compute_tested(Some(&metrics)), 1.0);
    }

    #[test]
    fn tested_negative_clamped() {
        let metrics = TestMetrics::new_with_coverage(-10.0, 0, 0);
        assert_eq!(compute_tested(Some(&metrics)), 0.0);
    }

    // compute_readable tests (REQ-QD-005)

    #[test]
    fn readable_no_lint_errors() {
        let metrics = lsp_clean();
        assert_eq!(compute_readable(Some(&metrics)), 1.0);
    }

    #[test]
    fn readable_with_lint_errors() {
        // 1000 lines → denominator = max(1, 1000/100) = 10
        // 5 lint errors → penalty = 5/10 = 0.5 → score = 0.5
        let metrics = LspMetrics::new_with_counts(0, 5, 0, 1000, 50);
        assert_eq!(compute_readable(Some(&metrics)), 0.5);
    }

    #[test]
    fn readable_no_data_neutral() {
        assert_eq!(compute_readable(None), 0.5);
    }

    #[test]
    fn readable_clamped_to_zero() {
        // 100 lines → denominator = max(1, 100/100) = 1
        // 20 lint errors → penalty = 20/1 = 20 → clamped to 0.0
        let metrics = LspMetrics::new_with_counts(0, 20, 0, 100, 10);
        assert_eq!(compute_readable(Some(&metrics)), 0.0);
    }

    // compute_unified tests (REQ-QD-005)

    #[test]
    fn unified_no_type_errors() {
        let metrics = lsp_clean();
        assert_eq!(compute_unified(Some(&metrics)), 1.0);
    }

    #[test]
    fn unified_with_type_errors() {
        let metrics = LspMetrics::new_with_counts(5, 0, 0, 1000, 50);
        assert_eq!(compute_unified(Some(&metrics)), 0.0);
    }

    #[test]
    fn unified_no_data_neutral() {
        assert_eq!(compute_unified(None), 0.5);
    }

    // compute_secured tests (REQ-QD-008)

    #[test]
    fn secured_no_vulnerabilities() {
        let metrics = security_clean();
        assert_eq!(compute_secured(Some(&metrics)), 1.0);
    }

    #[test]
    fn secured_critical_vuln_zero_score() {
        let metrics = SecurityMetrics::new_with_vulns(1, 0, 0, 0, true);
        assert_eq!(compute_secured(Some(&metrics)), 0.0);
    }

    #[test]
    fn secured_high_vuln_zero_score() {
        let metrics = SecurityMetrics::new_with_vulns(0, 1, 0, 0, true);
        assert_eq!(compute_secured(Some(&metrics)), 0.0);
    }

    #[test]
    fn secured_medium_vuln_penalty() {
        // 5 medium → penalty = 5 * 0.1 = 0.5 → score = 0.5
        let metrics = SecurityMetrics::new_with_vulns(0, 0, 5, 0, true);
        assert_eq!(compute_secured(Some(&metrics)), 0.5);
    }

    #[test]
    fn secured_low_vuln_penalty() {
        // 10 low → penalty = 10 * 0.05 = 0.5 → score = 0.5
        let metrics = SecurityMetrics::new_with_vulns(0, 0, 0, 10, true);
        assert_eq!(compute_secured(Some(&metrics)), 0.5);
    }

    #[test]
    fn secured_mixed_vulns() {
        // 5 medium + 10 low → penalty = 0.5 + 0.5 = 1.0 → clamped to 0.0
        let metrics = SecurityMetrics::new_with_vulns(0, 0, 5, 10, true);
        assert_eq!(compute_secured(Some(&metrics)), 0.0);
    }

    #[test]
    fn secured_no_data_neutral() {
        assert_eq!(compute_secured(None), 0.5);
    }

    // compute_trackable tests (REQ-QD-007)

    #[test]
    fn trackable_perfect_scores() {
        let metrics = git_perfect();
        assert_eq!(compute_trackable(Some(&metrics)), 1.0);
    }

    #[test]
    fn trackable_partial_scores() {
        // 50% + 75% + 80% = 205% / 300% = 0.683
        let metrics = GitMetrics::new_with_percentages(50.0, 75.0, 80.0);
        let expected = (50.0 + 75.0 + 80.0) / 300.0;
        assert!((compute_trackable(Some(&metrics)) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn trackable_no_data_neutral() {
        assert_eq!(compute_trackable(None), 0.5);
    }

    // Integration tests for ScoringEngine trait

    #[test]
    fn scoring_engine_all_metrics_perfect() {
        let engine = DefaultHeuristicEngine;
        let score = engine.compute_score(
            Some(&lsp_clean()),
            Some(&test_perfect()),
            Some(&git_perfect()),
            Some(&security_clean()),
        );

        assert_eq!(score.tested, 1.0);
        assert_eq!(score.readable, 1.0);
        assert_eq!(score.unified, 1.0);
        assert_eq!(score.secured, 1.0);
        assert_eq!(score.trackable, 1.0);
        assert_eq!(score.overall(), 1.0);
    }

    #[test]
    fn scoring_engine_no_metrics_neutral() {
        let engine = DefaultHeuristicEngine;
        let score = engine.compute_score(None, None, None, None);

        // All dimensions should be neutral (0.5)
        assert_eq!(score.tested, 0.5);
        assert_eq!(score.readable, 0.5);
        assert_eq!(score.unified, 0.5);
        assert_eq!(score.secured, 0.5);
        assert_eq!(score.trackable, 0.5);
        assert_eq!(score.overall(), 0.5);
    }

    #[test]
    fn scoring_engine_partial_metrics() {
        let engine = DefaultHeuristicEngine;
        let score = engine.compute_score(Some(&lsp_clean()), None, Some(&git_perfect()), None);

        assert_eq!(score.tested, 0.5); // No data
        assert_eq!(score.readable, 1.0); // Clean LSP
        assert_eq!(score.unified, 1.0); // Clean LSP
        assert_eq!(score.secured, 0.5); // No data
        assert_eq!(score.trackable, 1.0); // Perfect git
    }

    #[test]
    fn scoring_engine_realistic_mixed_scores() {
        let engine = DefaultHeuristicEngine;

        // Realistic scenario with some issues
        let lsp = LspMetrics::new_with_counts(0, 3, 5, 1000, 50); // 3 lint errors
        let test = TestMetrics::new_with_coverage(75.0, 15, 5); // 75% coverage
        let git = GitMetrics::new_with_percentages(80.0, 70.0, 60.0); // Good but not perfect
        let security = SecurityMetrics::new_with_vulns(0, 0, 2, 5, true); // 2 medium, 5 low

        let score = engine.compute_score(Some(&lsp), Some(&test), Some(&git), Some(&security));

        assert_eq!(score.tested, 0.75); // 75% / 100

        // Readable: 1000 lines → denom = 10, penalty = 3/10 = 0.3 → 0.7
        assert!((score.readable - 0.7).abs() < f32::EPSILON);

        assert_eq!(score.unified, 1.0); // No type errors

        // Secured: 2*0.1 + 5*0.05 = 0.2 + 0.25 = 0.45 → 0.55
        assert!((score.secured - 0.55).abs() < f32::EPSILON);

        // Trackable: (80 + 70 + 60) / 300 = 210 / 300 = 0.7
        assert!((score.trackable - 0.7).abs() < f32::EPSILON);

        // Overall: (0.75 + 0.7 + 1.0 + 0.55 + 0.7) / 5 = 3.7 / 5 = 0.74
        let expected_overall = (0.75 + 0.7 + 1.0 + 0.55 + 0.7) / 5.0;
        assert!((score.overall() - expected_overall).abs() < f32::EPSILON);
    }
}
