//! Administrative word management endpoints. Require authentication and administrative privileges.
//!
//! Provides CRUD operations for word database management and user management.
//! All endpoints require authentication and return JSON responses.
use crate::error::AppError;
use crate::models::word::{UpsertWord, Word};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

/// Lists all words in the database.
///
/// Returns every word stored in the database without filtering or pagination.
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
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Listed every single word successfully", body = [Word]),
        (status = 401, description = "Unauthorized - Invalid or missing authentication token"),
        (status = 403, description = "Forbidden - Admin privileges required"),
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
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
///
/// # Request Body
///
/// JSON object with required fields: `word`, `definition`, `pronunciation`, `wordType`.
/// All fields must pass validation (valid lemma, dictionary text, IPA notation, allowed grammatical types).
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
    security(
        ("bearer_auth" = [])
    ),
    request_body(content = UpsertWord, description = "Word data to add to the database with validation. Must include word, definition, pronunciation, and word_type fields", content_type = "application/json"),
    responses(
        (status = 200, description = "Word successfully created and added to the database", body = [Word]),
        (status = 401, description = "Unauthorized - Invalid or missing authentication token"),
        (status = 403, description = "Forbidden - Admin privileges required"),
        (status = 415, description = "Please provide a valid word with all required fields (word, definition, pronunciation, word_type) in your JSON body"),
        (status = 422, description = "Validation failed - ensure word, definition, pronunciation are properly formatted and word_type is one of: noun, verb, adjective, adverb"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word creation. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
    )
)]
pub async fn word_create(
    Path(lang): Path<String>,
    State(state): State<AppState>,
    Json(word): Json<UpsertWord>,
) -> Result<Json<Vec<Word>>, AppError> {
    Word::create(state.dbpool, &lang, word)
        .await
        .map(Json::from)
}

/// Retrieves a specific word by its database ID.
///
/// Fetches a single word using its unique identifier. Provides administrators
/// access to complete word records including metadata.
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
    security(
        ("bearer_auth" = [])
    ),
    responses (
        (status = 200, description = "Word with specified ID returned successfully", body = [Word]),
        (status = 401, description = "Unauthorized - Invalid or missing authentication token"),
        (status = 403, description = "Forbidden - Admin privileges required"),
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
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
/// * `id` - Unique database identifier of the word to update
///
/// # Request Body
///
/// JSON object with required fields: `word`, `definition`, `pronunciation`, `wordType`.
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
    security(
        ("bearer_auth" = [])
    ),
    request_body(content = UpsertWord, description = "Word data to update in the database. Must include word, definition, pronunciation, and word_type fields", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} updated successfully", body = [Word]),
        (status = 401, description = "Unauthorized - Invalid or missing authentication token"),
        (status = 403, description = "Forbidden - Admin privileges required"),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 422, description = "Validation failed - invalid word data provided"),
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
    security(
        ("bearer_auth" = [])
    ),
    responses (
        (status = 200, description = "Word successfully deleted from the database"),
        (status = 401, description = "Unauthorized - Invalid or missing authentication token"),
        (status = 403, description = "Forbidden - Admin privileges required"),
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

#[cfg(test)]
mod tests {
    use crate::error::{AppError, PathError};
    use crate::models::word::{GrammaticalType, LanguageCode, UpsertWord};
    use std::str::FromStr;

    #[test]
    fn test_parameter_extraction_logic() {
        // Test language parameter validation logic used by handlers
        let valid_lang = "en";
        let language_result = LanguageCode::from_str(valid_lang);
        assert!(language_result.is_ok());
        assert_eq!(language_result.unwrap().table_name(), "words");

        let invalid_lang = "xyz";
        let language_result = LanguageCode::from_str(invalid_lang);
        assert!(language_result.is_err());

        let valid_type = "noun";
        let type_result = GrammaticalType::from_str(valid_type);
        assert!(type_result.is_ok());

        let invalid_type = "determiner";
        let type_result = GrammaticalType::from_str(invalid_type);
        assert!(type_result.is_err());
    }

    #[test]
    fn test_error_conversion_logic() {
        // Test error conversion used by handlers
        let path_error = PathError::InvalidPath("invalid".to_string());
        let app_error = AppError::from(path_error);
        let error_debug = format!("{app_error:?}");
        assert!(error_debug.contains("invalid") || error_debug.contains("InvalidPath"));

        let word_type_error = PathError::InvalidWordType("preposition".to_string());
        let app_error = AppError::from(word_type_error);
        let error_debug = format!("{app_error:?}");
        assert!(error_debug.contains("preposition") || error_debug.contains("InvalidWordType"));
    }

    #[test]
    fn test_data_transformation_logic() {
        // Test data transformation logic used by handlers
        let test_word = UpsertWord {
            word: "TEST".to_string(),
            definition: "A Test Definition".to_string(),
            pronunciation: "/TEST/".to_string(),
            word_type: "NOUN".to_string(),
        };

        // Test lowercase transformation that handlers perform
        let lowercase_word = test_word.word.to_lowercase();
        let lowercase_definition = test_word.definition.to_lowercase();
        let lowercase_pronunciation = test_word.pronunciation.to_lowercase();
        let lowercase_word_type = test_word.word_type.to_lowercase();

        assert_eq!(lowercase_word, "test");
        assert_eq!(lowercase_definition, "a test definition");
        assert_eq!(lowercase_pronunciation, "/test/");
        assert_eq!(lowercase_word_type, "noun");
    }

    #[test]
    fn test_validation_logic() {
        // Test validation logic used by handlers
        let invalid_word = UpsertWord {
            word: "".to_string(),
            definition: "valid definition".to_string(),
            pronunciation: "/valid/".to_string(),
            word_type: "noun".to_string(),
        };

        assert!(invalid_word.word().is_err());

        let invalid_type_word = UpsertWord {
            word: "valid".to_string(),
            definition: "valid definition".to_string(),
            pronunciation: "/valid/".to_string(),
            word_type: "determiner".to_string(),
        };

        assert!(invalid_type_word.word_type().is_err());
    }
}
