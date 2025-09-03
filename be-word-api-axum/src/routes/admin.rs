//! Administrative route configuration with authentication middleware
//!
//! Configures CRUD endpoints for word management under `/admin/{lang}/words`.
//! Includes CORS configuration for development and production use.
//! All routes require admin authentication via JWT token.
//!
//! # Routes
//! - `GET /admin/{lang}/words` - List all words (admin only)
//! - `POST /admin/{lang}/words` - Create new word (admin only)
//! - `GET /admin/{lang}/words/{id}` - Get word by ID (admin only)
//! - `PUT /admin/{lang}/words/{id}` - Update word by ID (admin only)
//! - `DELETE /admin/{lang}/words/{id}` - Delete word by ID (admin only)

// Admin routes configuration
use axum::{middleware, routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::auth::admin_auth_middleware;
use crate::handlers::admin::*;
use crate::state::AppState;

/// Creates administrative routes with CORS and state injection
pub fn create_admin_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .nest(
            "/admin",
            Router::new()
                .route("/{lang}/words", get(word_list).post(word_create))
                .route(
                    "/{lang}/words/{id}",
                    get(word_read).put(word_update).delete(word_delete),
                )
                .layer(middleware::from_fn_with_state(
                    shared_state.clone(),
                    admin_auth_middleware,
                )),
        )
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.to_owned()),
        )
}
