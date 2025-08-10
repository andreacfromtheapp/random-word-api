//! Routes module
//!
//! This module provides the main routing configuration and setup for the random word API.
//! It combines all route modules into a unified router with proper middleware configuration,
//! CORS settings, and request tracing for comprehensive HTTP request handling.
//!
//! # Route Organization
//!
//! The routing system is organized into four main categories:
//!
//! - `admin`: Administrative endpoints for word management (authentication required)
//! - `healthcheck`: Public health and status endpoints for monitoring
//! - `openapi`: API documentation interfaces (SwaggerUI, Redoc, Scalar, RapiDoc)
//! - `word`: Public word retrieval endpoints for end-user consumption
//!
//! # Router Architecture
//!
//! The module follows a hierarchical router structure:
//! - Top-level router combines all sub-routers
//! - Each sub-router handles its specific domain with appropriate middleware
//! - CORS configuration is applied per route group based on security requirements
//! - Tracing middleware provides request logging and performance monitoring
//!
//! # Middleware Stack
//!
//! The router includes essential middleware for:
//! - HTTP request tracing and logging
//! - CORS policy enforcement
//! - Error handling and response formatting
//! - State injection for database and configuration access
//!
//! # CORS Configuration
//!
//! Cross-Origin Resource Sharing is configured to allow:
//! - Localhost development (127.0.0.1, localhost)
//! - Appropriate HTTP methods per route group
//! - Secure headers for production deployment
//!
//! # State Management
//!
//! All routes receive the shared application state containing:
//! - Database connection pool for efficient query execution
//! - Configuration settings for runtime behavior
//! - Shared resources for consistent request handling

// Routes module
use axum::Router;
use tower_http::trace::TraceLayer;

pub mod admin;
pub mod healthcheck;
pub mod openapi;
pub mod word;

use crate::state::AppState;
use admin::create_admin_routes;
use healthcheck::create_health_routes;
use openapi::create_apidocs_routes;
use word::create_word_routes;

/// Creates the main application router with all route modules and middleware.
///
/// This function assembles the complete routing structure for the API by combining
/// all specialized route modules into a unified router. It configures the middleware
/// stack including CORS policies, request tracing, and state injection for proper
/// request handling across all endpoints.
///
/// # Router Composition
///
/// The function creates and merges the following route groups:
/// - Admin routes for word management under `/admin` prefix
/// - Health check routes under `/health` prefix
/// - API documentation routes with configurable endpoints
/// - Public word routes with language-specific paths
///
/// # Middleware Configuration
///
/// Applies essential middleware layers:
/// - `TraceLayer`: HTTP request tracing for logging and monitoring
/// - Individual CORS policies per route group for security
/// - State injection for database and configuration access
///
/// # CORS Policy
///
/// Configures Cross-Origin Resource Sharing for:
/// - Development origins (localhost, 127.0.0.1)
/// - Method-specific permissions per route group
/// - Header policies for secure cross-origin requests
///
/// # Arguments
///
/// * `state` - Shared application state containing database pool and configuration
///
/// # Returns
///
/// A configured Axum Router ready for HTTP server binding with all routes,
/// middleware, and state properly configured for request handling.
///
/// # Error Handling
///
/// Router creation is infallible as all error handling is deferred to
/// request processing time through the centralized error handling system.
pub async fn create_router(state: AppState) -> Router {
    let origins = vec![
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = create_admin_routes(state.clone(), &origins);

    // Add admin routes under /admin
    let health_routes = create_health_routes(state.clone(), &origins);

    // Add API Docs under /swagger-ui, /rapidoc, /scalar, and /redoc
    let apidocs_routes = create_apidocs_routes(state.clone(), &origins);

    // Add public routes under /{lang}/word
    let word_routes = create_word_routes(state.clone(), &origins);

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(health_routes)
        .merge(apidocs_routes)
        .merge(word_routes)
        .layer(TraceLayer::new_for_http())
}
