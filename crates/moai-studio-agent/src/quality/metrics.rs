//! Metric snapshot types (SPEC-V3-017 RG-QD-2)
//!
//! Input data structures for computing TRUST 5 scores.
//! These snapshots represent the state of various metric sources
//! (LSP, test runner, git, security scanner) at a point in time.

use serde::{Deserialize, Serialize};

/// LSP diagnostic metrics (REQ-QD-005).
///
/// Captures type errors, lint errors, and warnings from LSP.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LspMetrics {
    /// Number of type errors
    pub type_errors: u32,
    /// Number of lint errors (clippy warnings, etc.)
    pub lint_errors: u32,
    /// Number of warnings
    pub warnings: u32,
    /// Total number of lines in the project (for normalization)
    pub total_lines: u32,
    /// Total number of files in the project (for normalization)
    pub total_files: u32,
}

impl LspMetrics {
    /// Create new LSP metrics with all counts zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create LSP metrics with specified counts.
    pub fn new_with_counts(
        type_errors: u32,
        lint_errors: u32,
        warnings: u32,
        total_lines: u32,
        total_files: u32,
    ) -> Self {
        Self {
            type_errors,
            lint_errors,
            warnings,
            total_lines,
            total_files,
        }
    }
}

/// Test coverage metrics (REQ-QD-006).
///
/// Captures coverage percentage and test pass/fail rates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Actual coverage percentage (0.0 to 100.0)
    pub coverage_percent: f32,
    /// Number of passing tests
    pub pass_count: u32,
    /// Number of failing tests
    pub fail_count: u32,
    /// Total number of tests
    pub total_count: u32,
}

impl TestMetrics {
    /// Create new test metrics with all counts zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create test metrics with specified values.
    pub fn new_with_coverage(coverage_percent: f32, pass_count: u32, fail_count: u32) -> Self {
        Self {
            coverage_percent,
            pass_count,
            fail_count,
            total_count: pass_count + fail_count,
        }
    }

    /// Check if all tests pass (fail_count == 0).
    pub fn all_pass(&self) -> bool {
        self.fail_count == 0
    }

    /// Test pass rate as a fraction (0.0 to 1.0).
    pub fn pass_rate(&self) -> f32 {
        if self.total_count == 0 {
            return 1.0; // No tests means 100% pass rate
        }
        self.pass_count as f32 / self.total_count as f32
    }
}

/// Git history metrics (REQ-QD-007).
///
/// Captures conventional commit compliance, SPEC reference coverage,
/// and MX tag coverage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitMetrics {
    /// Percentage of commits following conventional commit format (0.0 to 100.0)
    pub conventional_commit_pct: f32,
    /// Percentage of commits referencing a SPEC (0.0 to 100.0)
    pub spec_ref_pct: f32,
    /// Percentage of high-fan-in functions with @MX tags (0.0 to 100.0)
    pub mx_tag_pct: f32,
    /// Total number of commits analyzed
    pub total_commits: u32,
}

impl GitMetrics {
    /// Create new git metrics with all percentages zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create git metrics with specified percentages.
    ///
    /// All percentages are in range [0.0, 100.0].
    pub fn new_with_percentages(
        conventional_commit_pct: f32,
        spec_ref_pct: f32,
        mx_tag_pct: f32,
    ) -> Self {
        Self {
            conventional_commit_pct,
            spec_ref_pct,
            mx_tag_pct,
            total_commits: 0,
        }
    }
}

/// Security scan metrics (REQ-QD-008).
///
/// Captures vulnerability counts by severity and audit status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Number of critical vulnerabilities
    pub critical_vulns: u32,
    /// Number of high-severity vulnerabilities
    pub high_vulns: u32,
    /// Number of medium-severity vulnerabilities
    pub medium_vulns: u32,
    /// Number of low-severity vulnerabilities
    pub low_vulns: u32,
    /// Whether security audit passed (e.g., cargo audit, OWASP check)
    pub audit_pass: bool,
}

impl SecurityMetrics {
    /// Create new security metrics with no vulnerabilities and audit passing.
    pub fn new() -> Self {
        Self {
            critical_vulns: 0,
            high_vulns: 0,
            medium_vulns: 0,
            low_vulns: 0,
            audit_pass: true,
        }
    }

    /// Create security metrics with specified vulnerability counts.
    pub fn new_with_vulns(
        critical_vulns: u32,
        high_vulns: u32,
        medium_vulns: u32,
        low_vulns: u32,
        audit_pass: bool,
    ) -> Self {
        Self {
            critical_vulns,
            high_vulns,
            medium_vulns,
            low_vulns,
            audit_pass,
        }
    }

    /// Check if there are any critical or high vulnerabilities.
    pub fn has_critical_or_high(&self) -> bool {
        self.critical_vulns > 0 || self.high_vulns > 0
    }

    /// Total vulnerability count across all severities.
    pub fn total_vulns(&self) -> u32 {
        self.critical_vulns + self.high_vulns + self.medium_vulns + self.low_vulns
    }
}

