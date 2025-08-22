//! HTTP route configuration and middleware setup
//!
//! Combines all route modules with appropriate CORS settings and middleware.
//! Includes optional OpenAPI documentation interfaces and request tracing.
//!
//! # Route Groups
//! - `/auth` - Authentication and authorization (requires admin user)
//! - `/admin/{lang}/words` - Administrative CRUD endpoints (requires auth)
//! - `/health/alive` and `/health/ready` - Health check endpoints
//! - `/{lang}/random` and `/{lang}/{type}` - Public word retrieval endpoints
//! - `/swagger-ui`, `/redoc`, `/scalar`, `/rapidoc` - OpenAPI documentation interfaces
//!
//! # Security Model
//! - **Public routes**: Health checks, word retrieval, API documentation
//! - **Protected routes**: Admin word management (JWT required)
//!
//! # CORS Configuration
//! Configured for development (localhost) by default with appropriate method
//! allowlists per route group.
//!
//! # Middleware Stack (applied globally)
//! - Security headers
//! - Rate limiting per IP
//! - Request body size limits
//! - Compression (gzip/brotli)
//! - HTTP request tracing for observability
//! - CORS headers for cross-origin requests

// Routes module
use anyhow::{Context, Result};
use axum::{middleware, Router};
use http::HeaderValue;
use std::time::Duration;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub mod admin;
pub mod auth;
pub mod healthcheck;
pub mod openapi;
pub mod word;

use crate::error::AppError;
use crate::middleware::{create_body_limit_layer_with_size, init_tracing, security_headers};
use crate::state::AppState;
use admin::create_admin_routes;
use auth::create_auth_routes;
use healthcheck::create_health_routes;
use openapi::create_apidocs_routes;
use word::create_word_routes;

fn process_origins(allowed_origins: Vec<String>) -> anyhow::Result<Vec<HeaderValue>, AppError> {
    let origins = allowed_origins
        .into_iter()
        .map(|o| HeaderValue::try_from(o.clone()).context("couldn't convert origin"))
        .collect::<Result<Vec<_>, _>>()
        .context("Error processing allowed origins")?;

    Ok(origins)
}

/// Creates the main application router with all route modules and middleware
pub async fn create_router(shared_state: AppState) -> Result<Router, AppError> {
    // Get configuration for middleware setup
    let config = {
        let config_lock = shared_state.apiconfig.lock().unwrap();
        config_lock.clone()
    };

    // Make origins the expected type
    let origins = process_origins(config.server_settings.allowed_origins)?;

    // Add admin routes under /admin
    let admin_routes = create_admin_routes(shared_state.clone(), &origins);

    // Add auth routes under /auth
    let auth_routes = create_auth_routes(shared_state.clone(), &origins);

    // Add health routes under /health
    let health_routes = create_health_routes(shared_state.clone(), &origins);

    // Add API Docs under /swagger-ui, /rapidoc, /scalar, and /redoc
    let apidocs_routes = create_apidocs_routes(shared_state.clone(), &origins);

    // Add public word routes under /{lang}
    let word_routes = create_word_routes(shared_state.clone(), &origins);

    // Create the base router with all routes
    let mut router = Router::new()
        .merge(admin_routes)
        .merge(auth_routes)
        .merge(health_routes)
        .merge(apidocs_routes)
        .merge(word_routes);

    // Apply middleware stack in the correct order (inside-out):
    // Brotli and gzip compression
    router = router.layer(
        CompressionLayer::new()
            .br(config.compression.brotli)
            .gzip(config.compression.gzip),
    );

    // Timeout to prevent hanging requests
    router = router.layer(TimeoutLayer::new(Duration::from_secs(
        config.api_limits.request_timeout,
    )));

    // Creates a RequestBodyLimitLayer with custom size limit
    router = router.layer(create_body_limit_layer_with_size(
        config.api_limits.request_body_limit_kilobytes,
    ));

    // Rate limiting per IP
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(config.api_limits.rate_limit_per_second)
        .burst_size(config.api_limits.burst_size)
        .finish()
        .unwrap();

    router = router.layer(GovernorLayer::new(governor_conf));

    // Security headers
    // TODO make this configurable
    router = router.layer(middleware::from_fn(security_headers));

    // Enable tracing using https://tokio.rs/#tk-lib-tracing
    init_tracing();
    router = router.layer(TraceLayer::new_for_http());

    Ok(router)
}
