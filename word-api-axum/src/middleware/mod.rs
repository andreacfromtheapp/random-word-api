//! Middleware configuration and setup
//!
//! This module provides middleware components for the API including:
//! - Rate limiting per IP address
//! - Security headers
//! - Request body size limits
//!
//! All middleware is configured via the `ApiConfig` and applied globally.

pub mod limits;
pub mod security;
pub mod tracing;

/// Re-export middleware components for easy access
pub use limits::create_body_limit_layer_with_size;
pub use security::security_headers;
pub use tracing::init_tracing;
