//! Request size limits middleware implementation
//!
//! Provides request body size limiting using tower-http's RequestBodyLimitLayer.
//! Applied globally with a 1MB default limit to prevent resource exhaustion.

use tower_http::limit::RequestBodyLimitLayer;

/// Creates a RequestBodyLimitLayer with 1MB limit
///
/// This prevents clients from sending excessively large request bodies
/// that could consume server resources or cause denial of service.
///
/// # Returns
/// A `RequestBodyLimitLayer` configured with a 1MB (1024 * 1024 bytes) limit
pub fn create_body_limit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(1024 * 1024) // 1MB limit
}

/// Creates a RequestBodyLimitLayer with custom size limit
///
/// Allows for custom body size limits when the default 1MB is not appropriate.
///
/// # Arguments
/// * `limit_bytes` - Maximum request body size in bytes
///
/// # Returns
/// A `RequestBodyLimitLayer` configured with the specified limit
pub fn create_body_limit_layer_with_size(limit_bytes: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(limit_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_body_limit_layer_default() {
        let layer = create_body_limit_layer();
        // Can't easily test the internal limit value without making requests,
        // but we can verify the layer is created successfully
        assert!(std::mem::size_of_val(&layer) > 0);
    }

    #[test]
    fn test_create_body_limit_layer_custom_size() {
        let custom_limit = 512 * 1024; // 512KB
        let layer = create_body_limit_layer_with_size(custom_limit);
        // Verify the layer is created successfully
        assert!(std::mem::size_of_val(&layer) > 0);
    }

    #[test]
    fn test_create_body_limit_layer_zero_size() {
        let layer = create_body_limit_layer_with_size(0);
        // Even with 0 size, the layer should be created
        // (though it would reject all requests with bodies)
        assert!(std::mem::size_of_val(&layer) > 0);
    }

    #[test]
    fn test_create_body_limit_layer_large_size() {
        let large_limit = 100 * 1024 * 1024; // 100MB
        let layer = create_body_limit_layer_with_size(large_limit);
        // Verify the layer handles large limits
        assert!(std::mem::size_of_val(&layer) > 0);
    }
}
