//! Public word handlers module
//!
//! This module contains HTTP handlers for publicly accessible word retrieval
//! endpoints. These handlers provide the core functionality for end users to
//! access random words from the database without requiring authentication.
//!
//! # Public Access
//!
//! All endpoints in this module are publicly accessible and do not require
//! authentication or special privileges. They are designed to serve the primary
//! use case of the random word API - providing random dictionary words to
//! applications and users.
//!
//! # Functionality
//!
//! The public word endpoints provide:
//! - Random word retrieval with complete word information
//! - Random word retrieval filtered by grammatical type (noun, verb, adjective, adverb)
//! - JSON-formatted responses suitable for API consumption
//! - Consistent error handling and status codes
//! - OpenAPI documentation for easy integration
//!
//! # Response Format
//!
//! All endpoints return GetWord objects containing:
//! - The word text (lemma)
//! - Dictionary definition
//! - IPA pronunciation notation
//!
//! # Error Handling
//!
//! Endpoints handle common error scenarios including empty databases,
//! connection failures, and other system issues with appropriate HTTP
//! status codes and error messages.
use crate::error::AppError;
use crate::models::word::GetWord;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

/// Retrieves a random word from the database as a JSON object.
///
/// This endpoint provides the core functionality of the random word API by
/// returning a randomly selected word from the database. It uses SQLite's
/// built-in random functionality to ensure fair distribution across all
/// available words.
///
/// # Public Access
///
/// This endpoint is publicly accessible and does not require authentication.
/// It serves as the primary interface for applications and users seeking
/// random vocabulary words for various purposes.
///
/// # Database Operation
///
/// The endpoint performs a database query that:
/// - Selects from the entire words table
/// - Uses SQLite's RANDOM() function for selection
/// - Limits results to a single word
/// - Returns complete word information including metadata
///
/// # Response Content
///
/// Returns a complete Word object containing:
/// - The word text (dictionary lemma)
/// - Full definition text
/// - IPA pronunciation notation
///
/// # Returns
///
/// * `200 OK` - Random word successfully retrieved and returned
/// * `400 Bad Request` - Invalid language code provided
/// * `404 Not Found` - No words available in database (empty database)
/// * `500 Internal Server Error` - Database connection or query error
///
/// # Response Format
///
/// Returns a JSON object with the complete word structure, making it
/// suitable for direct consumption by client applications.
#[utoipa::path(
    get,
    path = "/{lang}/word",
    operation_id = "public_word_random",
    tag = "public_endpoints",
    summary = "Get Random Word",
    description = "Retrieves a randomly selected word from the specified language database without requiring authentication",
    responses(
        (status = 200, description = "Random word successfully retrieved and returned", body = GetWord),
        (status = 400, description = "Bad Request - Invalid language code provided"),
        (status = 404, description = "Not Found - No words available in the specified language database"),
        (status = 500, description = "Internal Server Error - Database connection or query error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word retrieval (currently supports 'en' for English)", example = "en"),
    )
)]
pub async fn word_random(
    State(state): State<AppState>,
    Path(lang): Path<String>,
) -> Result<Json<GetWord>, AppError> {
    GetWord::random_word(state.dbpool, &lang)
        .await
        .map(Json::from)
}

/// Retrieves a random word of a specific grammatical type from the database.
///
/// This endpoint extends the random word functionality by allowing clients to
/// request words of specific grammatical types (noun, verb, adjective, adverb).
/// It uses SQLite's built-in random functionality combined with type filtering
/// to ensure fair distribution within the specified word category.
///
/// # Public Access
///
/// This endpoint is publicly accessible and does not require authentication.
/// It serves applications that need words of specific grammatical types for
/// educational, creative, or gaming purposes.
///
/// # Database Operation
///
/// The endpoint performs a filtered database query that:
/// - Selects from the words table with type filtering
/// - Uses SQLite's RANDOM() function for selection within type
/// - Limits results to a single word matching the criteria
/// - Returns complete word information excluding metadata
///
/// # Response Content
///
/// Returns a GetWord object containing:
/// - The word text (dictionary lemma)
/// - Full definition text
/// - IPA pronunciation notation
///
/// # Returns
///
/// * `200 OK` - Random word of specified type successfully retrieved
/// * `400 Bad Request` - Invalid language code or unsupported word type
/// * `404 Not Found` - No words of specified type available in database
/// * `500 Internal Server Error` - Database connection or query error
///
/// # Response Format
///
/// Returns a JSON object with the complete word structure, filtered by
/// grammatical type for specialized application needs.
#[utoipa::path(
    get,
    path = "/{lang}/word/{type}",
    operation_id = "public_word_random_type",
    tag = "public_endpoints",
    summary = "Get Random Word by Type",
    description = "Retrieves a randomly selected word of a specific grammatical type (noun, verb, adjective, or adverb) from the specified language database without requiring authentication",
    responses(
        (status = 200, description = "Random word of specified type successfully retrieved and returned", body = GetWord),
        (status = 400, description = "Bad Request - Invalid language code or unsupported word type provided"),
        (status = 404, description = "Not Found - No words of specified type available in the language database"),
        (status = 500, description = "Internal Server Error - Database connection or query error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word retrieval (currently supports 'en' for English)", example = "en"),
        ("type" = String, Path, description = "Grammatical type filter for word selection (noun, verb, adjective, adverb)", example = "noun"),
    )
)]
pub async fn word_type(
    State(state): State<AppState>,
    Path((lang, word_type)): Path<(String, String)>,
) -> Result<Json<GetWord>, AppError> {
    GetWord::random_type(state.dbpool, &lang, &word_type)
        .await
        .map(Json::from)
}
