//! Quality history ring buffer (SPEC-V3-017 MS-3, REQ-QD-018~021)
//!
//! Stores up to 100 quality score snapshots in a ring buffer.
//! Used by HistoryView to render dimension sparklines over time.
//! NFR-QD-4: Peak memory for 100 snapshots <= 50KB.

use std::collections::VecDeque;

use super::Trust5Score;

/// Default maximum number of snapshots retained (REQ-QD-018).
pub const DEFAULT_MAX_SIZE: usize = 100;

/// Quality score snapshot with timestamp and optional commit hash.
///
/// Each snapshot records the TRUST 5 score at a specific point in time,
/// along with an optional git commit hash for traceability.
#[derive(Debug, Clone)]
pub struct QualitySnapshot {
    /// TRUST 5 dimension scores at this point in time.
    pub score: Trust5Score,
    /// Monotonic timestamp for ordering.
    pub timestamp: std::time::Instant,
    /// Optional git commit hash for traceability.
    pub commit_hash: Option<String>,
}

impl QualitySnapshot {
    /// Create a new snapshot with the given score and no commit hash.
    pub fn new(score: Trust5Score) -> Self {
        Self {
            score,
            timestamp: std::time::Instant::now(),
            commit_hash: None,
        }
    }

    /// Create a snapshot with the given score and commit hash.
    pub fn with_commit(score: Trust5Score, commit_hash: impl Into<String>) -> Self {
        Self {
            score,
            timestamp: std::time::Instant::now(),
            commit_hash: Some(commit_hash.into()),
        }
    }
}

/// Ring buffer of quality snapshots (max 100 entries, REQ-QD-018).
///
/// Iteration order is newest-first. When capacity is reached, the oldest
/// snapshot is evicted automatically.
///
/// Memory estimate (NFR-QD-4):
/// - Trust5Score: 5 * f32 = 20 bytes
/// - Instant: 12 bytes (platform-dependent)
/// - Option<String>: 32 bytes (platform-dependent)
/// - Per snapshot: ~64 bytes
/// - 100 snapshots: ~6.4 KB (well under 50 KB limit)
pub struct QualityHistory {
    snapshots: VecDeque<QualitySnapshot>,
    max_size: usize,
}

impl QualityHistory {
    /// Create an empty history with the default max size (100).
    pub fn new() -> Self {
        Self {
            snapshots: VecDeque::with_capacity(DEFAULT_MAX_SIZE),
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    /// Create an empty history with a custom max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_size),
            max_size: max_size.max(1),
        }
    }

    /// Add a snapshot to the history.
    ///
    /// If at capacity, the oldest snapshot is evicted (REQ-QD-019).
    pub fn push(&mut self, snapshot: QualitySnapshot) {
        if self.snapshots.len() >= self.max_size {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Number of snapshots in the history.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Get the most recent snapshot (newest).
    pub fn latest(&self) -> Option<&QualitySnapshot> {
        self.snapshots.back()
    }

    /// Iterate over snapshots from newest to oldest.
    pub fn iter(&self) -> impl Iterator<Item = &QualitySnapshot> {
        self.snapshots.iter().rev()
    }

    /// Extract a single dimension's scores as a time series for sparkline rendering.
    ///
    /// Returns scores ordered from newest to oldest (REQ-QD-020).
    /// `dimension_idx` must be 0-4 (T, R, U, S, K).
    pub fn dimension_trend(&self, dimension_idx: usize) -> Vec<f32> {
        let idx = dimension_idx.min(4);
        self.snapshots
            .iter()
            .rev()
            .map(|s| s.score.as_slice()[idx])
            .collect()
    }
}

impl Default for QualityHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ================================================================
// Tests (RED-GREEN-REFACTOR cycle)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_score(t: f32, r: f32, u: f32, s: f32, k: f32) -> Trust5Score {
        Trust5Score::new_with_scores(t, r, u, s, k)
    }

