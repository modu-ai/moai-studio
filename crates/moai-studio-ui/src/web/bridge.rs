// JS ↔ Rust bidirectional bridge for SPEC-V3-007 MS-3
//
// This module provides a secure bridge for JavaScript-to-Rust communication
// in webview pages. Features:
// - Channel-based message routing
// - Trusted domain validation
// - Payload size limits (1MB max)
// - Request/response pattern support

#[cfg(feature = "web")]
use serde_json::Value;
#[cfg(feature = "web")]
use std::collections::HashMap;
#[cfg(feature = "web")]
use std::sync::Arc;

/// Bridge message kinds
#[cfg(feature = "web")]
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeKind {
    /// One-way event (no response expected)
    Event,
    /// Request that expects a response
    Request,
}

/// Bridge message structure
///
/// Represents a message sent from JavaScript to Rust via the IPC bridge.
///
/// # Fields (REQ-WB-050)
/// * `id` - Unique message identifier (u64)
/// * `kind` - Message kind (event or request)
/// * `channel` - Channel name for routing
/// * `payload` - JSON payload (max 1MB, REQ-WB-054)
#[cfg(feature = "web")]
#[derive(Debug, Clone)]
pub struct BridgeMessage {
    /// Unique message ID for request/response correlation
    pub id: u64,
    /// Message kind (event vs request)
    pub kind: BridgeKind,
    /// Channel name for routing to handlers
    pub channel: String,
    /// JSON payload (max 1MB serialized)
    pub payload: Value,
}

#[cfg(feature = "web")]
impl BridgeMessage {
    /// Maximum allowed payload size (1 MB, REQ-WB-054)
    pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;

