// OpenAPI documentation configuration
use axum::Router;
use utoipa::OpenApi;

use crate::handlers::{admin::*, healthcheck::*, word::*};
use crate::model::word::{UpsertWord, Word};
use crate::state::AppState;

/// OpenAPI documentation with manual path configuration
#[derive(OpenApi)]
#[openapi(
    paths(
        // Public endpoints
        alive,
        ping,
        word_random,
        // Admin endpoints
        word_list,
        word_create,
        word_read,
        word_update,
        word_delete,
    ),
    components(
        schemas(Word, UpsertWord)
    ),
    tags(
        (name = "healthcheck_endpoints", description = "Health check and system status endpoints"),
        (name = "public_endpoints", description = "Public word retrieval endpoints"),
        (name = "administration_endpoints", description = "Administrative word management endpoints"),
    ),
)]
pub struct ApiDoc;

/// SwaggerUI documentation router
pub fn create_swagger_routes() -> Router {
    use utoipa_swagger_ui::SwaggerUi;

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Redoc documentation router
pub fn create_redoc_routes() -> Router {
    use utoipa_redoc::{Redoc, Servable};

    Router::new().merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
}

/// Scalar documentation router
pub fn create_scalar_routes() -> Router {
    use utoipa_scalar::{Scalar, Servable};

    Router::new().merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}

/// RapiDoc documentation router
pub fn create_rapidoc_routes() -> Router {
    use utoipa_rapidoc::RapiDoc;

    Router::new().merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
}

/// OpenAPI Docs router with SwaggerUI, Redoc, Scalar, and RapiDoc
pub fn create_apidocs_routes(state: AppState, origins: Vec<http::HeaderValue>) -> Router {
    use http::Method;
    use tower_http::cors::CorsLayer;

    let mut router = Router::new();

    // Get the config to check which documentation routes to enable
    if let Ok(config) = state.config.lock() {
        if config.openapi.enable_swagger_ui {
            router = router.merge(create_swagger_routes());
        }
        if config.openapi.enable_redoc {
            router = router.merge(create_redoc_routes());
        }
        if config.openapi.enable_scalar {
            router = router.merge(create_scalar_routes());
        }
        if config.openapi.enable_rapidoc {
            router = router.merge(create_rapidoc_routes());
        }
    }

    router.layer(
        CorsLayer::new()
            .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
            .allow_origin(origins.clone()),
    )
}
