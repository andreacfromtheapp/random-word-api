//! Authentication route configuration
//!
//! Configures login and registration endpoints under `/auth`.
//! Includes CORS configuration for development and production use.
//!
//! # Routes
//! - `POST /auth/login` - User login endpoint
//! - `POST /auth/register` - User registration endpoint

use axum::{routing::post, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::auth::*;
use crate::state::AppState;

/// Creates authentication routes with CORS and state injection
pub fn create_auth_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/login", post(login))
                .route("/register", post(register)),
        )
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST])
                .allow_origin(origins.to_owned()),
        )
}
