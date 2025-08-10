//! Health check routes module
//!
//! This module configures HTTP routes for health monitoring and system status endpoints.
//! It provides essential monitoring capabilities for the random word API including
//! liveness probes, readiness checks, and database connectivity validation.
//!
//! # Route Structure
//!
//! All health check routes are nested under the `/health` path prefix:
//! - `/health/alive` - Basic liveness probe for service availability
//! - `/health/ready` - Comprehensive readiness check with database validation
//!
//! # Monitoring Integration
//!
//! These routes are specifically designed for integration with:
//! - Container orchestration platforms (Kubernetes, Docker Swarm)
//! - Load balancer health checks (HAProxy, NGINX, AWS ALB)
//! - Application performance monitoring systems
//! - Automated alerting and notification systems
//!
//! # Response Characteristics
//!
//! Health check endpoints are optimized for monitoring systems:
//! - Fast response times for minimal monitoring overhead
//! - Simple text responses for easy parsing
//! - Clear success/failure status codes
//! - Minimal resource consumption
//!
//! # CORS Configuration
//!
//! Health check routes use permissive CORS settings allowing:
//! - GET method access for monitoring requests
//! - Development origins for local testing
//! - Standard headers for monitoring tool compatibility
//!
//! # Public Access
//!
//! All health check endpoints are publicly accessible without authentication
//! to ensure monitoring systems can always verify service status regardless
//! of authentication system availability.

// Public routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::healthcheck::*;
use crate::state::AppState;

/// Creates health check routes with monitoring-optimized configuration.
///
/// This function sets up essential health monitoring endpoints under the `/health`
/// prefix with appropriate middleware for cross-origin access and state injection.
/// The routes are designed for integration with monitoring systems and provide
/// both basic liveness and comprehensive readiness checking.
///
/// # Route Configuration
///
/// Sets up two primary health check endpoints:
/// - Liveness probe for basic service availability checking
/// - Readiness probe with database connectivity validation
///
/// # Monitoring Optimization
///
/// The routes are configured for optimal monitoring integration:
/// - Minimal middleware overhead for fast responses
/// - Simple CORS policy allowing monitoring tool access
/// - State injection for database connectivity testing
///
/// # CORS Policy
///
/// Configures cross-origin access to allow:
/// - GET method requests from monitoring systems
/// - Development origins for local testing and debugging
/// - Standard headers for monitoring tool compatibility
///
/// # Arguments
///
/// * `state` - Shared application state containing database pool and configuration
/// * `origins` - List of allowed CORS origins for cross-origin monitoring requests
///
/// # Returns
///
/// A configured Axum Router with health check endpoints, appropriate middleware,
/// and state injection ready for integration with the main application router.
///
/// # Performance Considerations
///
/// Health check routes are designed to be lightweight and respond quickly
/// to minimize impact on monitoring systems and overall application performance.
pub fn create_health_routes(state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .nest(
            "/health",
            Router::new()
                .route("/alive", get(alive))
                .route("/ready", get(ready)),
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.to_owned()),
        )
}
