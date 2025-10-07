//! Public endpoints for retrieving random words
//!
//! Provides random word retrieval with optional filtering by grammatical type.
//! All endpoints are publicly accessible and return JSON responses.

use crate::error::AppError;
use crate::models::word::GetWord;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

/// Retrieves a random word from the database.
///
/// Returns a randomly selected word using SQLite's RANDOM() function for fair
/// distribution across all available words in the specified language database.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
///
/// # Returns
///
/// * `200 OK` - Random word successfully retrieved and returned
/// * `400 Bad Request` - Invalid language code provided
/// * `404 Not Found` - No words available in database
/// * `500 Internal Server Error` - Database connection or query error
#[utoipa::path(
    get,
    path = "/{lang}/random",
    operation_id = "public_word_random",
    tag = "public_endpoints",

    responses(
        (status = 200, description = "Random word successfully retrieved and returned", body = [GetWord]),
        (status = 400, description = "Bad Request - Invalid language code provided"),
        (status = 404, description = "Not Found - No words available in the specified language database"),
        (status = 500, description = "Internal Server Error - Database connection or query error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word retrieval. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
    )
)]
pub async fn word_random(
    State(state): State<AppState>,
    Path(lang): Path<String>,
) -> Result<Json<Vec<GetWord>>, AppError> {
    GetWord::random_word(state.dbpool, &lang)
        .await
        .map(Json::from)
}

/// Retrieves a random word of a specific grammatical type from the database.
///
/// Returns a randomly selected word filtered by grammatical type using SQLite's
/// RANDOM() function for fair distribution within the specified word category.
///
/// # Parameters
///
/// * `lang` - Language code (currently supports 'en' for English; future versions
///   will support additional languages with separate database tables)
/// * `type` - Grammatical type filter with accepted values:
///   - 'noun' (people, places, things)
///   - 'verb' (actions, states)
///   - 'adjective' (descriptive words)
///   - 'adverb' (modifiers)
///   - 'pronoun' (words that replace nouns)
///   - 'preposition' (words showing relationships)
///   - 'conjunction' (connecting words)
///   - 'interjection' (exclamatory words)
///   - 'article' (definite and indefinite articles)
///
/// # Returns
///
/// * `200 OK` - Random word of specified type successfully retrieved
/// * `400 Bad Request` - Invalid language code or unsupported word type
/// * `404 Not Found` - No words of specified type available in database
/// * `500 Internal Server Error` - Database connection or query error
#[utoipa::path(
    get,
    path = "/{lang}/{type}",
    operation_id = "public_word_random_type",
    tag = "public_endpoints",

    responses(
        (status = 200, description = "Random word of specified type successfully retrieved and returned", body = [GetWord]),
        (status = 400, description = "Bad Request - Invalid language code or unsupported word type provided"),
        (status = 404, description = "Not Found - No words of specified type available in the language database"),
        (status = 500, description = "Internal Server Error - Database connection or query error"),
    ),
    params(
        ("lang" = String, Path, description = "Language code for word retrieval. Currently supports: 'en' (English). Future versions will support additional languages with separate database tables.", example = "en"),
        ("type" = String, Path, description = "Grammatical type filter for word selection. Accepted values: 'noun' (people, places, things), 'verb' (actions, states), 'adjective' (descriptive words), 'adverb' (modifiers), 'pronoun' (words that replace nouns), 'preposition' (words showing relationships), 'conjunction' (connecting words), 'interjection' (exclamatory words), 'article' (definite and indefinite articles).", example = "noun"),
    )
)]
pub async fn word_type(
    State(state): State<AppState>,
    Path((lang, word_type)): Path<(String, String)>,
) -> Result<Json<Vec<GetWord>>, AppError> {
    GetWord::random_type(state.dbpool, &lang, &word_type)
        .await
        .map(Json::from)
}

#[cfg(test)]
mod tests {

    use crate::error::{AppError, PathError};
    use crate::models::word::{GrammaticalType, LanguageCode};
    use std::str::FromStr;

    #[test]
    fn test_language_validation_logic() {
        // Test language validation used by word handlers
        let valid_lang = LanguageCode::from_str("en");
        assert!(valid_lang.is_ok());
        assert_eq!(valid_lang.unwrap().table_name(), "words");

        let invalid_lang = LanguageCode::from_str("xyz");
        assert!(invalid_lang.is_err());

        // Test PathError creation for invalid language
        let path_error = PathError::InvalidPath("xyz".to_string());
        let app_error = AppError::from(path_error);
        let error_debug = format!("{app_error:?}");
        assert!(error_debug.contains("xyz") || error_debug.contains("InvalidPath"));
    }

    #[test]
    fn test_word_type_validation_logic() {
        // Test word type validation used by handlers
        let valid_word_type = GrammaticalType::from_str("noun");
        assert!(valid_word_type.is_ok());

        let invalid_word_type = GrammaticalType::from_str("determiner");
        assert!(invalid_word_type.is_err());

        // Test PathError creation for invalid word_type
        let path_error = PathError::InvalidWordType("determiner".to_string());
        let app_error = AppError::from(path_error);
        let error_debug = format!("{app_error:?}");
        assert!(error_debug.contains("determiner") || error_debug.contains("InvalidWordType"));
    }

    #[test]
    fn test_all_grammatical_types_supported() {
        // Test that all grammatical types are supported by the handler
        let supported_types = [
            "noun",
            "verb",
            "adjective",
            "adverb",
            "pronoun",
            "preposition",
            "conjunction",
            "interjection",
            "article",
        ];

        for word_type in supported_types {
            let result = GrammaticalType::from_str(word_type);
            assert!(
                result.is_ok(),
                "Word type '{}' should be supported",
                word_type
            );
        }

        // Test that unsupported types are rejected
        let unsupported_types = ["determiner", "particle", "auxiliary", "modal"];

        for word_type in unsupported_types {
            let result = GrammaticalType::from_str(word_type);
            assert!(
                result.is_err(),
                "Word type '{}' should not be supported",
                word_type
            );
        }
    }
}
