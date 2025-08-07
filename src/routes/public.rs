// Public routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::{healthcheck::*, word::*};
use crate::state::AppState;

/// Public API router
pub fn create_public_routes(state: AppState, origins: Vec<HeaderValue>) -> Router {
    Router::new()
        .route("/alive", get(alive))
        .route("/ready", get(ping))
        .route("/word", get(word_random))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.clone()),
        )
}
