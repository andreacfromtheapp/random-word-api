//! Administrative handlers module
//!
//! This module contains HTTP handlers for administrative word management operations.
//! All endpoints in this module require authentication and administrative privileges
//! to access, providing complete CRUD (Create, Read, Update, Delete) functionality
//! for managing the word database.
//!
//! # Security
//!
//! These endpoints are protected by authentication middleware and should only be
//! accessible to users with administrative privileges. They provide full access
//! to the word database including the ability to view all words, modify existing
//! entries, and permanently delete words.
//!
//! # Functionality
//!
//! - List all words in the database
//! - Create new word entries with validation
//! - Read individual words by ID
//! - Update existing word entries
//! - Delete words permanently from the database
//!
//! # Validation
//!
//! All create and update operations include comprehensive validation for:
//! - Word format (valid lemma according to Merriam-Webster standards)
//! - Definition content (appropriate dictionary language)
//! - Pronunciation format (valid IPA notation)
//!
//! # Error Handling
//!
//! All handlers return appropriate HTTP status codes and error messages for
//! various failure scenarios including validation errors, database connection
//! issues, and resource not found errors.
use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::word::{UpsertWord, Word};

/// Lists all words in the database as a JSON array.
///
/// This endpoint returns every word stored in the database without any filtering
/// or pagination. It is intended for administrative oversight and should be used
/// with caution on large datasets as it returns all records in a single response.
///
/// # Security
///
/// Requires authentication and administrative privileges. This endpoint exposes
/// the entire word database and should only be accessible to trusted administrators.
///
/// # Returns
///
/// * `200 OK` - JSON array containing all words with their complete information
/// * `404 Not Found` - Database is empty or no words are available
/// * `500 Internal Server Error` - Database connection or query error
///
/// # Response Format
///
/// Returns a JSON array where each element is a complete Word object containing
/// id, word, definition, pronunciation, and timestamp information.
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/words",
    operation_id = "admin_words_list_all",
    tag = "administration_endpoints",
    responses(
        (status = 200, description = "Listed every single word successfully", body = [Word]),
        (status = 404, description = "Couldn't list words. Does your database contain any?"),
    )
)]
pub async fn word_list(State(dbpool): State<SqlitePool>) -> Result<Json<Vec<Word>>, AppError> {
    Word::list(dbpool).await.map(Json::from)
}

