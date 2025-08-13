//! Public word retrieval routes
//!
//! Provides `/{lang}/word` and `/{lang}/word/{type}` endpoints for retrieving
//! random words with optional grammatical type filtering. All endpoints are
//! publicly accessible and return JSON responses.

// Public routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::word::*;
use crate::state::AppState;

/// Creates public word routes with language support and CORS configuration
pub fn create_word_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .route("/{lang}/random", get(word_random))
        .route("/{lang}/{type}", get(word_type))
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.to_owned()),
        )
}
