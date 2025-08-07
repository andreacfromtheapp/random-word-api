// Admin routes configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::admin::*;
use crate::state::AppState;

/// Admin router (will have auth and more soon enough)
pub fn create_admin_routes(state: AppState, origins: Vec<HeaderValue>) -> Router {
    Router::new()
        .nest(
            "/admin",
            Router::new()
                .route("/words", get(word_list).post(word_create))
                .route(
                    "/words/{id}",
                    get(word_read).put(word_update).delete(word_delete),
                ),
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        )
}
