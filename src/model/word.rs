// Model for Word and its methods to use with handlers
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};
use utoipa::ToSchema;
use utoipauto::utoipa_ignore;
use validator::{Validate, ValidationError};

use crate::error::AppError;

/// Represents a word in the database and in API responses
#[derive(ToSchema, Serialize, Clone, sqlx::FromRow)]
pub struct Word {
    id: u32,
    word: String,
    definition: String,
    pronunciation: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

/// Word implemnentation of actual random, list, and CRUD operations
///
/// These are encapusulated and not publicly available on any of the doc UIs (swagger, rapidoc, redoc, scalar)
#[utoipa_ignore]
impl Word {
    /// Return a random word from /word
    pub async fn random(dbpool: SqlitePool) -> Result<Self, AppError> {
        query_as("SELECT * FROM words ORDER BY random() LIMIT 1")
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// List all words (admin only)
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Self>, AppError> {
        query_as("SELECT * FROM words")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Add a new word to the database (admin only)
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

    /// Get a single word by id (admin only)
    pub async fn read(dbpool: SqlitePool, id: u32) -> Result<Self, AppError> {
        query_as("SELECT * FROM words WHERE id = $1")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Update an existing word (admin only)
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

    /// Delete a word from the database (admin only)
    pub async fn delete(dbpool: SqlitePool, id: u32) -> Result<(), AppError> {
        query("DELETE FROM words WHERE id = $1")
            .bind(id)
            .execute(&dbpool)
            .await?;
        Ok(())
    }
}

/// Represents a word with create and update
#[derive(ToSchema, Deserialize, Validate)]
pub struct UpsertWord {
    #[validate(length(min = 1), custom(function = "validate_word"))]
    word: String,
    #[validate(length(min = 1), custom(function = "validate_definition"))]
    definition: String,
    #[validate(length(min = 1), custom(function = "validate_pronunciation"))]
    pronunciation: String,
}

/// Validate word using lemma rules
fn validate_word(text: &str) -> Result<(), ValidationError> {
    if text.chars().any(char::is_whitespace) || !is_valid_lemma(text) {
        return Err(ValidationError::new("invalid_lemma"));
    }

    Ok(())
}

/// Validate definition contains only alphabetic, punctuation, and whitespace characters
fn validate_definition(text: &str) -> Result<(), ValidationError> {
    if !is_valid_definition(text) {
        return Err(ValidationError::new("invalid_definition"));
    }

    Ok(())
}

/// Validate pronunciation uses IPA phonetic notation
fn validate_pronunciation(text: &str) -> Result<(), ValidationError> {
    if !is_valid_pronunciation(text) {
        return Err(ValidationError::new("invalid_pronunciation"));
    }

    Ok(())
}

/// Accessors (getters) helpers
#[utoipa_ignore]
impl UpsertWord {
    pub fn word(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.word.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn definition(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.definition.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn pronunciation(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.pronunciation.as_ref()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Validates a Merriam-Webster lemma using regex.
///
/// Accepts alphanumeric characters, hyphens, apostrophes, periods, and common accented characters.
///
/// # Examples
///
/// ```
/// use crate::model::word::is_valid_lemma;
/// assert!(is_valid_lemma("hello"));
/// assert!(is_valid_lemma("co-worker"));
/// assert!(is_valid_lemma("don't"));
/// assert!(is_valid_lemma("Mr."));
/// assert!(is_valid_lemma("café"));
/// assert!(!is_valid_lemma("hello@world"));
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

/// Validates a definition string using regex.
///
/// Accepts alphabetic characters, common punctuation, and whitespace characters.
/// Excludes symbols like @, #, $, etc. that shouldn't appear in dictionary definitions.
///
/// # Examples
///
/// ```
/// use crate::model::word::is_valid_definition;
/// assert!(is_valid_definition("a word or phrase"));
/// assert!(is_valid_definition("departing from an accepted standard"));
/// assert!(is_valid_definition("restrain oneself from indulging in something"));
/// assert!(!is_valid_definition("test@email.com"));
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

/// Validates a pronunciation string using IPA phonetic notation.
///
/// Accepts IPA symbols enclosed in forward slashes, common phonetic characters,
/// and standard IPA notation based on existing pronunciation patterns.
///
/// # Examples
///
/// ```
/// use crate::model::word::is_valid_pronunciation;
/// assert!(is_valid_pronunciation("/əˈbeɪt/"));
/// assert!(is_valid_pronunciation("/æˈberənt/"));
/// assert!(is_valid_pronunciation("/ˌæbəˈreɪʃən/"));
/// assert!(!is_valid_pronunciation("invalid"));
/// assert!(!is_valid_pronunciation("/test@/"));
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
