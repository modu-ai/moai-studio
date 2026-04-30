// Web configuration for SPEC-V3-007 MS-3
//
// This module provides configuration for webview behavior.
// Future: load from .moai/config/sections/web.yaml

use serde::{Deserialize, Serialize};

/// WebView configuration
///
/// Configuration settings for webview behavior.
///
/// # Fields
/// * `trusted_domains` - Domains allowed for JS bridge (REQ-WB-045)
/// * `devtools_enabled` - Whether DevTools is enabled
/// * `max_concurrent_webviews` - Maximum concurrent webview tabs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebConfig {
    /// Trusted domains for JS bridge (default: localhost, 127.0.0.1, [::1])
    /// REQ-WB-045
    #[serde(default = "WebConfig::default_trusted_domains")]
    pub trusted_domains: Vec<String>,

    /// Whether DevTools is enabled (default: true)
    #[serde(default = "WebConfig::default_devtools_enabled")]
    pub devtools_enabled: bool,

    /// Maximum concurrent webview tabs (default: 10)
    #[serde(default = "WebConfig::default_max_concurrent_webviews")]
    pub max_concurrent_webviews: usize,
}

impl WebConfig {
    /// Get default trusted domains (REQ-WB-045)
    fn default_trusted_domains() -> Vec<String> {
        vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "[::1]".to_string(),
        ]
    }

    /// Get default DevTools enabled setting
    fn default_devtools_enabled() -> bool {
        true
    }

    /// Get default max concurrent webviews
    fn default_max_concurrent_webviews() -> usize {
        10
    }

    /// Create a new WebConfig with default settings
    pub fn new() -> Self {
        Self {
            trusted_domains: Self::default_trusted_domains(),
            devtools_enabled: Self::default_devtools_enabled(),
            max_concurrent_webviews: Self::default_max_concurrent_webviews(),
        }
    }

    /// Create a new WebConfig with custom trusted domains
    pub fn with_trusted_domains(mut self, domains: Vec<String>) -> Self {
        self.trusted_domains = domains;
        self
    }

    /// Create a new WebConfig with DevTools enabled/disabled
    pub fn with_devtools(mut self, enabled: bool) -> Self {
        self.devtools_enabled = enabled;
        self
    }

    /// Create a new WebConfig with max concurrent webviews
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_webviews = max;
        self
    }

    /// Check if a domain is trusted
    pub fn is_trusted_domain(&self, domain: &str) -> bool {
        self.trusted_domains.contains(&domain.to_string())
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_config_default() {
        let config = WebConfig::new();

        assert_eq!(config.trusted_domains.len(), 3);
        assert!(config.trusted_domains.contains(&"localhost".to_string()));
        assert!(config.trusted_domains.contains(&"127.0.0.1".to_string()));
        assert!(config.trusted_domains.contains(&"[::1]".to_string()));
        assert_eq!(config.devtools_enabled, true);
        assert_eq!(config.max_concurrent_webviews, 10);
    }

    #[test]
    fn test_web_config_with_trusted_domains() {
        let config = WebConfig::new().with_trusted_domains(vec!["example.com".to_string()]);

        assert_eq!(config.trusted_domains.len(), 1);
        assert_eq!(config.trusted_domains[0], "example.com");
    }

    #[test]
    fn test_web_config_with_devtools() {
        let config = WebConfig::new().with_devtools(false);

        assert_eq!(config.devtools_enabled, false);
    }

    #[test]
    fn test_web_config_with_max_concurrent() {
        let config = WebConfig::new().with_max_concurrent(5);

        assert_eq!(config.max_concurrent_webviews, 5);
    }

    #[test]
    fn test_web_config_is_trusted_domain() {
        let config = WebConfig::new();

        assert!(config.is_trusted_domain("localhost"));
        assert!(config.is_trusted_domain("127.0.0.1"));
        assert!(config.is_trusted_domain("[::1]"));
        assert!(!config.is_trusted_domain("evil.com"));
    }

    #[test]
    fn test_web_config_default_trait() {
        let config: WebConfig = Default::default();

        assert_eq!(config.trusted_domains.len(), 3);
        assert_eq!(config.devtools_enabled, true);
        assert_eq!(config.max_concurrent_webviews, 10);
    }

    #[test]
    fn test_web_config_chaining() {
        let config = WebConfig::new()
            .with_devtools(false)
            .with_max_concurrent(5)
            .with_trusted_domains(vec!["custom.com".to_string()]);

        assert_eq!(config.devtools_enabled, false);
        assert_eq!(config.max_concurrent_webviews, 5);
        assert_eq!(config.trusted_domains.len(), 1);
        assert_eq!(config.trusted_domains[0], "custom.com");
    }

    #[test]
    fn test_web_config_serialize_deserialize() {
        let config = WebConfig::new();

        // Serialize
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("localhost"));
        assert!(json.contains("127.0.0.1"));

        // Deserialize
        let deserialized: WebConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_web_config_serialize_with_custom_settings() {
        let config = WebConfig::new()
            .with_devtools(false)
            .with_max_concurrent(20);

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"devtools_enabled\":false"));
        assert!(json.contains("\"max_concurrent_webviews\":20"));
    }

    #[test]
    fn test_web_config_deserialize_from_json() {
        let json = r#"{
            "trusted_domains": ["example.com"],
            "devtools_enabled": false,
            "max_concurrent_webviews": 15
        }"#;

        let config: WebConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.trusted_domains.len(), 1);
        assert_eq!(config.trusted_domains[0], "example.com");
        assert_eq!(config.devtools_enabled, false);
        assert_eq!(config.max_concurrent_webviews, 15);
    }

    #[test]
    fn test_web_config_partial_deserialize_uses_defaults() {
        let json = r#"{
            "devtools_enabled": false
        }"#;

        let config: WebConfig = serde_json::from_str(json).unwrap();

        // trusted_domains should use default
        assert_eq!(config.trusted_domains.len(), 3);
        assert!(config.trusted_domains.contains(&"localhost".to_string()));

        // devtools_enabled should be from JSON
        assert_eq!(config.devtools_enabled, false);

        // max_concurrent_webviews should use default
        assert_eq!(config.max_concurrent_webviews, 10);
    }
}
