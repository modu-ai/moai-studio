// URL validation and sanitization for SPEC-V3-007 MS-2
//
// This module provides URL validation to prevent security issues
// like javascript: and data: scheme injection.

use std::fmt;

/// URL validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlValidationError {
    /// Blocked unsafe scheme (javascript:, data:, vbscript:)
    UnsafeScheme(String),
    /// Empty URL string
    EmptyUrl,
    /// Invalid URL format
    InvalidFormat,
}

impl fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlValidationError::UnsafeScheme(scheme) => {
                write!(f, "Blocked unsafe scheme: {}", scheme)
            }
            UrlValidationError::EmptyUrl => write!(f, "URL cannot be empty"),
            UrlValidationError::InvalidFormat => write!(f, "Invalid URL format"),
        }
    }
}

/// Validate and sanitize a URL string
///
/// # Security
/// - Blocks `javascript:`, `data:`, `vbscript:` schemes
/// - Only allows `http:` and `https:` schemes
/// - Prepends `https://` if no scheme present
///
/// # Arguments
/// * `url` - URL string to validate
///
/// # Returns
/// * `Ok(String)` - Sanitized URL with scheme
/// * `Err(UrlValidationError)` - Validation error with reason
///
/// # Examples
/// ```
/// use moai_studio_ui::web::url::validate_url;
///
/// // HTTPS URL passes
/// assert!(validate_url("https://example.com").is_ok());
///
/// // HTTP URL passes
/// assert!(validate_url("http://example.com").is_ok());
///
/// // No scheme - https:// is prepended
/// assert_eq!(validate_url("example.com").unwrap(), "https://example.com");
///
/// // javascript: blocked
/// assert!(validate_url("javascript:alert(1)").is_err());
///
/// // Empty string blocked
/// assert!(validate_url("").is_err());
/// ```
pub fn validate_url(url: &str) -> Result<String, UrlValidationError> {
    // Check empty
    let url = url.trim();
    if url.is_empty() {
        return Err(UrlValidationError::EmptyUrl);
    }

    // Check for unsafe schemes
    let url_lower = url.to_lowercase();
    if url_lower.starts_with("javascript:") {
        return Err(UrlValidationError::UnsafeScheme("javascript:".to_string()));
    }
    if url_lower.starts_with("data:") {
        return Err(UrlValidationError::UnsafeScheme("data:".to_string()));
    }
    if url_lower.starts_with("vbscript:") {
        return Err(UrlValidationError::UnsafeScheme("vbscript:".to_string()));
    }

    // Detect scheme via :// pattern (avoids false positives on host:port)
    if let Some(scheme_end) = url.find("://") {
        let scheme = &url[..scheme_end];
        let scheme_lower = scheme.to_lowercase();
        if scheme_lower == "http" || scheme_lower == "https" {
            Ok(url.to_string())
        } else {
            Err(UrlValidationError::UnsafeScheme(format!("{}:", scheme)))
        }
    } else {
        // No scheme detected — prepend https://
        Ok(format!("https://{}", url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_https_url_passes() {
        let result = validate_url("https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com");
    }

    #[test]
    fn test_validate_http_url_passes() {
        let result = validate_url("http://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com");
    }

    #[test]
    fn test_validate_javascript_scheme_blocked() {
        let result = validate_url("javascript:alert('XSS')");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UrlValidationError::UnsafeScheme("javascript:".to_string()));
    }

    #[test]
    fn test_validate_data_scheme_blocked() {
        let result = validate_url("data:text/html,<script>alert('XSS')</script>");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UrlValidationError::UnsafeScheme("data:".to_string()));
    }

    #[test]
    fn test_validate_vbscript_scheme_blocked() {
        let result = validate_url("vbscript:msgbox('XSS')");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UrlValidationError::UnsafeScheme("vbscript:".to_string()));
    }

    #[test]
    fn test_validate_no_scheme_prepends_https() {
        let result = validate_url("example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com");
    }

    #[test]
    fn test_validate_empty_url_fails() {
        let result = validate_url("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UrlValidationError::EmptyUrl);
    }

    #[test]
    fn test_validate_whitespace_only_fails() {
        let result = validate_url("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UrlValidationError::EmptyUrl);
    }

    #[test]
    fn test_validate_unsafe_scheme_blocked() {
        let result = validate_url("file:///etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlValidationError::UnsafeScheme(scheme) => {
                assert_eq!(scheme, "file:");
            }
            _ => panic!("Expected UnsafeScheme error"),
        }
    }

    #[test]
    fn test_validate_fttp_scheme_blocked() {
        let result = validate_url("fttp://example.com");
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlValidationError::UnsafeScheme(scheme) => {
                assert_eq!(scheme, "fttp:");
            }
            _ => panic!("Expected UnsafeScheme error"),
        }
    }

    #[test]
    fn test_validate_host_port_prepends_https() {
        // host:port should not be treated as a scheme
        let result = validate_url("example.com:8080");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com:8080");
    }

    #[test]
    fn test_validate_localhost_port_prepends_https() {
        let result = validate_url("localhost:3000");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://localhost:3000");
    }
}