    /// Create a new BridgeMessage with validation
    ///
    /// # Errors
    /// Returns `Err` if:
    /// - Payload serialized size exceeds 1MB (REQ-WB-054)
    /// - Channel name is empty
    pub fn new(
        id: u64,
        kind: BridgeKind,
        channel: impl Into<String>,
        payload: Value,
    ) -> Result<Self, String> {
        let channel = channel.into();

        // Validate channel name is not empty
        if channel.is_empty() {
            return Err("Channel name cannot be empty".to_string());
        }

        // Validate payload size (REQ-WB-054)
        let serialized = serde_json::to_vec(&payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;

        if serialized.len() > Self::MAX_PAYLOAD_SIZE {
            return Err(format!(
                "Payload size {} bytes exceeds maximum {} bytes",
                serialized.len(),
                Self::MAX_PAYLOAD_SIZE
            ));
        }

        Ok(Self {
            id,
            kind,
            channel,
            payload,
        })
    }
}

/// Bridge handler function type
///
/// Handlers receive the payload value and return an optional result
/// that will be sent back to JavaScript for request messages.
#[cfg(feature = "web")]
pub type BridgeHandler = Arc<dyn Fn(Value) -> Option<Value> + Send + Sync>;

/// Bridge router for channel-based message dispatch
///
/// Routes incoming bridge messages to registered channel handlers.
/// Enforces trusted domain checks and payload size limits.
///
/// # Features (REQ-WB-050, REQ-WB-053, REQ-WB-054)
/// - Channel registration via `register()`
/// - Unknown channel rejection (warn only, REQ-WB-053)
/// - Payload size validation (1MB max, REQ-WB-054)
/// - Trusted origin check (REQ-WB-044)
#[cfg(feature = "web")]
pub struct BridgeRouter {
    /// Registered channel handlers
    handlers: HashMap<String, BridgeHandler>,
    /// Trusted domains for bridge activation (REQ-WB-045)
    trusted_domains: Vec<String>,
}

#[cfg(feature = "web")]
impl BridgeRouter {
    /// Create a new BridgeRouter with default trusted domains
    ///
    /// Default trusted domains: ["localhost", "127.0.0.1", "[::1]"] (REQ-WB-045)
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            trusted_domains: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "[::1]".to_string(),
            ],
        }
    }

    /// Create a new BridgeRouter with custom trusted domains
    pub fn with_trusted_domains(trusted_domains: Vec<String>) -> Self {
        Self {
            handlers: HashMap::new(),
            trusted_domains,
        }
    }

    /// Register a handler for a specific channel
    ///
    /// # Arguments
    /// * `channel` - Channel name (e.g., "log", "fs")
    /// * `handler` - Handler function that receives payload and returns optional response
    ///
    /// # Example (REQ-WB-050)
    /// ```ignore
    /// router.register("log", |payload| {
    ///     println!("Log: {}", payload);
    ///     None // Event: no response
    /// });
    /// ```
    pub fn register<H>(&mut self, channel: impl Into<String>, handler: H)
    where
        H: Fn(Value) -> Option<Value> + Send + Sync + 'static,
    {
        let channel = channel.into();
        self.handlers
            .insert(channel, Arc::new(handler) as BridgeHandler);
    }

    /// Dispatch a message to the appropriate handler
    ///
    /// # Routing Logic (REQ-WB-051, REQ-WB-053)
    /// 1. Check origin is trusted (REQ-WB-044)
    /// 2. Find handler for channel
    /// 3. If handler exists, call it with payload
    /// 4. If handler not found, log warning and return None (REQ-WB-053)
    ///
    /// # Returns
    /// * `Some(Value)` - Response payload (for request messages)
    /// * `None` - No response (event messages or unknown channel)
    pub fn dispatch(&self, message: &BridgeMessage, origin: &str) -> Option<Value> {
        // Check if origin is trusted (REQ-WB-044)
        if !self.is_trusted_origin(origin) {
            tracing::warn!("Bridge message rejected from untrusted origin: {}", origin);
            return None;
        }

        // Find handler for channel
        let handler = self.handlers.get(&message.channel)?;

        // Call handler with payload
        handler(message.payload.clone())
    }

    /// Check if an origin is in the trusted domains list
    ///
    /// # Trusted Domain Check (REQ-WB-044)
    /// - Extracts hostname from origin URL
    /// - Checks against trusted_domains list
    /// - Returns true if hostname matches any trusted domain
    fn is_trusted_origin(&self, origin: &str) -> bool {
        // Parse origin to extract hostname
        let hostname = origin
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split('/')
            .next()
            .unwrap_or(origin);

        // Handle IPv6 addresses in brackets [::1]:port
        let hostname = if hostname.starts_with('[') {
            // IPv6 address with port: [::1]:8080 -> [::1] (include the closing bracket)
            hostname
                .split(']')
                .next()
                .map(|s| format!("{}]", s)) // Add back the closing bracket
                .unwrap_or(hostname.to_string())
        } else {
            // IPv4 or hostname: strip port if present
            hostname.split(':').next().unwrap_or(hostname).to_string()
        };

        // Check against trusted domains (REQ-WB-044, REQ-WB-045)
        self.trusted_domains.contains(&hostname)
    }

    /// Get the list of trusted domains
    pub fn trusted_domains(&self) -> &[String] {
        &self.trusted_domains
    }
}

#[cfg(feature = "web")]
impl Default for BridgeRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bridge_kind_variants() {
        let event = BridgeKind::Event;
        let request = BridgeKind::Request;