    /// REQ-QD-018: New history is empty.
    #[test]
    fn history_new_is_empty() {
        let history = QualityHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    /// REQ-QD-018: Push increments length.
    #[test]
    fn history_push_increments_len() {
        let mut history = QualityHistory::new();
        history.push(QualitySnapshot::new(make_score(0.5, 0.5, 0.5, 0.5, 0.5)));
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        history.push(QualitySnapshot::new(make_score(0.6, 0.6, 0.6, 0.6, 0.6)));
        assert_eq!(history.len(), 2);
    }

    /// REQ-QD-019: Push at capacity evicts oldest snapshot.
    #[test]
    fn history_push_at_capacity_evicts_oldest() {
        let mut history = QualityHistory::with_max_size(3);

        // Push 3 snapshots to fill capacity
        history.push(QualitySnapshot::new(make_score(0.1, 0.0, 0.0, 0.0, 0.0)));
        history.push(QualitySnapshot::new(make_score(0.2, 0.0, 0.0, 0.0, 0.0)));
        history.push(QualitySnapshot::new(make_score(0.3, 0.0, 0.0, 0.0, 0.0)));
        assert_eq!(history.len(), 3);

        // Push a 4th snapshot -- oldest (0.1) should be evicted
        history.push(QualitySnapshot::new(make_score(0.4, 0.0, 0.0, 0.0, 0.0)));
        assert_eq!(history.len(), 3);

        // Latest should be 0.4
        let latest = history.latest().unwrap();
        assert!((latest.score.tested - 0.4).abs() < f32::EPSILON);

        // The trend should NOT contain 0.1 (evicted)
        let trend = history.dimension_trend(0);
        assert_eq!(trend.len(), 3);
        // newest first: 0.4, 0.3, 0.2
        assert!((trend[0] - 0.4).abs() < f32::EPSILON);
        assert!((trend[1] - 0.3).abs() < f32::EPSILON);
        assert!((trend[2] - 0.2).abs() < f32::EPSILON);
    }

    /// REQ-QD-020: latest returns the most recent snapshot.
    #[test]
    fn history_latest_returns_newest() {
        let mut history = QualityHistory::new();
        assert!(history.latest().is_none());

        history.push(QualitySnapshot::new(make_score(0.1, 0.0, 0.0, 0.0, 0.0)));
        history.push(QualitySnapshot::new(make_score(0.9, 0.0, 0.0, 0.0, 0.0)));

        let latest = history.latest().unwrap();
        assert!((latest.score.tested - 0.9).abs() < f32::EPSILON);
    }

    /// REQ-QD-020: dimension_trend extracts a single dimension's scores.
    #[test]
    fn history_dimension_trend_extracts_scores() {
        let mut history = QualityHistory::new();

        history.push(QualitySnapshot::new(make_score(0.1, 0.2, 0.3, 0.4, 0.5)));
        history.push(QualitySnapshot::new(make_score(0.5, 0.6, 0.7, 0.8, 0.9)));

        // Dimension 0 (Tested): newest first → [0.5, 0.1]
        let trend_t = history.dimension_trend(0);
        assert_eq!(trend_t.len(), 2);
        assert!((trend_t[0] - 0.5).abs() < f32::EPSILON);
        assert!((trend_t[1] - 0.1).abs() < f32::EPSILON);

        // Dimension 4 (Trackable): newest first → [0.9, 0.5]
        let trend_k = history.dimension_trend(4);
        assert!((trend_k[0] - 0.9).abs() < f32::EPSILON);
        assert!((trend_k[1] - 0.5).abs() < f32::EPSILON);
    }

    /// REQ-QD-021: iter yields snapshots from newest to oldest.
    #[test]
    fn history_iter_gives_newest_first() {
        let mut history = QualityHistory::new();

        history.push(QualitySnapshot::new(make_score(0.1, 0.0, 0.0, 0.0, 0.0)));
        history.push(QualitySnapshot::new(make_score(0.2, 0.0, 0.0, 0.0, 0.0)));
        history.push(QualitySnapshot::new(make_score(0.3, 0.0, 0.0, 0.0, 0.0)));

        let scores: Vec<f32> = history.iter().map(|s| s.score.tested).collect();
        // newest first: 0.3, 0.2, 0.1
        assert_eq!(scores.len(), 3);
        assert!((scores[0] - 0.3).abs() < f32::EPSILON);
        assert!((scores[1] - 0.2).abs() < f32::EPSILON);
        assert!((scores[2] - 0.1).abs() < f32::EPSILON);
    }

    /// Snapshot with_commit stores the hash.
    #[test]
    fn snapshot_with_commit_stores_hash() {
        let snap = QualitySnapshot::with_commit(make_score(0.5, 0.5, 0.5, 0.5, 0.5), "abc1234");
        assert_eq!(snap.commit_hash.as_deref(), Some("abc1234"));
    }

    /// Default QualityHistory has max_size 100.
    #[test]
    fn history_default_max_size() {
        let history = QualityHistory::new();
        assert_eq!(history.max_size, 100);
    }
}
