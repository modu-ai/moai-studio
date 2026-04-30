// URL auto-detection from PTY output for SPEC-V3-007 MS-3
//
// This module provides automatic detection of local dev server URLs
// from terminal output (REQ-WB-030~035).
//
// Features:
// - Regex-based detection of localhost URLs
// - Debouncer to prevent duplicate notifications (5s window, REQ-WB-035)
// - Dismissed URL silence (30 minutes, REQ-WB-035)

use regex::Regex;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Detected URL with source information
///
/// Represents a URL detected from PTY output along with the source text.
///
/// # Fields (REQ-WB-030)
/// * `url` - The detected URL string (e.g., "http://localhost:8080")
/// * `source` - The source text chunk that contained the URL
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DetectedUrl {
    /// The detected URL
    pub url: String,
    /// Source text where the URL was found
    pub source: String,
}

impl DetectedUrl {
    /// Create a new DetectedUrl
    pub fn new(url: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            source: source.into(),
        }
    }
}

/// Detect local dev server URLs from stdout text chunk
///
/// # Pattern (REQ-WB-030)
/// Matches URLs like:
/// - `http://localhost:8080`
/// - `https://127.0.0.1:3000/path`
/// - `http://[::1]:9000`
///
/// # Arguments
/// * `stdout_chunk` - A chunk of text from PTY stdout
///
/// # Returns
/// Vector of detected URLs (deduplicated within the chunk)
///
/// # Example
/// ```ignore
/// let urls = detect_local_urls("Server running at http://localhost:8080");
/// assert_eq!(urls.len(), 1);
/// assert_eq!(urls[0].url, "http://localhost:8080");
/// ```
pub fn detect_local_urls(stdout_chunk: &str) -> Vec<DetectedUrl> {
    // Regex pattern: https?://(localhost|127.0.0.1|[::1]):port(/path)?
    // REQ-WB-030: r"https?://(localhost|127\.0\.0\.1|\[::1\]):(\d+)(/[^\s]*)?"
    let re = Regex::new(r"https?://(localhost|127\.0\.0\.1|\[::1\]):(\d+)(/[^\s]*)?").unwrap();

    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for caps in re.captures_iter(stdout_chunk) {
        let url = caps.get(0).unwrap().as_str().to_string();

        // Deduplicate within the same chunk
        if seen.insert(url.clone()) {
            results.push(DetectedUrl::new(url, stdout_chunk.to_string()));
        }
    }

    results
}

/// URL detection debouncer to prevent notification spam
///
/// Enforces two deduplication rules (REQ-WB-035):
/// 1. Same URL within 5 seconds: suppress duplicate
/// 2. User-dismissed URL: silence for 30 minutes
#[derive(Debug)]
pub struct UrlDetectionDebouncer {
    /// Recently seen URLs with their first seen timestamp
    seen_urls: Vec<(String, Instant)>,
    /// User-dismissed URLs with their dismissal timestamp
    dismissed_urls: Vec<(String, Instant)>,
    /// Deduplication window (5 seconds, REQ-WB-035)
    dedupe_window: Duration,
    /// Silence duration for dismissed URLs (30 minutes, REQ-WB-035)
    dismissed_silence: Duration,
}

impl UrlDetectionDebouncer {
    /// Create a new debouncer with default timeouts
    ///
    /// Default settings (REQ-WB-035):
    /// - dedupe_window: 5 seconds
    /// - dismissed_silence: 30 minutes
    pub fn new() -> Self {
        Self {
            seen_urls: Vec::new(),
            dismissed_urls: Vec::new(),
            dedupe_window: Duration::from_secs(5),
            dismissed_silence: Duration::from_secs(30 * 60), // 30 minutes
        }
    }

    /// Create a new debouncer with custom timeouts
    pub fn with_timeouts(dedupe_window: Duration, dismissed_silence: Duration) -> Self {
        Self {
            seen_urls: Vec::new(),
            dismissed_urls: Vec::new(),
            dedupe_window,
            dismissed_silence,
        }
    }

