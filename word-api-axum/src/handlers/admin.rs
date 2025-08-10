//! Administrative word management endpoints (authenticated)
//!
//! Provides CRUD operations for word database management. All endpoints
//! require authentication and return JSON responses.
use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::models::word::{UpsertWord, Word};
use crate::state::AppState;

/// Lists all words in the database as a JSON array.
///
/// Returns every word stored in the database without filtering or pagination.
/// Requires authentication and administrative privileges.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
///
/// # Returns
///
/// * `200 OK` - JSON array containing all words with complete information
/// * `404 Not Found` - Database is empty or no words are available
/// * `500 Internal Server Error` - Database connection or query error
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/{lang}/words",
    operation_id = "admin_words_list_all",
    tag = "administration_endpoints",

    responses(
        (status = 200, description = "Listed every single word successfully", body = [Word]),
        (status = 404, description = "Couldn't list words. Does your database contain any?"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("lang" = String, Path, description = "Language code for word operations. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
    )
)]
pub async fn word_list(
    State(state): State<AppState>,
    Path(lang): Path<String>,
) -> Result<Json<Vec<Word>>, AppError> {
    Word::list(state.dbpool, &lang).await.map(Json::from)
}

/// Creates a new word entry in the database.
///
/// Accepts a JSON payload with word data and creates a new entry after validation.
/// All text fields are automatically converted to lowercase for consistency.
/// Requires authentication and administrative privileges.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
///
/// # Request Body
///
/// JSON object with required fields: `word`, `definition`, `pronunciation`, `word_type`.
/// All fields must pass validation (valid lemma, dictionary text, IPA notation).
///
/// # Returns
///
/// * `200 OK` - Word successfully created with generated ID
/// * `415 Unsupported Media Type` - Invalid content type
/// * `422 Unprocessable Entity` - Validation failed
/// * `500 Internal Server Error` - Database error
#[utoipa::path(
    post,
    context_path = "/admin",
    path = "/{lang}/words",
    operation_id = "admin_words_create",
    tag = "administration_endpoints",

    request_body(content = UpsertWord, description = "Word data to add to the database with validation. Must include word, definition, pronunciation, and word_type fields", content_type = "application/json"),
    responses(
        (status = 200, description = "Word successfully created and added to the database", body = [Word]),
        (status = 415, description = "Please provide a valid word with all required fields (word, definition, pronunciation, word_type) in your JSON body"),
        (status = 422, description = "Validation failed - ensure word, definition, pronunciation are properly formatted and word_type is one of: noun, verb, adjective, adverb"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word creation. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
    )
)]
pub async fn word_create(
    State(state): State<AppState>,
    Path(lang): Path<String>,
    Json(new_word): Json<UpsertWord>,
) -> Result<Json<Vec<Word>>, AppError> {
    Word::create(state.dbpool, &lang, new_word)
        .await
        .map(Json::from)
}

/// Retrieves a specific word by its database ID.
///
/// Fetches a single word using its unique identifier. Provides administrators
/// access to complete word records including metadata.
/// Requires authentication and administrative privileges.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
/// * `id` - Unique database identifier of the word to retrieve
///
/// # Returns
///
/// * `200 OK` - Word found and returned with all fields
/// * `404 Not Found` - No word exists with specified ID
/// * `500 Internal Server Error` - Database connection or query error
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/{lang}/words/{id}",
    operation_id = "admin_words_read_by_id",
    tag = "administration_endpoints",

    responses (
        (status = 200, description = "Word with specified ID returned successfully", body = [Word]),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word retrieval. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
        ("id" = u32, Path, description = "Unique database identifier of the word to retrieve", example = 1),
    )
)]
pub async fn word_read(
    State(state): State<AppState>,
    Path((lang, id)): Path<(String, u32)>,
) -> Result<Json<Vec<Word>>, AppError> {
    Word::read(state.dbpool, &lang, id).await.map(Json::from)
}

/// Updates an existing word entry in the database.
///
/// Modifies an existing word identified by its database ID. All text fields
/// are converted to lowercase and the updated_at timestamp is set automatically.
/// Requires authentication and administrative privileges.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
/// * `id` - Unique database identifier of the word to update
///
/// # Request Body
///
/// JSON object with required fields: `word`, `definition`, `pronunciation`, `word_type`.
/// Must pass same validation as word creation.
///
/// # Returns
///
/// * `200 OK` - Word successfully updated
/// * `404 Not Found` - No word exists with specified ID
/// * `422 Unprocessable Entity` - Validation failed
/// * `500 Internal Server Error` - Database error
#[utoipa::path(
    put,
    context_path = "/admin",
    path = "/{lang}/words/{id}",
    operation_id = "admin_words_update_by_id",
    tag = "administration_endpoints",

    request_body(content = UpsertWord, description = "Word data to update in the database. Must include word, definition, pronunciation, and word_type fields", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} updated successfully", body = [Word]),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word update. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
        ("id" = u32, Path, description = "Unique database identifier of the word to update", example = 1),
    )
)]
pub async fn word_update(
    State(state): State<AppState>,
    Path((lang, id)): Path<(String, u32)>,
    Json(updated_word): Json<UpsertWord>,
) -> Result<Json<Vec<Word>>, AppError> {
    Word::update(state.dbpool, &lang, id, updated_word)
        .await
        .map(Json::from)
}

/// Permanently removes a word from the database.
///
/// Deletes a word record by its database ID. This operation is irreversible
/// and permanently removes all associated data.
/// Requires authentication and administrative privileges.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
/// * `id` - Unique database identifier of the word to delete
///
/// # Returns
///
/// * `200 OK` - Word successfully deleted
/// * `404 Not Found` - No word exists with specified ID
/// * `500 Internal Server Error` - Database error during deletion
#[utoipa::path(
    delete,
    context_path = "/admin",
    path = "/{lang}/words/{id}",
    operation_id = "admin_words_delete_by_id",
    tag = "administration_endpoints",

    responses (
        (status = 200, description = "Word successfully deleted from the database"),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word deletion. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
        ("id" = u32, Path, description = "Unique database identifier of the word to delete", example = 1),
    )
)]
pub async fn word_delete(
    State(state): State<AppState>,
    Path((lang, id)): Path<(String, u32)>,
) -> Result<(), AppError> {
    Word::delete(state.dbpool, &lang, id).await
}
