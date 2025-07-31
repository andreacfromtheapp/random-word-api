use axum::{routing::get, routing::post, Router};
use http::Method;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{general::ping, word::*};

/// Top level router setup function
pub async fn create_router(dbpool: sqlx::Pool<sqlx::Sqlite>) -> axum::Router {
    let origins = [
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = Router::new()
        .nest(
            "/admin",
            Router::new()
                .route("/alive", get(|| async { "ok" }))
                .route("/ready", get(ping))
                .route("/words", get(word_list))
                .route("/words/new", post(word_create))
                .route(
                    "/words/{id}",
                    get(word_read).put(word_update).delete(word_delete),
                ),
        )
        .with_state(dbpool.clone())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        );

    // Add public routes under /
    let public_routes = Router::new()
        .route("/alive", get(|| async { "ok" }))
        .route("/ready", get(ping))
        .route("/word", get(word_random))
        .with_state(dbpool.clone())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.clone()),
        );

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .layer(TraceLayer::new_for_http())
}