    /// Process detected URLs and filter out duplicates
    ///
    /// # Deduplication Rules (REQ-WB-035)
    /// 1. If URL was seen within dedupe_window (5s), suppress it
    /// 2. If URL was user-dismissed within dismissed_silence (30min), suppress it
    /// 3. Otherwise, emit the URL
    ///
    /// # Returns
    /// Vector of new URLs that should trigger notifications
    pub fn process(&mut self, urls: Vec<DetectedUrl>) -> Vec<DetectedUrl> {
        let now = Instant::now();
        let mut new_urls = Vec::new();

        // Purge expired seen entries and dismissed entries
        self.purge_expired(now);

        for detected in urls {
            // Check if dismissed (REQ-WB-035)
            if self.is_dismissed(&detected.url, now) {
                continue;
            }

            // Check if seen within dedupe_window (REQ-WB-035)
            if self.is_seen_recently(&detected.url, now) {
                continue;
            }

            // New URL: mark as seen and emit
            self.seen_urls.push((detected.url.clone(), now));
            new_urls.push(detected);
        }

        new_urls
    }

    /// Mark a URL as dismissed by the user
    ///
    /// Dismissed URLs will be silenced for 30 minutes (REQ-WB-035).
    pub fn dismiss(&mut self, url: impl Into<String>) {
        let url = url.into();
        let now = Instant::now();

        // Remove from seen_urls to allow re-emission after silence period
        self.seen_urls.retain(|(u, _)| u != &url);

        // Add to dismissed set with timestamp
        self.dismissed_urls.push((url, now));
    }

    /// Check if a URL is currently dismissed
    fn is_dismissed(&self, url: &str, now: Instant) -> bool {
        self.dismissed_urls
            .iter()
            .any(|(dismissed_url, timestamp)| {
                dismissed_url == url
                    && now.saturating_duration_since(*timestamp) < self.dismissed_silence
            })
    }

    /// Check if a URL was seen within the dedupe_window
    fn is_seen_recently(&self, url: &str, now: Instant) -> bool {
        self.seen_urls.iter().any(|(seen_url, timestamp)| {
            seen_url == url && now.saturating_duration_since(*timestamp) < self.dedupe_window
        })
    }

    /// Purge expired seen and dismissed entries
    fn purge_expired(&mut self, now: Instant) {
        self.seen_urls
            .retain(|(_, ts)| now.saturating_duration_since(*ts) < self.dedupe_window);
        self.dismissed_urls
            .retain(|(_, ts)| now.saturating_duration_since(*ts) < self.dismissed_silence);
    }

    /// Reset all seen URLs (useful for testing)
    #[cfg(test)]
    fn reset_seen(&mut self) {
        self.seen_urls.clear();
    }
}