/// Creates a new word entry in the database.
///
/// This endpoint accepts a JSON payload containing word data and creates a new
/// entry in the database after comprehensive validation. All text fields are
/// automatically converted to lowercase for consistency.
///
/// # Security
///
/// Requires authentication and administrative privileges. Only authorized
/// administrators should be able to add new words to the database.
///
/// # Validation
///
/// The request body is validated for:
/// - Word format: Must be a valid lemma with no whitespace and appropriate characters
/// - Definition: Must contain only dictionary-appropriate text and punctuation
/// - Pronunciation: Must be valid IPA notation enclosed in forward slashes
///
/// # Request Body
///
/// Expects a JSON object with `word`, `definition`, and `pronunciation` fields.
/// All fields are required and must pass validation rules.
///
/// # Returns
///
/// * `200 OK` - Word successfully created, returns the new word with generated ID
/// * `415 Unsupported Media Type` - Invalid content type, expecting application/json
/// * `422 Unprocessable Entity` - Validation failed for one or more fields
/// * `500 Internal Server Error` - Database error or duplicate word constraint
#[utoipa::path(
    post,
    context_path = "/admin",
    path = "/words",
    operation_id = "admin_words_create",
    tag = "administration_endpoints",
    request_body(content = UpsertWord, description = "Word to add to the database", content_type = "application/json"),
    responses(
        (status = 200, description = "Word with {id} successfully added to the database", body = Word),
        (status = 415, description = "Please provide a valid word definition in your JSON body"),
        (status = 422, description = "Please provide a valid word definition in your JSON body"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn word_create(
    State(dbpool): State<SqlitePool>,
    Json(new_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::create(dbpool, new_word).await.map(Json::from)
}

/// Retrieves a specific word by its database ID.
///
/// This endpoint fetches a single word from the database using its unique
/// identifier. It provides administrators with the ability to inspect
/// individual word entries including their complete metadata.
///
/// # Security
///
/// Requires authentication and administrative privileges. This endpoint
/// provides access to internal database IDs and complete word records.
///
/// # Path Parameters
///
/// * `id` - The unique database identifier of the word to retrieve
///
/// # Returns
///
/// * `200 OK` - Word found and returned with all fields
/// * `404 Not Found` - No word exists with the specified ID
/// * `500 Internal Server Error` - Database connection or query error
///
/// # Response Format
///
/// Returns a complete Word object including id, word text, definition,
/// pronunciation, and creation/update timestamps.
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "admin_words_read_by_id",
    tag = "administration_endpoints",
    responses (
        (status = 200, description = "Word with {id} returned successfully", body = Word),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word database id to get Word for"),
    )
)]
pub async fn word_read(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<Json<Word>, AppError> {
    Word::read(dbpool, id).await.map(Json::from)
}

/// Updates an existing word entry in the database.
///
/// This endpoint modifies an existing word identified by its database ID.
/// The entire word record is updated with new values, and all text fields
/// are converted to lowercase for consistency. The updated_at timestamp
/// is automatically set to the current time.
///
/// # Security
///
/// Requires authentication and administrative privileges. This endpoint
/// allows modification of existing database records and should be restricted
/// to authorized administrators.
///
/// # Path Parameters
///
/// * `id` - The unique database identifier of the word to update
///
/// # Request Body
///
/// Expects a JSON object with `word`, `definition`, and `pronunciation` fields.
/// All fields are required and must pass the same validation as word creation.
///
/// # Validation
///
/// The request body undergoes the same validation as word creation:
/// - Word format validation for lemma standards
/// - Definition content validation for appropriate dictionary language
/// - Pronunciation validation for proper IPA notation format
///
/// # Returns
///
/// * `200 OK` - Word successfully updated, returns the modified word
/// * `404 Not Found` - No word exists with the specified ID
/// * `422 Unprocessable Entity` - Validation failed for one or more fields
/// * `500 Internal Server Error` - Database error during update operation
#[utoipa::path(
    put,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "admin_words_update_by_id",
    tag = "administration_endpoints",
    request_body(content = UpsertWord, description = "Word to update in the database", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} updated successfully", body = Word),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word id to update Word for"),
    )
)]
pub async fn word_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
    Json(updated_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::update(dbpool, id, updated_word).await.map(Json::from)
}

/// Permanently removes a word from the database.
///
/// This endpoint deletes a word record identified by its database ID. The
/// operation is irreversible and permanently removes all associated data
/// including the word text, definition, pronunciation, and timestamps.
///
/// # Security
///
/// Requires authentication and administrative privileges. This is a destructive
/// operation that permanently removes data from the database and should only
/// be accessible to trusted administrators.
///
/// # Path Parameters
///
/// * `id` - The unique database identifier of the word to delete
///
/// # Caution
///
/// This operation is permanent and cannot be undone. The word and all its
/// associated data will be completely removed from the database.
///
/// # Returns
///
/// * `200 OK` - Word successfully deleted from the database
/// * `404 Not Found` - No word exists with the specified ID
/// * `500 Internal Server Error` - Database error during deletion
///
/// # Response Format
///
/// Returns an empty response body with a 200 status code on successful deletion.
#[utoipa::path(
    delete,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "admin_words_delete_by_id",
    tag = "administration_endpoints",
    request_body(content = u32, description = "Word to delete from the database", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} deleted successfully"),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word id to delete Word for"),
    )
)]
pub async fn word_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<(), AppError> {
    Word::delete(dbpool, id).await
}
