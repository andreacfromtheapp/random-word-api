//! Public word routes module
//!
//! This module configures HTTP routes for publicly accessible word retrieval endpoints.
//! These routes provide the core functionality of the random word API, allowing end
//! users to access random dictionary words without authentication or special privileges.
//!
//! # Route Structure
//!
//! All public word routes follow a language-specific pattern:
//! - `/{lang}/word` - Random word retrieval for specified language
//!
//! # Language Support
//!
//! The routes include language path parameters to support:
//! - Multi-language word databases
//! - Language-specific word retrieval
//! - Internationalization capabilities
//! - Future expansion to additional languages
//!
//! # Public Access
//!
//! All routes in this module are publicly accessible and designed for:
//! - High-frequency access from client applications
//! - Caching-friendly responses for performance
//! - Simple integration with various client technologies
//! - Minimal authentication overhead for ease of use
//!
//! # CORS Configuration
//!
//! Public word routes use permissive CORS settings allowing:
//! - GET method access for word retrieval
//! - Cross-origin requests from web applications
//! - Development and production origin support
//!
//! # Performance Optimization
//!
//! Routes are optimized for high-throughput word retrieval:
//! - Minimal middleware overhead
//! - Efficient database query patterns
//! - Lightweight response formatting
//! - Connection pool utilization for concurrent requests

// Public routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::word::*;
use crate::state::AppState;

/// Creates public word routes with language support and CORS configuration.
///
/// This function sets up publicly accessible endpoints for word retrieval under
/// language-specific paths. The routes are designed for high-frequency access
/// and provide the core functionality of the random word API without requiring
/// authentication.
///
/// # Route Configuration
///
/// Sets up language-parameterized routes for:
/// - Random word retrieval with language specification
/// - Future expansion to language-specific word operations
///
/// # Language Parameter
///
/// The `{lang}` path parameter allows for:
/// - Language-specific word database access
/// - Future multi-language support expansion
/// - Consistent URL patterns across language variants
///
/// # CORS Policy
///
/// Configures cross-origin access to allow:
/// - GET method requests for word retrieval
/// - Development origins for local testing
/// - Production origins for deployed applications
///
/// # State Injection
///
/// All routes receive the shared AppState containing:
/// - Database connection pool for word queries
/// - Configuration settings for runtime behavior
/// - Shared resources for consistent response handling
///
/// # Arguments
///
/// * `state` - Shared application state with database and configuration access
/// * `origins` - List of allowed CORS origins for cross-origin word requests
///
/// # Returns
///
/// A configured Axum Router with public word endpoints, appropriate CORS policies,
/// and state injection ready for integration with the main application router.
///
/// # Performance Characteristics
///
/// The routes are optimized for high-frequency access with minimal overhead
/// and efficient database connection utilization for concurrent word requests.
pub fn create_word_routes(state: AppState, origins: Vec<HeaderValue>) -> Router {
    Router::new()
        .route("/{lang}/word", get(word_random))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.clone()),
        )
}