impl Default for UrlDetectionDebouncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_url_new() {
        let detected = DetectedUrl::new("http://localhost:8080", "Server running");
        assert_eq!(detected.url, "http://localhost:8080");
        assert_eq!(detected.source, "Server running");
    }

    #[test]
    fn test_detect_local_urls_localhost() {
        let chunk = "Server running at http://localhost:8080";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://localhost:8080");
    }

    #[test]
    fn test_detect_local_urls_127_0_0_1() {
        let chunk = "Serving on http://127.0.0.1:3000";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://127.0.0.1:3000");
    }

    #[test]
    fn test_detect_local_urls_ipv6() {
        let chunk = "Listening on http://[::1]:9000";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://[::1]:9000");
    }

    #[test]
    fn test_detect_local_urls_with_path() {
        let chunk = "Visit http://localhost:8080/docs for help";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://localhost:8080/docs");
    }

    #[test]
    fn test_detect_local_urls_multiple() {
        let chunk = "Servers: http://localhost:8080 and https://127.0.0.1:3000";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].url, "http://localhost:8080");
        assert_eq!(urls[1].url, "https://127.0.0.1:3000");
    }

    #[test]
    fn test_detect_local_urls_dedup_within_chunk() {
        let chunk = "http://localhost:8080 http://localhost:8080";
        let urls = detect_local_urls(chunk);

        // Should deduplicate within the same chunk
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_detect_local_urls_no_match() {
        let chunk = "No URLs here, just plain text";
        let urls = detect_local_urls(chunk);

        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_detect_local_urls_ignores_non_localhost() {
        let chunk = "Visit https://example.com:8080 or http://192.168.1.1:3000";
        let urls = detect_local_urls(chunk);

        // Should only match localhost variants
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_debouncer_new_url_emits() {
        let mut debouncer = UrlDetectionDebouncer::new();
        let urls = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results = debouncer.process(urls);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "http://localhost:8080");
    }

    #[test]
    fn test_debouncer_duplicate_within_window_suppressed() {
        let mut debouncer = UrlDetectionDebouncer::new();

        // First URL should emit
        let urls1 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results1 = debouncer.process(urls1);
        assert_eq!(results1.len(), 1);

        // Second URL within 5s window should be suppressed
        let urls2 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results2 = debouncer.process(urls2);
        assert_eq!(results2.len(), 0);
    }

    #[test]
    fn test_debouncer_different_urls_emit() {
        let mut debouncer = UrlDetectionDebouncer::new();

        // First URL
        let urls1 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results1 = debouncer.process(urls1);
        assert_eq!(results1.len(), 1);

        // Different URL should also emit
        let urls2 = vec![DetectedUrl::new("http://localhost:3000", "Server")];
        let results2 = debouncer.process(urls2);
        assert_eq!(results2.len(), 1);
    }

    #[test]
    fn test_debouncer_dismissed_url_silenced() {
        let mut debouncer = UrlDetectionDebouncer::new();

        // First URL should emit
        let urls1 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results1 = debouncer.process(urls1);
        assert_eq!(results1.len(), 1);

        // User dismisses the URL
        debouncer.dismiss("http://localhost:8080");

        // Same URL should be silenced
        let urls2 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results2 = debouncer.process(urls2);
        assert_eq!(results2.len(), 0);
    }

    #[test]
    fn test_debouncer_dismissal_expires_after_silence_period() {
        // Create debouncer with very short silence period for testing
        let mut debouncer = UrlDetectionDebouncer::with_timeouts(
            Duration::from_secs(5),
            Duration::from_millis(100), // 100ms silence for testing
        );

        // First URL should emit
        let urls1 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results1 = debouncer.process(urls1);
        assert_eq!(results1.len(), 1);

        // User dismisses the URL
        debouncer.dismiss("http://localhost:8080");

        // Same URL should be silenced immediately
        let urls2 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results2 = debouncer.process(urls2);
        assert_eq!(results2.len(), 0);

        // Wait for silence period to expire
        std::thread::sleep(Duration::from_millis(150));

        // Reset seen to allow re-emission
        debouncer.reset_seen();

        // Same URL should emit again after silence period
        let urls3 = vec![DetectedUrl::new("http://localhost:8080", "Server")];
        let results3 = debouncer.process(urls3);
        assert_eq!(results3.len(), 1);
    }

    #[test]
    fn test_debouncer_multiple_urls_mixed() {
        let mut debouncer = UrlDetectionDebouncer::new();

        // Multiple URLs in one batch
        let urls1 = vec![
            DetectedUrl::new("http://localhost:8080", "Server"),
            DetectedUrl::new("http://localhost:3000", "Server"),
        ];
        let results1 = debouncer.process(urls1);
        assert_eq!(results1.len(), 2);

        // Duplicate one, add one new
        let urls2 = vec![
            DetectedUrl::new("http://localhost:8080", "Server"), // duplicate
            DetectedUrl::new("http://127.0.0.1:9000", "Server"), // new
        ];
        let results2 = debouncer.process(urls2);
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].url, "http://127.0.0.1:9000");
    }
}
