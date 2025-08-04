// All routers configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{general::*, word::*};

/// SwaggerUI router with RapidDoc too
fn mk_swagger_ui_routes(origins: Vec<HeaderValue>) -> axum::Router {
    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_swagger_ui::SwaggerUi;
    use utoipauto::utoipauto;

    // Setup SwaggerUI router
    #[utoipauto(paths = "./src/handlers, ./src/model")]
    #[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "Random Word API", description = "Word management endpoints.")
        ),
    )]
    pub struct ApiDoc;

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        )
}

/// Admin router (will have auth and more soon enough)
fn mk_admin_routes(dbpool: sqlx::Pool<sqlx::Sqlite>, origins: Vec<HeaderValue>) -> axum::Router {
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
        .with_state(dbpool.clone())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        )
}

/// Public API router
fn mk_public_routes(dbpool: sqlx::Pool<sqlx::Sqlite>, origins: Vec<HeaderValue>) -> axum::Router {
    Router::new()
        .route(
            "/alive",
            get(|| async { "The API is successfully running" }),
        )
        .route("/ready", get(ping))
        .route("/word", get(word_random))
        .with_state(dbpool.clone())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.clone()),
        )
}

/// Top level router setup function
pub async fn create_router(dbpool: sqlx::Pool<sqlx::Sqlite>) -> axum::Router {
    let origins = [
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = mk_admin_routes(dbpool.clone(), origins.to_vec());

    // Add public routes under /
    let public_routes = mk_public_routes(dbpool.clone(), origins.to_vec());

    // Add SwaggerUi under /swagger-ui
    let swagger_ui = mk_swagger_ui_routes(origins.to_vec());

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .merge(swagger_ui)
        .layer(TraceLayer::new_for_http())
}
