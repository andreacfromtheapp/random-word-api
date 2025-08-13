//! HTTP route configuration and middleware setup
//!
//! Combines all route modules with appropriate CORS settings and middleware.
//! Includes optional OpenAPI documentation interfaces.
//!
//! # Route groups
//! - `/admin` - Administrative endpoints
//! - `/health` - Health checks
//! - `/{lang}/word` - Public word endpoints
//! - OpenAPI docs (SwaggerUI, Redoc, etc.)

// Routes module
use axum::Router;
use tower_http::trace::TraceLayer;

pub mod admin;
pub mod healthcheck;
pub mod openapi;
pub mod word;

use crate::state::AppState;
use admin::create_admin_routes;
use healthcheck::create_health_routes;
use openapi::create_apidocs_routes;
use word::create_word_routes;

/// Creates the main application router with all route modules and middleware
pub async fn create_router(shared_state: AppState) -> Router {
    let origins = vec![
        "http://localhost:5173".parse().unwrap(),
        "https://speak-and-spell.netlify.app/".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = create_admin_routes(shared_state.clone(), &origins);

    // Add admin routes under /admin
    let health_routes = create_health_routes(shared_state.clone(), &origins);

    // Add API Docs under /swagger-ui, /rapidoc, /scalar, and /redoc
    let apidocs_routes = create_apidocs_routes(shared_state.clone(), &origins);

    // Add public routes under /{lang}/word
    let word_routes = create_word_routes(shared_state.clone(), &origins);

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(health_routes)
        .merge(apidocs_routes)
        .merge(word_routes)
        .layer(TraceLayer::new_for_http())
}
