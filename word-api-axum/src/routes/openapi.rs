//! Optional API documentation interfaces
//!
//! Provides SwaggerUI, Redoc, Scalar, and RapiDoc interfaces when enabled
//! via configuration. Each interface can be independently enabled or disabled.

// OpenAPI documentation configuration
use axum::Router;
use http::HeaderValue;
use utoipa::OpenApi;

use crate::handlers::{admin::*, auth::*, healthcheck::*, word::*};
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest};
use crate::models::word::{GetWord, UpsertWord, Word};
use crate::state::AppState;

/// OpenAPI specification structure with comprehensive endpoint documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health check endpoints
        alive,
        ready,
        // Public word endpoints
        word_random,
        word_type,
        // Authentication endpoints
        login,
        register,
        // Administrative endpoints
        word_list,
        word_create,
        word_read,
        word_update,
        word_delete,
    ),
    components(
        schemas(Word, GetWord, UpsertWord, LoginRequest, RegisterRequest, AuthResponse)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "healthcheck_endpoints", description = "Health check and system status endpoints"),
        (name = "public_endpoints", description = "Public word retrieval endpoints"),
        (name = "auth_endpoints", description = "User authentication endpoints"),
        (name = "administration_endpoints", description = "Administrative word management endpoints. Require authentication and administrative privileges."),
    ),
)]
pub struct ApiDoc;

/// Security scheme modifier to add Bearer authentication
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.security_schemes.insert(
                "bearer_auth".to_string(),
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter JWT token"))
                        .build(),
                ),
            );
        }
    }
}

/// Creates SwaggerUI documentation router with interactive API exploration
/// Protected by authentication middleware
pub fn create_swagger_routes() -> Router {
    use utoipa_swagger_ui::SwaggerUi;

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Creates Redoc documentation router with clean, readable interface
pub fn create_redoc_routes() -> Router {
    use utoipa_redoc::{Redoc, Servable};

    Router::new().merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
}

/// Creates Scalar documentation router with modern, advanced interface
/// Protected by authentication middleware
pub fn create_scalar_routes() -> Router {
    use utoipa_scalar::{Scalar, Servable};

    Router::new().merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}

/// Creates RapiDoc documentation router with lightweight, fast interface
/// Protected by authentication middleware
pub fn create_rapidoc_routes() -> Router {
    use utoipa_rapidoc::RapiDoc;

    Router::new().merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
}

/// Creates the complete OpenAPI documentation router with configurable interfaces
/// Protected routes require admin authentication, ReDoc remains public
pub fn create_apidocs_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    use crate::auth::admin_auth_middleware;
    use axum::middleware;
    use http::Method;
    use tower_http::cors::CorsLayer;

    let mut router = Router::new();

    // Get the config to check which documentation routes to enable
    if let Ok(config) = shared_state.apiconfig.lock() {
        if config.openapi.enable_swagger_ui {
            router = router.merge(create_swagger_routes().route_layer(
                middleware::from_fn_with_state(shared_state.clone(), admin_auth_middleware),
            ));
        }
        if config.openapi.enable_redoc {
            // Redoc remains public as requested
            router = router.merge(create_redoc_routes());
        }
        if config.openapi.enable_scalar {
            router = router.merge(create_scalar_routes().route_layer(
                middleware::from_fn_with_state(shared_state.clone(), admin_auth_middleware),
            ));
        }
        if config.openapi.enable_rapidoc {
            router = router.merge(create_rapidoc_routes().route_layer(
                middleware::from_fn_with_state(shared_state.clone(), admin_auth_middleware),
            ));
        }
    }

    router.layer(
        CorsLayer::new()
            .allow_methods([Method::POST, Method::GET, Method::PUT, Method::DELETE])
            .allow_origin(origins.to_owned()),
    )
}
