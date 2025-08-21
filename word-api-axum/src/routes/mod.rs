//! HTTP route configuration and middleware setup
//!
//! Combines all route modules with appropriate CORS settings and middleware.
//! Includes optional OpenAPI documentation interfaces and request tracing.
//!
//! # Route Groups
//! - `/admin/{lang}/words` - Administrative CRUD endpoints (requires auth)
//! - `/health/alive` and `/health/ready` - Health check endpoints
//! - `/{lang}/random` and `/{lang}/{type}` - Public word retrieval endpoints
//! - `/swagger-ui`, `/redoc`, `/scalar`, `/rapidoc` - OpenAPI documentation interfaces
//!
//! # CORS Configuration
//! Configured for development (localhost:5173) and production (speak-and-spell.netlify.app)
//! with appropriate method allowlists per route group.
//!
//! # Middleware
//! - HTTP request tracing for observability
//! - CORS headers for cross-origin requests

// Routes module
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

pub mod admin;
pub mod auth;
pub mod healthcheck;
pub mod openapi;
pub mod word;

use crate::state::AppState;
use admin::create_admin_routes;
use auth::create_auth_routes;
use healthcheck::create_health_routes;
use openapi::create_apidocs_routes;
use word::create_word_routes;

/// Creates the main application router with all route modules and middleware
pub async fn create_router(shared_state: AppState) -> Router {
    let origins = vec![
        "http://localhost:5173".parse().unwrap(),
        "https://speak-and-spell.netlify.app/".parse().unwrap(),
    ];

    let compression_layer = CompressionLayer::new().br(true).gzip(true);

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

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(auth_routes)
        .merge(health_routes)
        .merge(apidocs_routes)
        .merge(word_routes)
        .layer(compression_layer)
        .layer(TraceLayer::new_for_http())
}