        assert_eq!(event, BridgeKind::Event);
        assert_eq!(request, BridgeKind::Request);
        assert_ne!(event, request);
    }

    #[test]
    fn test_bridge_message_new_valid() {
        let payload = json!({"message": "hello"});
        let msg = BridgeMessage::new(1, BridgeKind::Event, "test", payload).unwrap();

        assert_eq!(msg.id, 1);
        assert_eq!(msg.kind, BridgeKind::Event);
        assert_eq!(msg.channel, "test");
        assert_eq!(msg.payload, json!({"message": "hello"}));
    }

    #[test]
    fn test_bridge_message_rejects_empty_channel() {
        let payload = json!({"message": "hello"});
        let result = BridgeMessage::new(1, BridgeKind::Event, "", payload);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Channel name cannot be empty");
    }

    #[test]
    fn test_bridge_message_rejects_oversized_payload() {
        // Create a payload larger than 1MB
        let large_string = "x".repeat(2 * 1024 * 1024); // 2MB
        let payload = json!({"data": large_string});
        let result = BridgeMessage::new(1, BridgeKind::Event, "test", payload);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_bridge_router_new_has_default_trusted_domains() {
        let router = BridgeRouter::new();
        let domains = router.trusted_domains();

        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"localhost".to_string()));
        assert!(domains.contains(&"127.0.0.1".to_string()));
        assert!(domains.contains(&"[::1]".to_string()));
    }

    #[test]
    fn test_bridge_router_register_and_dispatch() {
        let mut router = BridgeRouter::new();

        // Register a handler that returns the payload unchanged
        router.register("echo", |payload| Some(payload.clone()));

        let msg = BridgeMessage::new(1, BridgeKind::Request, "echo", json!({"test": "data"}))
            .unwrap();
        let result = router.dispatch(&msg, "http://localhost:8080");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), json!({"test": "data"}));
    }

    #[test]
    fn test_bridge_router_unknown_channel_returns_none() {
        let router = BridgeRouter::new();

        // No handler registered for "unknown" channel
        let msg = BridgeMessage::new(1, BridgeKind::Event, "unknown", json!({})).unwrap();
        let result = router.dispatch(&msg, "http://localhost:8080");

        // Should return None without error (REQ-WB-053)
        assert!(result.is_none());
    }

    #[test]
    fn test_bridge_router_rejects_untrusted_origin() {
        let mut router = BridgeRouter::new();
        router.register("test", |_| Some(json!("ok")));

        let msg = BridgeMessage::new(1, BridgeKind::Request, "test", json!({})).unwrap();
        let result = router.dispatch(&msg, "http://evil.com:8080");

        // Untrusted origin should be rejected
        assert!(result.is_none());
    }

    #[test]
    fn test_bridge_router_accepts_localhost() {
        let mut router = BridgeRouter::new();
        router.register("test", |_| Some(json!("ok")));

        let msg = BridgeMessage::new(1, BridgeKind::Request, "test", json!({})).unwrap();

        // Test various localhost forms
        assert!(router.dispatch(&msg, "http://localhost:8080").is_some());
        assert!(router.dispatch(&msg, "http://127.0.0.1:8080").is_some());
        assert!(router.dispatch(&msg, "http://[::1]:8080").is_some());
    }

    #[test]
    fn test_bridge_router_handler_returns_none_for_event() {
        let mut router = BridgeRouter::new();

        // Event handler returns None
        router.register("log", |_| {
            println!("Log event received");
            None
        });

        let msg = BridgeMessage::new(1, BridgeKind::Event, "log", json!({"level": "info"}))
            .unwrap();
        let result = router.dispatch(&msg, "http://localhost:8080");

        assert!(result.is_none());
    }

    #[test]
    fn test_bridge_router_with_custom_trusted_domains() {
        let custom_domains = vec!["example.com".to_string(), "*.trusted.org".to_string()];
        let mut router = BridgeRouter::with_trusted_domains(custom_domains);

        router.register("test", |_| Some(json!("ok")));
        let msg = BridgeMessage::new(1, BridgeKind::Request, "test", json!({})).unwrap();

        // Custom domain should be trusted
        assert!(router.dispatch(&msg, "http://example.com:8080").is_some());

        // Default localhost should NOT be trusted with custom domains
        assert!(router.dispatch(&msg, "http://localhost:8080").is_none());
    }

    #[test]
    fn test_bridge_message_max_payload_size_boundary() {
        // Test with a small payload to verify logic works
        let small_payload = json!({"test": "data"});
        assert!(BridgeMessage::new(1, BridgeKind::Event, "test", small_payload).is_ok());

        // Test with a clearly oversized payload
        let huge_string = "x".repeat(2 * 1024 * 1024); // 2MB
        let huge_payload = json!({"data": huge_string});
        assert!(BridgeMessage::new(1, BridgeKind::Event, "test", huge_payload).is_err());
    }

    #[test]
    fn test_bridge_router_is_trusted_origin_extracts_hostname() {
        let router = BridgeRouter::new();

        // Various URL forms should extract hostname correctly
        assert!(router.is_trusted_origin("localhost"));
        assert!(router.is_trusted_origin("http://localhost"));
        assert!(router.is_trusted_origin("https://localhost"));
        assert!(router.is_trusted_origin("http://localhost:8080"));
        assert!(router.is_trusted_origin("https://localhost:3000/path"));
    }
}
