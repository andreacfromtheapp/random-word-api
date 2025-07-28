use axum::{routing::get, routing::post, Router};
use http::Method;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::*;

pub async fn create_router(dbpool: sqlx::Pool<sqlx::Sqlite>) -> axum::Router {
    let origins = [
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    Router::new()
        .route("/alive", get(|| async { "ok" }))
        .route("/ready", get(ping))
        .route("/word", get(word_random))
        .nest(
            "/admin",
            Router::new()
                .route("/words", get(word_list))
                .route("/words/new", post(word_create))
                .route(
                    "/words/{id}",
                    get(word_read).put(word_update).delete(word_delete),
                ),
        )
        .with_state(dbpool)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins),
        )
        .layer(TraceLayer::new_for_http())
}
