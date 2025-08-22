//! Request size limits middleware implementation
//!
//! Provides request body size limiting using tower-http's RequestBodyLimitLayer.
//! Applied globally with a 1MB default limit to prevent resource exhaustion.

use tower_http::limit::RequestBodyLimitLayer;

/// Creates a RequestBodyLimitLayer with custom size limit
///
/// Allows for custom body size limits when the default 1MB is not appropriate.
///
/// # Arguments
/// * `limit_bytes` - Maximum request body size in bytes
///
/// # Returns
/// A `RequestBodyLimitLayer` configured with the specified limit
pub fn create_body_limit_layer_with_size(limit_kilobytes: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(limit_kilobytes * 1024)
}
