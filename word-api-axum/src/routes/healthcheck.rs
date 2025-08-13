//! Health check routes for monitoring systems
//!
//! Provides `/health/alive` and `/health/ready` endpoints for liveness
//! and readiness probes. Suitable for load balancers and orchestration platforms.

// Public routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::healthcheck::*;
use crate::state::AppState;

/// Creates health check routes with monitoring-optimized configuration
pub fn create_health_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .nest(
            "/health",
            Router::new()
                .route("/alive", get(alive))
                .route("/ready", get(ready)),
        )
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.to_owned()),
        )
}
