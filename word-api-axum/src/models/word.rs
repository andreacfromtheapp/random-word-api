//! Word data structure with CRUD operations and validation
//!
//! Provides database operations for managing dictionary words with lemmas,
//! definitions, and IPA pronunciations. Supports random word retrieval
//! with optional filtering by grammatical type.
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};
use std::str::FromStr;
use strum_macros::EnumString;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::error::{AppError, PathError};

///  Type-based retrieval supports common grammatical categories
///
/// - Nouns for entity-based word requests
/// - Verbs for action-based word requests
/// - Adjectives for descriptive word requests
/// - Adverbs for modifier-based word requests
///
/// Methods include language parameter validation to ensure:
/// - Only supported grammatical types are processed
/// - Proper error handling for unsupported grammatical types
/// - Future extensibility for additional grammatical types support
#[derive(Debug, PartialEq, EnumString)]
pub enum GrammaticalType {
    #[strum(serialize = "noun")]
    Noun,
    #[strum(serialize = "verb")]
    Verb,
    #[strum(serialize = "adjective")]
    Adjective,
    #[strum(serialize = "adverb")]
    Adverb,
    #[strum(serialize = "pronoun")]
    Pronoun,
    #[strum(serialize = "preposition")]
    Preposition,
    #[strum(serialize = "conjunction")]
    Conjunction,
    #[strum(serialize = "interjection")]
    Interjection,
    #[strum(serialize = "article")]
    Article,
}

impl GrammaticalType {
    pub fn type_name(&self) -> &str {
        match self {
            GrammaticalType::Noun => "noun",
            GrammaticalType::Verb => "verb",
            GrammaticalType::Adjective => "adjective",
            GrammaticalType::Adverb => "adverb",
            GrammaticalType::Pronoun => "pronoun",
            GrammaticalType::Preposition => "preposition",
            GrammaticalType::Conjunction => "conjunction",
            GrammaticalType::Interjection => "interjection",
            GrammaticalType::Article => "article",
        }
    }
}

impl std::fmt::Display for GrammaticalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammaticalType::Noun => write!(f, "noun"),
            GrammaticalType::Verb => write!(f, "verb"),
            GrammaticalType::Adjective => write!(f, "adjective"),
            GrammaticalType::Adverb => write!(f, "adverb"),
            GrammaticalType::Pronoun => write!(f, "pronoun"),
            GrammaticalType::Preposition => write!(f, "preposition"),
            GrammaticalType::Conjunction => write!(f, "conjunction"),
            GrammaticalType::Interjection => write!(f, "interjection"),
            GrammaticalType::Article => write!(f, "article"),
        }
    }
}

/// Languages support. Currently only supports American English.
///
/// Methods include language parameter validation to ensure:
/// - Only supported languages are processed
/// - Proper error handling for unsupported language codes
/// - Future extensibility for multi-language support
///
/// To add new longuages see the commented out examples.
///
/// NOTE: the database need to have the tables and data ready to
/// accommodate any additiomal language
#[derive(Debug, PartialEq, EnumString)]
pub enum LanguageCode {
    #[strum(serialize = "en")]
    English,
    // #[strum(serialize = "de")]
    // German,
    // #[strum(serialize = "fr")]
    // French,
    // #[strum(serialize = "es")]
    // Spanish,
    // #[strum(serialize = "it")]
    // Italian,
    // #[strum(serialize = "nl")]
    // Dutch,
}

impl LanguageCode {
    pub fn table_name(&self) -> &str {
        match self {
            LanguageCode::English => "words",
            // Language::German => "words_de",
            // Language::French => "words_fr",
            // Language::Spanish => "words_es",
            // Language::Italian => "words_it",
            // Language::Dutch => "words_nl",
        }
    }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageCode::English => write!(f, "en"),
            // Language::German => write!(f, "de"),
            // Language::French => write!(f, "fr"),
            // Language::Spanish => write!(f, "es"),
            // Language::Italian => write!(f, "it"),
            // Language::Dutch => write!(f, "nl"),
        }
    }
}

