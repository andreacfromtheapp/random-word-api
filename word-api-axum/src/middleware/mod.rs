//! Middleware configuration and setup
//!
//! This module provides middleware components for the API including:
//! - Rate limiting per IP address
//! - Security headers
//! - Request body size limits
//!
//! All middleware is configured via the `ApiConfig` and applied globally.

pub mod limits;
pub mod rate_limit;
pub mod security;

/// Re-export middleware components for easy access
pub use limits::create_body_limit_layer;
pub use rate_limit::{get_burst_size, get_per_second, is_enabled};
pub use security::security_headers;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;

    #[test]
    fn test_middleware_modules_exist() {
        // Simple test to ensure all modules are properly exported
        let config = ApiConfig::default();

        // Test that rate limit functions work
        let _enabled = is_enabled(&config);
        let _per_second = get_per_second(&config);
        let _burst_size = get_burst_size(&config);

        // Test that body limit layer can be created
        let _body_limit = create_body_limit_layer();

        // Test that security_headers function exists
        // Just verify the module exports work correctly
        assert!(true);
    }
}
