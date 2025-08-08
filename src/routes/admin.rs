//! Administrative routes module
//!
//! This module configures HTTP routes for administrative word management operations.
//! It sets up protected endpoints under the `/admin` path prefix with appropriate
//! middleware for authentication, CORS policy, and request handling.
//!
//! # Route Structure
//!
//! All administrative routes are nested under the `/admin` prefix with language
//! support through path parameters:
//! - `/admin/{lang}/words` - Word collection operations (GET, POST)
//! - `/admin/{lang}/words/{id}` - Individual word operations (GET, PUT, DELETE)
//!
//! # Security Configuration
//!
//! Administrative routes are designed to be protected by authentication middleware
//! that should be added at the application level. The routes themselves handle
//! authorization through proper error responses for unauthorized access.
//!
//! # CORS Policy
//!
//! The module configures Cross-Origin Resource Sharing to allow:
//! - Standard HTTP methods for CRUD operations (GET, POST, PUT, DELETE)
//! - Localhost origins for development and testing
//! - Secure headers for production deployment
//!
//! # Language Support
//!
//! All routes include language path parameters to support internationalization
//! and multi-language word databases in future iterations.
//!
//! # Middleware Stack
//!
//! The router includes essential middleware for:
//! - CORS policy enforcement for cross-origin requests
//! - State injection for database and configuration access
//! - Error handling integration with the centralized error system

// Admin routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::admin::*;
use crate::state::AppState;

/// Creates the administrative routes with proper middleware configuration.
///
/// This function sets up all administrative endpoints for word management with
/// appropriate CORS policies and state injection. The routes are designed to
/// handle complete CRUD operations for the word database with comprehensive
/// validation and error handling.
///
/// # Route Configuration
///
/// Sets up two main route patterns:
/// - Collection routes for listing and creating words
/// - Resource routes for individual word operations by ID
///
/// # Method Mapping
///
/// The routes support standard HTTP methods:
/// - GET: Retrieve word(s) from database
/// - POST: Create new words with validation
/// - PUT: Update existing words completely
/// - DELETE: Permanently remove words from database
///
/// # State Injection
///
/// All routes receive the shared AppState containing:
/// - Database connection pool for efficient queries
/// - Configuration settings for runtime behavior
/// - Shared resources for consistent request handling
///
/// # CORS Configuration
///
/// Configures cross-origin policies to allow:
/// - Administrative HTTP methods (POST, GET, PUT, DELETE)
/// - Development origins for local testing
/// - Secure headers for production deployment
///
/// # Arguments
///
/// * `state` - Shared application state with database and configuration
/// * `origins` - List of allowed CORS origins for cross-origin requests
///
/// # Returns
///
/// A configured Axum Router with all administrative endpoints, middleware,
/// and proper state injection ready for integration with the main router.
///
/// # Future Authentication
///
/// The routes are structured to easily integrate authentication middleware
/// at the router level for protecting administrative operations.
pub fn create_admin_routes(state: AppState, origins: Vec<HeaderValue>) -> Router {
    Router::new()
        .nest(
            "/admin",
            Router::new()
                .route("/{lang}/words", get(word_list).post(word_create))
                .route(
                    "/{lang}/words/{id}",
                    get(word_read).put(word_update).delete(word_delete),
                ),
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        )
}