/// Represents a word in the database and in API responses.
///
/// This struct contains all the information about a dictionary word including
/// its definition, pronunciation in IPA notation, and timestamp metadata.
///
/// # Fields
///
/// - `id`: Unique identifier for the word in the database
/// - `word_type`: Grammatical type of the word (noun, verb, adjective, adverb)
/// - `word`: The actual word/lemma following Merriam-Webster standards
/// - `definition`: Human-readable definition of the word
/// - `pronunciation`: IPA phonetic notation enclosed in forward slashes
/// - `created_at`: Timestamp when the word was added to the database
/// - `updated_at`: Timestamp when the word was last modified
///
#[derive(ToSchema, Deserialize, Serialize, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    id: u32,
    word_type: String,
    word: String,
    definition: String,
    pronunciation: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl Word {
    /// Retrieves all words from the database (admin only)
    pub async fn list(dbpool: SqlitePool, lang: &str) -> Result<Vec<Self>, AppError> {
        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!("SELECT * FROM {}", language_code.table_name());

        // perform the actual query
        query_as(&my_query)
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Creates a new word in the database with validation
    pub async fn create(
        dbpool: SqlitePool,
        lang: &str,
        new_word: UpsertWord,
    ) -> Result<Vec<Self>, AppError> {
        let word = new_word.word()?.to_lowercase();
        let definition = new_word.definition()?.to_lowercase();
        let pronunciation = new_word.pronunciation()?.to_lowercase();
        let word_type = new_word.word_type()?.to_lowercase();

        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!("INSERT INTO {} (word, definition, pronunciation, word_type) VALUES ($1, $2, $3, $4) RETURNING *", language_code.table_name());

        // perform the actual query
        query_as(&my_query)
            .bind(word)
            .bind(definition)
            .bind(pronunciation)
            .bind(word_type)
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Retrieves a specific word by ID
    pub async fn read(dbpool: SqlitePool, lang: &str, id: u32) -> Result<Vec<Self>, AppError> {
        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!("SELECT * FROM {} WHERE id = $1", language_code.table_name());

        // perform the actual query
        query_as(&my_query)
            .bind(id)
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Updates an existing word in the database
    pub async fn update(
        dbpool: SqlitePool,
        lang: &str,
        id: u32,
        updated_word: UpsertWord,
    ) -> Result<Vec<Self>, AppError> {
        let word = updated_word.word()?.to_lowercase();
        let definition = updated_word.definition()?.to_lowercase();
        let pronunciation = updated_word.pronunciation()?.to_lowercase();
        let word_type = updated_word.word_type()?.to_lowercase();

        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!("UPDATE {} SET word = $1, definition = $2, pronunciation = $3, word_type = $4 WHERE id = $5 RETURNING *", language_code.table_name());

        // perform the actual query
        query_as(&my_query)
            .bind(word)
            .bind(definition)
            .bind(pronunciation)
            .bind(word_type)
            .bind(id)
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Deletes a word from the database
    pub async fn delete(dbpool: SqlitePool, lang: &str, id: u32) -> Result<(), AppError> {
        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!("DELETE FROM {} WHERE id = $1", language_code.table_name());

        // perform the actual query
        query(&my_query).bind(id).execute(&dbpool).await?;
        Ok(())
    }
}

/// Public word response structure for API endpoints.
///
/// This struct represents a simplified word structure used for public API responses,
/// containing only the essential word information without internal database metadata.
/// It is designed for public consumption and excludes sensitive information like
/// database IDs and timestamps that are not relevant for end users.
///
/// # Fields
///
/// - `word`: The actual word/lemma following dictionary standards
/// - `definition`: Human-readable definition of the word
/// - `pronunciation`: IPA phonetic notation enclosed in forward slashes
///
/// # Language Support
///
/// Methods include language parameter validation to ensure:
/// - Only supported languages are processed
/// - Proper error handling for unsupported language codes
/// - Future extensibility for multi-language support
///
/// # Type Filtering
///
/// The type-based retrieval supports common grammatical categories:
/// - Nouns for entity-based word requests
/// - Verbs for action-based word requests
/// - Adjectives for descriptive word requests
/// - Adverbs for modifier-based word requests
///
#[derive(ToSchema, Deserialize, Serialize, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GetWord {
    word: String,
    definition: String,
    pronunciation: String,
}

impl GetWord {
    /// Retrieves a random word from the database
    pub async fn random_word(dbpool: SqlitePool, lang: &str) -> Result<Vec<Self>, AppError> {
        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // form the query with the right table
        let my_query = format!(
            "SELECT word, definition, pronunciation FROM {} ORDER BY random() LIMIT 1",
            language_code.table_name()
        );

        // perform the actual query
        query_as(&my_query)
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Retrieves a random word of a specific grammatical type
    pub async fn random_type(
        dbpool: SqlitePool,
        lang: &str,
        word_type: &str,
    ) -> Result<Vec<Self>, AppError> {
        // if the language code is in the allowed ones
        let language_code =
            LanguageCode::from_str(lang).map_err(|_| PathError::InvalidPath(lang.to_string()))?;

        // if the grammatical type is in the allowed ones
        let grammatical_type = GrammaticalType::from_str(word_type)
            .map_err(|_| PathError::InvalidWordType(word_type.to_string()))?;

        // form the query with the right table
        let my_query = format!("SELECT word, definition, pronunciation FROM {} WHERE word_type = $1 ORDER BY random() LIMIT 1", language_code.table_name());

        // perform the actual query
        query_as(&my_query)
            .bind(grammatical_type.type_name())
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
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
/// - `word_type`: Must be one of the allowed grammatical types (noun, verb, adjective, adverb)
/// - `word`: Must be a valid lemma (no whitespace, follows Merriam-Webster format)
/// - `definition`: Must contain only alphabetic characters, punctuation, and whitespace
/// - `pronunciation`: Must be valid IPA notation enclosed in forward slashes
#[derive(ToSchema, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertWord {
    #[validate(length(min = 1), custom(function = "validate_word"))]
    pub word: String,
    #[validate(length(min = 1), custom(function = "validate_definition"))]
    pub definition: String,
    #[validate(length(min = 1), custom(function = "validate_pronunciation"))]
    pub pronunciation: String,
    #[validate(length(min = 1), custom(function = "validate_word_type"))]
    pub word_type: String,
}

/// Validates a word field using Merriam-Webster lemma rules
fn validate_word(text: &str) -> Result<(), ValidationError> {
    if !is_valid_lemma(text) {
        return Err(ValidationError::new("invalid_lemma"));
    }
    Ok(())
}

/// Validates a definition field for appropriate dictionary content
fn validate_definition(text: &str) -> Result<(), ValidationError> {
    if !is_valid_definition(text) {
        return Err(ValidationError::new("invalid_definition"));
    }
    Ok(())
}

/// Validates a pronunciation field for IPA phonetic notation
fn validate_pronunciation(text: &str) -> Result<(), ValidationError> {
    if !is_valid_pronunciation(text) {
        return Err(ValidationError::new("invalid_pronunciation"));
    }
    Ok(())
}

/// Validates a word_type field for allowed grammatical types (noun, verb, adjective, adverb)
pub fn validate_word_type(text: &str) -> Result<(), ValidationError> {
    let _ =
        GrammaticalType::from_str(text).map_err(|_| ValidationError::new("invalid_word_type"))?;
    Ok(())
}

impl UpsertWord {
    /// Returns the word field after validation
    pub fn word(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.word.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the definition field after validation
    pub fn definition(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.definition.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the pronunciation field after validation
    pub fn pronunciation(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.pronunciation.as_ref()),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the word_type field after validation
    pub fn word_type(&self) -> Result<&str, AppError> {
        match self.validate() {
            Ok(_) => Ok(self.word_type.as_ref()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Validates a Merriam-Webster lemma using regex pattern matching
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

/// Validates a definition string for dictionary-appropriate content
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

/// Validates a pronunciation string using International Phonetic Alphabet (IPA) notation
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_lemma() {
        // Valid lemmas
        assert!(is_valid_lemma("hello"));
        assert!(is_valid_lemma("co-worker"));
        assert!(is_valid_lemma("don't"));
        assert!(is_valid_lemma("Mr."));
        assert!(is_valid_lemma("café"));
        assert!(is_valid_lemma("naïve"));
        assert!(is_valid_lemma("test123"));

        // Invalid lemmas
        assert!(!is_valid_lemma("hello world")); // space
        assert!(!is_valid_lemma("hello@world")); // invalid character
        assert!(!is_valid_lemma("")); // empty
        assert!(!is_valid_lemma("test\nline")); // newline
    }

    #[test]
    fn test_is_valid_definition() {
        // Valid definitions
        assert!(is_valid_definition("a word or phrase"));
        assert!(is_valid_definition("departing from an accepted standard"));
        assert!(is_valid_definition(
            "restrain oneself from indulging in something"
        ));
        assert!(is_valid_definition("having the quality of being naïve"));
        assert!(is_valid_definition("test: definition with punctuation!"));

        // Invalid definitions
        assert!(!is_valid_definition("contact us at test@email.com")); // email
        assert!(!is_valid_definition("costs $50 or more")); // currency
        assert!(!is_valid_definition("")); // empty
        assert!(!is_valid_definition("test & more")); // ampersand
    }

    #[test]
    fn test_is_valid_pronunciation() {
        // Valid pronunciations
        assert!(is_valid_pronunciation("/əˈbeɪt/"));
        assert!(is_valid_pronunciation("/æˈberənt/"));
        assert!(is_valid_pronunciation("/ˌæbəˈreɪʃən/"));
        assert!(is_valid_pronunciation("/ˈhɛloʊ/"));
        assert!(is_valid_pronunciation("/test/"));

        // Invalid pronunciations
        assert!(!is_valid_pronunciation("invalid")); // no slashes
        assert!(!is_valid_pronunciation("//")); // empty content
        assert!(!is_valid_pronunciation("/test@/")); // invalid character
        assert!(!is_valid_pronunciation("əˈbeɪt")); // missing slashes
        assert!(!is_valid_pronunciation("")); // empty
    }

    #[test]
    fn test_validate_word_type() {
        // Valid word types
        assert!(validate_word_type("noun").is_ok());
        assert!(validate_word_type("verb").is_ok());
        assert!(validate_word_type("adjective").is_ok());
        assert!(validate_word_type("adverb").is_ok());
        assert!(validate_word_type("pronoun").is_ok());
        assert!(validate_word_type("preposition").is_ok());
        assert!(validate_word_type("conjunction").is_ok());
        assert!(validate_word_type("interjection").is_ok());
        assert!(validate_word_type("article").is_ok());

        // Invalid word types
        assert!(validate_word_type("").is_err());
        assert!(validate_word_type("invalid").is_err());
        assert!(validate_word_type("determiner").is_err());
        assert!(validate_word_type("NOUN").is_err()); // case sensitive
    }

    #[test]
    fn test_grammatical_type_from_str() {
        use std::str::FromStr;

        // Valid grammatical types
        assert_eq!(
            GrammaticalType::from_str("noun").unwrap(),
            GrammaticalType::Noun
        );
        assert_eq!(
            GrammaticalType::from_str("verb").unwrap(),
            GrammaticalType::Verb
        );
        assert_eq!(
            GrammaticalType::from_str("adjective").unwrap(),
            GrammaticalType::Adjective
        );
        assert_eq!(
            GrammaticalType::from_str("adverb").unwrap(),
            GrammaticalType::Adverb
        );
        assert_eq!(
            GrammaticalType::from_str("pronoun").unwrap(),
            GrammaticalType::Pronoun
        );
        assert_eq!(
            GrammaticalType::from_str("preposition").unwrap(),
            GrammaticalType::Preposition
        );
        assert_eq!(
            GrammaticalType::from_str("conjunction").unwrap(),
            GrammaticalType::Conjunction
        );
        assert_eq!(
            GrammaticalType::from_str("interjection").unwrap(),
            GrammaticalType::Interjection
        );
        assert_eq!(
            GrammaticalType::from_str("article").unwrap(),
            GrammaticalType::Article
        );

        // Invalid grammatical types
        assert!(GrammaticalType::from_str("invalid").is_err());
        assert!(GrammaticalType::from_str("").is_err());
        assert!(GrammaticalType::from_str("NOUN").is_err()); // case sensitive
    }

    #[test]
    fn test_grammatical_type_type_name() {
        assert_eq!(GrammaticalType::Noun.type_name(), "noun");
        assert_eq!(GrammaticalType::Verb.type_name(), "verb");
        assert_eq!(GrammaticalType::Adjective.type_name(), "adjective");
        assert_eq!(GrammaticalType::Adverb.type_name(), "adverb");
        assert_eq!(GrammaticalType::Pronoun.type_name(), "pronoun");
        assert_eq!(GrammaticalType::Preposition.type_name(), "preposition");
        assert_eq!(GrammaticalType::Conjunction.type_name(), "conjunction");
        assert_eq!(GrammaticalType::Interjection.type_name(), "interjection");
        assert_eq!(GrammaticalType::Article.type_name(), "article");
    }

    #[test]
    fn test_grammatical_type_display() {
        assert_eq!(format!("{}", GrammaticalType::Noun), "noun");
        assert_eq!(format!("{}", GrammaticalType::Verb), "verb");
        assert_eq!(format!("{}", GrammaticalType::Adjective), "adjective");
        assert_eq!(format!("{}", GrammaticalType::Adverb), "adverb");
        assert_eq!(format!("{}", GrammaticalType::Pronoun), "pronoun");
        assert_eq!(format!("{}", GrammaticalType::Preposition), "preposition");
        assert_eq!(format!("{}", GrammaticalType::Conjunction), "conjunction");
        assert_eq!(format!("{}", GrammaticalType::Interjection), "interjection");
        assert_eq!(format!("{}", GrammaticalType::Article), "article");
    }

    #[test]
    fn test_validate_word() {
        // Valid words
        assert!(validate_word("hello").is_ok());
        assert!(validate_word("co-worker").is_ok());
        assert!(validate_word("don't").is_ok());

        // Invalid words
        assert!(validate_word("hello world").is_err()); // whitespace
        assert!(validate_word("hello@world").is_err()); // invalid character
        assert!(validate_word("").is_err()); // empty
    }

    #[test]
    fn test_validate_definition() {
        // Valid definitions
        assert!(validate_definition("a simple definition").is_ok());
        assert!(validate_definition("definition with punctuation!").is_ok());

        // Invalid definitions
        assert!(validate_definition("bad@definition").is_err());
        assert!(validate_definition("").is_err());
    }

    #[test]
    fn test_validate_pronunciation() {
        // Valid pronunciations
        assert!(validate_pronunciation("/əˈbeɪt/").is_ok());
        assert!(validate_pronunciation("/test/").is_ok());

        // Invalid pronunciations
        assert!(validate_pronunciation("invalid").is_err());
        assert!(validate_pronunciation("").is_err());
    }

    #[test]
    fn test_upsert_word_validation() {
        let valid_word = UpsertWord {
            word: "hello".to_string(),
            definition: "a greeting".to_string(),
            pronunciation: "/həˈloʊ/".to_string(),
            word_type: "noun".to_string(),
        };

        assert!(valid_word.validate().is_ok());
        assert!(valid_word.word().is_ok());
        assert!(valid_word.definition().is_ok());
        assert!(valid_word.pronunciation().is_ok());
        assert!(valid_word.word_type().is_ok());

        let invalid_word = UpsertWord {
            word: "hello world".to_string(), // invalid: contains space
            definition: "a greeting".to_string(),
            pronunciation: "/həˈloʊ/".to_string(),
            word_type: "noun".to_string(),
        };

        assert!(invalid_word.validate().is_err());
        assert!(invalid_word.word().is_err());
    }
}
