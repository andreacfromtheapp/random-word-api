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
//! - JSON-formatted responses suitable for API consumption
//! - Consistent error handling and status codes
//! - OpenAPI documentation for easy integration
//!
//! # Response Format
//!
//! All endpoints return complete Word objects containing:
//! - The word text (lemma)
//! - Dictionary definition
//! - IPA pronunciation notation
//! - Database metadata (ID, timestamps)
//!
//! # Error Handling
//!
//! Endpoints handle common error scenarios including empty databases,
//! connection failures, and other system issues with appropriate HTTP
//! status codes and error messages.
use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::word::Word;

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
/// - Unique database identifier
/// - The word text (dictionary lemma)
/// - Full definition text
/// - IPA pronunciation notation
/// - Creation and update timestamps
///
/// # Returns
///
/// * `200 OK` - Random word successfully retrieved and returned
/// * `404 Not Found` - No words available in database (empty database)
/// * `500 Internal Server Error` - Database connection or query error
///
/// # Response Format
///
/// Returns a JSON object with the complete word structure, making it
/// suitable for direct consumption by client applications.
#[utoipa::path(
    get,
    path = "/word",
    operation_id = "public_word_random",
    tag = "public_endpoints",
    responses(
        (status = 200, description = "A random word returned successfully", body = Word),
        (status = 404, description = "Couldn't return a random word. Does your database contain any?"),
    ),
)]
pub async fn word_random(State(dbpool): State<SqlitePool>) -> Result<Json<Word>, AppError> {
    Word::random(dbpool).await.map(Json::from)
}
