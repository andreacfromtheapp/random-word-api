// All routers configuration
use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{admin::*, general::*, word::*};

/// SwaggerUI router with RapidDoc and Scalar as well
fn create_apidocs_routes(origins: Vec<HeaderValue>) -> axum::Router {
    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_redoc::{Redoc, Servable as RedocServable};
    use utoipa_scalar::{Scalar, Servable as ScalarServable};
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

    // Set up SwaggerUi, RapiDoc, and Scalar endpoints
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
                .allow_origin(origins.clone()),
        )
}

/// Admin router (will have auth and more soon enough)
fn create_admin_routes(
    dbpool: sqlx::Pool<sqlx::Sqlite>,
    origins: Vec<HeaderValue>,
) -> axum::Router {
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
fn create_public_routes(
    dbpool: sqlx::Pool<sqlx::Sqlite>,
    origins: Vec<HeaderValue>,
) -> axum::Router {
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
///
/// Gotta try https://docs.rs/utoipa-axum/latest/utoipa_axum/ next...
pub async fn create_router(dbpool: sqlx::Pool<sqlx::Sqlite>) -> axum::Router {
    let origins = [
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = create_admin_routes(dbpool.clone(), origins.to_vec());

    // Add public routes under /
    let public_routes = create_public_routes(dbpool.clone(), origins.to_vec());

    // Add API Docs under /swagger-ui, /rapidoc, /scalar, and /redoc
    let apidocs_routes = create_apidocs_routes(origins.to_vec());

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .merge(apidocs_routes)
        .layer(TraceLayer::new_for_http())
}
