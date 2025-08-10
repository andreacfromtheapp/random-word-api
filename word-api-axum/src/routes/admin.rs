//! Administrative route configuration with authentication middleware
//!
//! Configures CRUD endpoints for word management under `/admin/{lang}/words`.
//! Includes CORS configuration for development and production use.

// Admin routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::admin::*;
use crate::state::AppState;

/// Creates administrative routes with CORS and state injection
pub fn create_admin_routes(state: AppState, origins: &[HeaderValue]) -> Router {
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
                .allow_origin(origins.to_owned()),
        )
}
