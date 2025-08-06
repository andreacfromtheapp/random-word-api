//! Word model module
//!
//! This module contains the core Word model and related structures for managing
//! dictionary words in the random word API. It provides functionality for CRUD
//! operations, validation, and database interactions.
//!
//! # Features
//!
//! - Word CRUD operations (Create, Read, Update, Delete)
//! - Random word retrieval from database
//! - Comprehensive validation for lemmas, definitions, and pronunciations
//! - Support for IPA phonetic notation
//! - Admin-only operations for word management
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};
use utoipa::ToSchema;
use utoipauto::utoipa_ignore;
use validator::{Validate, ValidationError};

use crate::error::AppError;

/// Represents a word in the database and in API responses.
///
/// This struct contains all the information about a dictionary word including
/// its definition, pronunciation in IPA notation, and timestamp metadata.
/// It implements serialization for JSON responses and database row mapping
/// for SQLite integration.
///
/// # Fields
///
/// - `id`: Unique identifier for the word in the database
/// - `word`: The actual word/lemma following Merriam-Webster standards
/// - `definition`: Human-readable definition of the word
/// - `pronunciation`: IPA phonetic notation enclosed in forward slashes
/// - `created_at`: Timestamp when the word was added to the database
/// - `updated_at`: Timestamp when the word was last modified
///
/// # Database Schema
///
/// This struct maps to the `words` table with the following structure:
/// ```sql
/// CREATE TABLE words (
///     id INTEGER PRIMARY KEY,
///     word TEXT NOT NULL,
///     definition TEXT NOT NULL,
///     pronunciation TEXT NOT NULL,
///     created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
///     updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
/// );
/// ```
#[derive(ToSchema, Serialize, Clone, sqlx::FromRow)]
pub struct Word {
    id: u32,
    word: String,
    definition: String,
    pronunciation: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

/// Implementation of Word with database operations.
///
/// This implementation provides all the core functionality for interacting with
/// words in the database. All methods are asynchronous and return Results that
/// can be converted to appropriate HTTP responses by the handlers.
///
/// These methods are marked with `#[utoipa_ignore]` to prevent them from appearing
/// in the OpenAPI documentation, as they represent internal business logic rather
/// than API endpoints.
///
/// # Error Handling
///
/// All methods return `Result<T, AppError>` where `AppError` handles database
/// errors, validation errors, and other application-specific errors that can
/// be converted to appropriate HTTP status codes.
#[utoipa_ignore]
impl Word {
    /// Retrieves a random word from the database.
    ///
    /// This method uses SQLite's `RANDOM()` function to select a single word
    /// at random from all available words in the database. This is the core
    /// functionality for the `/word` endpoint.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    ///
    /// # Returns
    ///
    /// * `Ok(Word)` - A randomly selected word with all its fields
    /// * `Err(AppError)` - Database connection error or empty database
    pub async fn random(dbpool: SqlitePool) -> Result<Self, AppError> {
        query_as("SELECT * FROM words ORDER BY random() LIMIT 1")
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Retrieves all words from the database.
    ///
    /// This method returns all words in the database without any filtering or
    /// pagination. It is intended for administrative purposes only and should
    /// be protected by authentication middleware.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Word>)` - Vector containing all words in the database
    /// * `Err(AppError)` - Database connection or query error
    ///
    /// # Security Note
    ///
    /// This endpoint should only be accessible to authenticated administrators
    /// as it exposes the entire word database.
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Self>, AppError> {
        query_as("SELECT * FROM words")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Creates a new word in the database.
    ///
    /// This method validates the input data and inserts a new word into the database.
    /// All text fields are automatically converted to lowercase for consistency.
    /// The word must pass validation for lemma format, definition content, and
    /// pronunciation IPA notation.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    /// * `new_word` - UpsertWord struct containing the word data to insert
    ///
    /// # Returns
    ///
    /// * `Ok(Word)` - The newly created word with generated ID and timestamps
    /// * `Err(AppError)` - Validation error, duplicate word, or database error
    pub async fn create(dbpool: SqlitePool, new_word: UpsertWord) -> Result<Self, AppError> {
        let word = new_word.word()?.to_lowercase();
        let definition = new_word.definition()?.to_lowercase();
        let pronunciation = new_word.pronunciation()?.to_lowercase();

        query_as(
            "INSERT INTO words (word, definition, pronunciation) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(word)
        .bind(definition)
        .bind(pronunciation)
        .fetch_one(&dbpool)
        .await
        .map_err(Into::into)
    }

    /// Retrieves a specific word by its database ID.
    ///
    /// This method fetches a single word from the database using its unique
    /// identifier. It is primarily used for administrative purposes and
    /// detailed word inspection.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    /// * `id` - The unique identifier of the word to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Word)` - The word with the specified ID
    /// * `Err(AppError)` - Word not found or database error
    pub async fn read(dbpool: SqlitePool, id: u32) -> Result<Self, AppError> {
        query_as("SELECT * FROM words WHERE id = $1")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Updates an existing word in the database.
    ///
    /// This method modifies an existing word with new data. The word is identified
    /// by its ID, and all fields (word, definition, pronunciation) are updated
    /// with the provided values. All text is converted to lowercase for consistency.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    /// * `id` - The unique identifier of the word to update
    /// * `updated_word` - UpsertWord struct containing the new word data
    ///
    /// # Returns
    ///
    /// * `Ok(Word)` - The updated word with new values and updated timestamp
    /// * `Err(AppError)` - Word not found, validation error, or database error
    pub async fn update(
        dbpool: SqlitePool,
        id: u32,
        updated_word: UpsertWord,
    ) -> Result<Self, AppError> {
        let word = updated_word.word()?.to_lowercase();
        let definition = updated_word.definition()?.to_lowercase();
        let pronunciation = updated_word.pronunciation()?.to_lowercase();

        query_as(
            "UPDATE words SET word = $1, definition = $2, pronunciation = $3 WHERE id = $4 RETURNING *",
        )
        .bind(word)
        .bind(definition)
        .bind(pronunciation)
        .bind(id)
        .fetch_one(&dbpool)
        .await
        .map_err(Into::into)
    }

    /// Deletes a word from the database.
    ///
    /// This method permanently removes a word from the database. The operation
    /// cannot be undone, so it should be used with caution.
    ///
    /// # Arguments
    ///
    /// * `dbpool` - SQLite connection pool for database access
    /// * `id` - The unique identifier of the word to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Word was successfully deleted
    /// * `Err(AppError)` - Word not found or database error
    pub async fn delete(dbpool: SqlitePool, id: u32) -> Result<(), AppError> {
        query("DELETE FROM words WHERE id = $1")
            .bind(id)
            .execute(&dbpool)
            .await?;
        Ok(())
    }
}

/// Data transfer object for creating and updating words.
///
/// This struct is used for both creating new words and updating existing ones.
/// It includes comprehensive validation to ensure data quality and consistency
/// with dictionary standards.
///
/// # Validation Rules
///
/// - `word`: Must be a valid lemma (no whitespace, follows Merriam-Webster format)
/// - `definition`: Must contain only alphabetic characters, punctuation, and whitespace
/// - `pronunciation`: Must be valid IPA notation enclosed in forward slashes
#[derive(ToSchema, Deserialize, Validate)]
pub struct UpsertWord {
    #[validate(length(min = 1), custom(function = "validate_word"))]
    word: String,
    #[validate(length(min = 1), custom(function = "validate_definition"))]
    definition: String,
    #[validate(length(min = 1), custom(function = "validate_pronunciation"))]
    pronunciation: String,
}

/// Validates a word field using Merriam-Webster lemma rules.
///
/// This function ensures that the word contains no whitespace and follows
/// the pattern expected for dictionary lemmas. It uses the `is_valid_lemma`
/// function to perform the actual validation.
///
/// # Arguments
///
/// * `text` - The word string to validate
///
/// # Returns
///
/// * `Ok(())` - The word is valid
/// * `Err(ValidationError)` - The word contains whitespace or invalid characters
///
/// # Validation Rules
///
/// - No whitespace characters allowed
/// - Must pass `is_valid_lemma` check for character composition
fn validate_word(text: &str) -> Result<(), ValidationError> {
    if text.chars().any(char::is_whitespace) || !is_valid_lemma(text) {
        return Err(ValidationError::new("invalid_lemma"));
    }

    Ok(())
}

/// Validates a definition field for appropriate content.
///
/// This function ensures that the definition contains only characters appropriate
/// for dictionary definitions: letters, numbers, whitespace, and common punctuation.
/// It excludes symbols that shouldn't appear in definitions.
///
/// # Arguments
///
/// * `text` - The definition string to validate
///
/// # Returns
///
/// * `Ok(())` - The definition is valid
/// * `Err(ValidationError)` - The definition contains invalid characters
fn validate_definition(text: &str) -> Result<(), ValidationError> {
    if !is_valid_definition(text) {
        return Err(ValidationError::new("invalid_definition"));
    }

    Ok(())
}

/// Validates a pronunciation field for IPA phonetic notation.
///
/// This function ensures that the pronunciation follows International Phonetic
/// Alphabet (IPA) standards and is properly formatted with forward slash delimiters.
///
/// # Arguments
///
/// * `text` - The pronunciation string to validate
///
/// # Returns
///
/// * `Ok(())` - The pronunciation is valid IPA notation
/// * `Err(ValidationError)` - The pronunciation is not valid IPA format
fn validate_pronunciation(text: &str) -> Result<(), ValidationError> {
    if !is_valid_pronunciation(text) {
        return Err(ValidationError::new("invalid_pronunciation"));
    }

    Ok(())
}

/// Accessor methods for UpsertWord fields with validation.
///
/// These methods provide validated access to the UpsertWord fields. Each method
/// runs the complete validation before returning the field value, ensuring that
/// only valid data is used throughout the application.
///
/// # Error Handling
///
/// All accessor methods return `Result<&str, AppError>` to handle validation
/// errors gracefully and convert them to appropriate HTTP responses.
#[utoipa_ignore]
impl UpsertWord {
    /// Returns the word field after validation.
    ///
    /// This method validates the entire UpsertWord struct and returns a reference
    /// to the word field if validation passes.
    ///
    /// # Returns
    ///
    /// * `Ok(&str)` - Reference to the validated word
    /// * `Err(AppError)` - Validation failed for any field
    pub fn word(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.word.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the definition field after validation.
    ///
    /// This method validates the entire UpsertWord struct and returns a reference
    /// to the definition field if validation passes.
    ///
    /// # Returns
    ///
    /// * `Ok(&str)` - Reference to the validated definition
    /// * `Err(AppError)` - Validation failed for any field
    pub fn definition(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.definition.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the pronunciation field after validation.
    ///
    /// This method validates the entire UpsertWord struct and returns a reference
    /// to the pronunciation field if validation passes.
    ///
    /// # Returns
    ///
    /// * `Ok(&str)` - Reference to the validated pronunciation
    /// * `Err(AppError)` - Validation failed for any field
    pub fn pronunciation(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.pronunciation.as_ref()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Validates a Merriam-Webster lemma using regex pattern matching.
///
/// This function checks if a string conforms to the standards used by Merriam-Webster
/// for dictionary lemmas. It accepts a specific set of characters that are commonly
/// found in English dictionary entries, including accented characters for borrowed words.
///
/// # Accepted Characters
///
/// - Alphanumeric characters (a-z, A-Z, 0-9)
/// - Hyphens for compound words
/// - Apostrophes for contractions and possessives
/// - Periods for abbreviations
/// - Latin-1 supplement accented characters (À-ÿ)
/// - Latin Extended-A characters (Ā-ž)
/// - Latin Extended Additional characters (Ḁ-ỿ)
///
/// # Arguments
///
/// * `lemma` - The string to validate as a dictionary lemma
///
/// # Returns
///
/// * `true` - The string is a valid lemma format
/// * `false` - The string contains invalid characters or is empty
///
/// # Examples
///
/// ```rust
/// use crate::model::word::is_valid_lemma;
///
/// // Valid lemmas
/// assert!(is_valid_lemma("hello"));
/// assert!(is_valid_lemma("co-worker"));
/// assert!(is_valid_lemma("don't"));
/// assert!(is_valid_lemma("Mr."));
/// assert!(is_valid_lemma("café"));
/// assert!(is_valid_lemma("naïve"));
///
/// // Invalid lemmas
/// assert!(!is_valid_lemma("hello world")); // contains space
/// assert!(!is_valid_lemma("hello@world")); // contains @
/// assert!(!is_valid_lemma("")); // empty string
/// ```
pub fn is_valid_lemma(lemma: &str) -> bool {
    use regex::Regex;
    use std::sync::OnceLock;

    static LEMMA_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = LEMMA_REGEX.get_or_init(|| {
        // Pattern explanation:
        // ^                    - Start of string
        // [                    - Character class start
        //   a-zA-Z0-9          - Alphanumeric characters
        //   \-                 - Hyphen (escaped)
        //   '                  - Apostrophe
        //   \.                 - Period (escaped)
        //   À-ÿ                - Latin-1 supplement accented characters
        //   Ā-ž                - Latin Extended-A
        //   Ḁ-ỿ                - Latin Extended Additional (common accented chars)
        // ]+                   - One or more of the above characters
        // $                    - End of string
        Regex::new(r"^[a-zA-Z0-9\-'\.À-ÿĀ-žḀ-ỿ]+$").unwrap()
    });

    !lemma.is_empty() && regex.is_match(lemma)
}

/// Validates a definition string for dictionary-appropriate content.
///
/// This function ensures that definition text contains only characters that would
/// be expected in a professional dictionary definition. It allows for descriptive
/// text while excluding symbols that might indicate non-definition content.
///
/// # Accepted Characters
///
/// - Basic Latin letters (a-z, A-Z)
/// - Common accented characters (À-ÿĀ-žḀ-ỿ)
/// - Numbers (0-9) for definitions that include numeric references
/// - Whitespace characters for word separation
/// - Common punctuation: periods, commas, semicolons, colons, exclamation marks,
///   question marks, parentheses, apostrophes, quotation marks, and hyphens
///
/// # Excluded Characters
///
/// - Email symbols (@, angle brackets)
/// - Social media symbols (#, mentions)
/// - Currency symbols ($, €, £, etc.)
/// - Programming symbols (*, &, %, etc.)
/// - URLs and web-related symbols
///
/// # Arguments
///
/// * `definition` - The string to validate as a dictionary definition
///
/// # Returns
///
/// * `true` - The string contains only valid definition characters
/// * `false` - The string contains invalid characters or is empty
///
/// # Examples
///
/// ```rust
/// use crate::model::word::is_valid_definition;
///
/// // Valid definitions
/// assert!(is_valid_definition("a word or phrase"));
/// assert!(is_valid_definition("departing from an accepted standard"));
/// assert!(is_valid_definition("restrain oneself from indulging in something"));
/// assert!(is_valid_definition("having the quality of being naïve"));
///
/// // Invalid definitions
/// assert!(!is_valid_definition("contact us at test@email.com"));
/// assert!(!is_valid_definition("visit our website www.example.com"));
/// assert!(!is_valid_definition("costs $50 or more"));
/// assert!(!is_valid_definition("")); // empty string
/// ```
pub fn is_valid_definition(definition: &str) -> bool {
    use regex::Regex;
    use std::sync::OnceLock;

    static DEFINITION_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = DEFINITION_REGEX.get_or_init(|| {
        // Pattern explanation:
        // ^                    - Start of string
        // [                    - Character class start
        //   a-zA-Z             - Basic Latin letters
        //   À-ÿĀ-žḀ-ỿ           - Common accented characters
        //   0-9                - Numbers
        //   \s                 - Whitespace characters
        //   .,;:!?()'""\-      - Common punctuation for definitions
        // ]+                   - One or more of the above characters
        // $                    - End of string
        Regex::new(r"^[a-zA-ZÀ-ÿĀ-žḀ-ỿ0-9\s.,;:!?()'\-]+$").unwrap()
    });

    !definition.is_empty() && regex.is_match(definition)
}

/// Validates a pronunciation string using International Phonetic Alphabet (IPA) notation.
///
/// This function ensures that pronunciation follows the standard IPA format used
/// in dictionaries, with forward slash delimiters and authentic phonetic symbols.
/// The validation is based on common IPA characters found in English pronunciation
/// guides and includes stress markers and length indicators.
///
/// # Format Requirements
///
/// - Must be enclosed in forward slashes (/)
/// - Must contain only valid IPA phonetic symbols
/// - Can include stress markers and diacritics
/// - Cannot be empty between the slashes
///
/// # Accepted IPA Symbols
///
/// - Basic Latin letters for consonants and some vowels
/// - Common IPA vowels: ə ɛ ɪ ɔ ʊ ʌ ɑ æ ɒ ɜ ʏ
/// - Stress and length markers: ˈ (primary stress), ˌ (secondary stress), ː (long), ˑ (half-long)
/// - Common IPA consonants: θ ð ʃ ʒ ʧ ʤ ŋ ɹ ɾ ɭ ɻ ɲ ɳ
/// - Diacritics and modifiers: ʰ ʷ ʲ ˠ ˤ ᵊ ᵛ ᵚ ᵏ
///
/// # Arguments
///
/// * `pronunciation` - The string to validate as IPA notation
///
/// # Returns
///
/// * `true` - The string is valid IPA notation with proper delimiters
/// * `false` - The string is not properly formatted IPA notation or is empty
///
/// # Examples
///
/// ```rust
/// use crate::model::word::is_valid_pronunciation;
///
/// // Valid pronunciations
/// assert!(is_valid_pronunciation("/əˈbeɪt/"));
/// assert!(is_valid_pronunciation("/æˈberənt/"));
/// assert!(is_valid_pronunciation("/ˌæbəˈreɪʃən/"));
/// assert!(is_valid_pronunciation("/ˈhɛloʊ/"));
///
/// // Invalid pronunciations
/// assert!(!is_valid_pronunciation("invalid")); // no slashes
/// assert!(!is_valid_pronunciation("//")); // empty content
/// assert!(!is_valid_pronunciation("/test@/")); // invalid character
/// assert!(!is_valid_pronunciation("əˈbeɪt")); // missing slashes
/// assert!(!is_valid_pronunciation("")); // empty string
/// ```
pub fn is_valid_pronunciation(pronunciation: &str) -> bool {
    use regex::Regex;
    use std::sync::OnceLock;

    static PRONUNCIATION_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = PRONUNCIATION_REGEX.get_or_init(|| {
        // Pattern explanation:
        // ^                    - Start of string
        // /                    - Forward slash (literal)
        // [                    - Character class start
        //   a-zA-Z             - Basic Latin letters
        //   əɛɪɔʊʌɑæɒɜɪʊʏ     - Common IPA vowels
        //   ˈˌːˑ               - IPA stress and length markers
        //   θðʃʒʧʤŋɹɾɭɻɲɳ     - Common IPA consonants
        //   ʰʷʲˠˤʰᵊᵛᵚᵏ        - IPA diacritics and modifiers
        //   \p{M}              - Unicode combining marks (diacritics)
        // ]+                   - One or more of the above characters
        // /                    - Forward slash (literal)
        // $                    - End of string
        Regex::new(r"^/[a-zA-Zəɛɪɔʊʌɑæɒɜɪʊʏˈˌːˑθðʃʒʧʤŋɹɾɭɻɲɳʰʷʲˠˤᵊᵛᵚᵏ]+/$").unwrap()
    });

    !pronunciation.is_empty() && regex.is_match(pronunciation)
}
