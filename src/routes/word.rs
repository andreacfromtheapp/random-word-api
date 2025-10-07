//! Public word retrieval routes
//!
//! Provides `/{lang}/random` and `/{lang}/{type}` endpoints for retrieving
//! random words with optional grammatical type filtering. All endpoints are
//! publicly accessible and return JSON responses.
//!
//! # Routes
//! - `GET /{lang}/random` - Get random word from any grammatical type
//! - `GET /{lang}/{type}` - Get random word of specific grammatical type
//!
//! # Supported Languages
//! - `en` - English (currently the only supported language)
//!
//! # Supported Word Types
//! - `noun`, `verb`, `adjective`, `adverb`, `pronoun`, `preposition`,
//!   `conjunction`, `interjection`, `article`

use axum::{routing::get, Router};
use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::handlers::word::*;
use crate::state::AppState;

/// Creates public word routes with language support and CORS configuration
///
/// Sets up the main public API endpoints for word retrieval with appropriate
/// CORS headers for cross-origin requests from web applications.
pub fn create_word_routes(shared_state: AppState, origins: &[HeaderValue]) -> Router {
    Router::new()
        .route("/{lang}/random", get(word_random))
        .route("/{lang}/{type}", get(word_type))
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins.to_owned()),
        )
}