// ================================================================
// Tests
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // LspMetrics tests

    #[test]
    fn lsp_metrics_new_creates_zeros() {
        let metrics = LspMetrics::new();
        assert_eq!(metrics.type_errors, 0);
        assert_eq!(metrics.lint_errors, 0);
        assert_eq!(metrics.warnings, 0);
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.total_files, 0);
    }

    #[test]
    fn lsp_metrics_new_with_counts() {
        let metrics = LspMetrics::new_with_counts(5, 10, 15, 1000, 50);
        assert_eq!(metrics.type_errors, 5);
        assert_eq!(metrics.lint_errors, 10);
        assert_eq!(metrics.warnings, 15);
        assert_eq!(metrics.total_lines, 1000);
        assert_eq!(metrics.total_files, 50);
    }

    // TestMetrics tests

    #[test]
    fn test_metrics_new_creates_zeros() {
        let metrics = TestMetrics::new();
        assert_eq!(metrics.coverage_percent, 0.0);
        assert_eq!(metrics.pass_count, 0);
        assert_eq!(metrics.fail_count, 0);
        assert_eq!(metrics.total_count, 0);
    }

    #[test]
    fn test_metrics_new_with_coverage() {
        let metrics = TestMetrics::new_with_coverage(85.5, 17, 3);
        assert_eq!(metrics.coverage_percent, 85.5);
        assert_eq!(metrics.pass_count, 17);
        assert_eq!(metrics.fail_count, 3);
        assert_eq!(metrics.total_count, 20);
    }

    #[test]
    fn test_metrics_all_pass() {
        let metrics = TestMetrics::new_with_coverage(100.0, 10, 0);
        assert!(metrics.all_pass());

        let metrics_with_fail = TestMetrics::new_with_coverage(90.0, 9, 1);
        assert!(!metrics_with_fail.all_pass());
    }

    #[test]
    fn test_metrics_pass_rate() {
        let metrics = TestMetrics::new_with_coverage(80.0, 16, 4);
        assert!((metrics.pass_rate() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_metrics_pass_rate_no_tests() {
        let metrics = TestMetrics::new();
        assert_eq!(metrics.pass_rate(), 1.0); // No tests = 100% pass rate
    }

    // GitMetrics tests

    #[test]
    fn git_metrics_new_creates_zeros() {
        let metrics = GitMetrics::new();
        assert_eq!(metrics.conventional_commit_pct, 0.0);
        assert_eq!(metrics.spec_ref_pct, 0.0);
        assert_eq!(metrics.mx_tag_pct, 0.0);
        assert_eq!(metrics.total_commits, 0);
    }

    #[test]
    fn git_metrics_new_with_percentages() {
        let metrics = GitMetrics::new_with_percentages(75.0, 85.5, 90.0);
        assert_eq!(metrics.conventional_commit_pct, 75.0);
        assert_eq!(metrics.spec_ref_pct, 85.5);
        assert_eq!(metrics.mx_tag_pct, 90.0);
    }

    // SecurityMetrics tests

    #[test]
    fn security_metrics_new_creates_clean() {
        let metrics = SecurityMetrics::new();
        assert_eq!(metrics.critical_vulns, 0);
        assert_eq!(metrics.high_vulns, 0);
        assert_eq!(metrics.medium_vulns, 0);
        assert_eq!(metrics.low_vulns, 0);
        assert!(metrics.audit_pass);
    }

    #[test]
    fn security_metrics_new_with_vulns() {
        let metrics = SecurityMetrics::new_with_vulns(1, 2, 5, 10, false);
        assert_eq!(metrics.critical_vulns, 1);
        assert_eq!(metrics.high_vulns, 2);
        assert_eq!(metrics.medium_vulns, 5);
        assert_eq!(metrics.low_vulns, 10);
        assert!(!metrics.audit_pass);
    }

    #[test]
    fn security_metrics_has_critical_or_high() {
        let clean = SecurityMetrics::new();
        assert!(!clean.has_critical_or_high());

        let with_critical = SecurityMetrics::new_with_vulns(1, 0, 0, 0, true);
        assert!(with_critical.has_critical_or_high());

        let with_high = SecurityMetrics::new_with_vulns(0, 1, 0, 0, true);
        assert!(with_high.has_critical_or_high());

        let with_medium_only = SecurityMetrics::new_with_vulns(0, 0, 1, 0, true);
        assert!(!with_medium_only.has_critical_or_high());
    }

    #[test]
    fn security_metrics_total_vulns() {
        let metrics = SecurityMetrics::new_with_vulns(1, 2, 5, 10, false);
        assert_eq!(metrics.total_vulns(), 18);
    }

    // Serde round-trip tests

    #[test]
    fn lsp_metrics_serde_round_trip() {
        let original = LspMetrics::new_with_counts(5, 10, 15, 1000, 50);
        let json = serde_json::to_string(&original).expect("serialize failed");
        let decoded: LspMetrics = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(decoded.type_errors, original.type_errors);
        assert_eq!(decoded.lint_errors, original.lint_errors);
    }

    #[test]
    fn test_metrics_serde_round_trip() {
        let original = TestMetrics::new_with_coverage(85.5, 17, 3);
        let json = serde_json::to_string(&original).expect("serialize failed");
        let decoded: TestMetrics = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(decoded.coverage_percent, original.coverage_percent);
    }

    #[test]
    fn git_metrics_serde_round_trip() {
        let original = GitMetrics::new_with_percentages(75.0, 85.5, 90.0);
        let json = serde_json::to_string(&original).expect("serialize failed");
        let decoded: GitMetrics = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(
            decoded.conventional_commit_pct,
            original.conventional_commit_pct
        );
    }

    #[test]
    fn security_metrics_serde_round_trip() {
        let original = SecurityMetrics::new_with_vulns(1, 2, 5, 10, false);
        let json = serde_json::to_string(&original).expect("serialize failed");
        let decoded: SecurityMetrics = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(decoded.critical_vulns, original.critical_vulns);
        assert_eq!(decoded.audit_pass, original.audit_pass);
    }
}
