// Routes module
use axum::Router;

use tower_http::trace::TraceLayer;

pub mod admin;
pub mod openapi;
pub mod public;

use crate::state::AppState;
use admin::create_admin_routes;
use openapi::create_apidocs_routes;
use public::create_public_routes;

/// Top level router setup function
///
/// Gotta try https://docs.rs/utoipa-axum/latest/utoipa_axum/ next...
pub async fn create_router(state: AppState) -> Router {
    let origins = [
        "http://localhost".parse().unwrap(),
        "http://127.0.0.1".parse().unwrap(),
    ];

    // Add admin routes under /admin
    let admin_routes = create_admin_routes(state.clone(), origins.to_vec());

    // Add public routes under /
    let public_routes = create_public_routes(state.clone(), origins.to_vec());

    // Add API Docs under /swagger-ui, /rapidoc, /scalar, and /redoc
    let apidocs_routes = create_apidocs_routes(state.clone(), origins.to_vec());

    // Setup top-level router
    Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .merge(apidocs_routes)
        .layer(TraceLayer::new_for_http())
}
