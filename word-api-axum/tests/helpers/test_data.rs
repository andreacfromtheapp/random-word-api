//! Consolidated test data utilities for integration tests
//!
//! This module provides comprehensive utilities for managing test databases and creating
//! test data for integration tests. It combines database operations, test data generation,
//! and validation functions into a single cohesive interface.
//!
//! Consolidated from database.rs and fixtures.rs to eliminate redundant word creation
//! logic while maintaining all essential functionality.

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use word_api_axum::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, Language, UpsertWord, Word,
    ALLOWED_WORD_TYPES,
};

// === Database Operations ===

/// Counts the total number of words in the database
#[allow(dead_code)]
pub async fn count_words(pool: &Pool<Sqlite>) -> Result<i64> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) as count FROM words")
        .fetch_one(pool)
        .await
        .context("Failed to count words in database")?;

    Ok(count)
}

/// Counts words by type in the database
#[allow(dead_code)]
pub async fn count_words_by_type(pool: &Pool<Sqlite>, word_type: &str) -> Result<i64> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM words WHERE word_type = ?",
        word_type
    )
    .fetch_one(pool)
    .await
    .context("Failed to count words by type")?;

    Ok(count)
}

/// Cleans up all test data from the database
#[allow(dead_code)]
pub async fn cleanup_test_data(pool: &Pool<Sqlite>) -> Result<()> {
    // Clean up test data with numeric suffixes (1-9)
    for i in 1..=9 {
        let pattern = format!("%{i}");
        sqlx::query!("DELETE FROM words WHERE word LIKE ?", pattern)
            .execute(pool)
            .await
            .context("Failed to cleanup test data with numeric suffix")?;
    }

    // Clean up other test patterns
    let patterns = vec![
        "test%",
        "workflow%",
        "perf%",
        "load%",
        "bulk%",
        "memory%",
        "multi%",
    ];
    for pattern in patterns {
        sqlx::query!("DELETE FROM words WHERE word LIKE ?", pattern)
            .execute(pool)
            .await
            .context("Failed to cleanup test data with pattern")?;
    }

    Ok(())
}

// === Test Data Generation ===

/// Convert suffix to valid IPA characters for unique pronunciations
fn get_unique_ipa(suffix: &str) -> String {
    let mut hasher = DefaultHasher::new();
    suffix.hash(&mut hasher);
    let hash = hasher.finish();

    // Use hash to select from valid IPA characters, ensuring uniqueness
    let ipa_chars = ["ə", "ɪ", "ʊ", "ɛ", "ɔ", "æ", "ʌ", "ɑ", "ɒ"];
    let index = (hash as usize) % ipa_chars.len();

    ipa_chars[index].to_string()
}

/// Creates a test word with unique values to avoid conflicts using built-in validation
#[allow(dead_code)]
pub fn create_unique_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    // Validate word_type using ALLOWED_WORD_TYPES
    assert!(
        ALLOWED_WORD_TYPES.contains(&word_type),
        "Word type '{word_type}' must be one of: {ALLOWED_WORD_TYPES:?}"
    );

    let word = format!("testword{suffix}");
    let definition = format!("Test definition for word {suffix}");
    let pronunciation = format!("/tɛstwɜːrd{}/", get_unique_ipa(suffix));

    // Validate using built-in functions
    assert!(is_valid_lemma(&word), "Generated word should be valid");
    assert!(
        is_valid_definition(&definition),
        "Generated definition should be valid"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation should be valid"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

/// Create a simple test word with basic fields
#[allow(dead_code)]
pub fn create_test_word(suffix: &str) -> UpsertWord {
    let word = format!("test{suffix}");
    let definition = format!("Test definition {suffix}");
    let pronunciation = "/tɛst/".to_string();
    let word_type = ALLOWED_WORD_TYPES[0]; // "noun"

    // Validate using built-in functions
    assert!(is_valid_lemma(&word), "Generated word should be valid");
    assert!(
        is_valid_definition(&definition),
        "Generated definition should be valid"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation should be valid"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

/// Creates test words for different word types
#[allow(dead_code)]
pub fn create_test_word_by_type(word_type: &str, suffix: &str) -> UpsertWord {
    // Validate word_type using ALLOWED_WORD_TYPES
    assert!(
        ALLOWED_WORD_TYPES.contains(&word_type),
        "Word type '{word_type}' must be one of: {ALLOWED_WORD_TYPES:?}"
    );

    let (word, definition, pronunciation) = match word_type {
        "noun" => (
            format!("cat{suffix}"),
            format!("a small domesticated carnivorous mammal {suffix}"),
            "/kæt/".to_string(),
        ),
        "verb" => (
            format!("run{suffix}"),
            format!("move at a speed faster than a walk {suffix}"),
            "/rʌn/".to_string(),
        ),
        "adjective" => (
            format!("beautiful{suffix}"),
            format!("pleasing to the senses or mind aesthetically {suffix}"),
            "/bjuːtɪfəl/".to_string(),
        ),
        "adverb" => (
            format!("quickly{suffix}"),
            format!("at a fast speed {suffix}"),
            "/kwɪkli/".to_string(),
        ),
        _ => (
            format!("word{suffix}"),
            format!("definition {suffix}"),
            "/wɜːrd/".to_string(),
        ),
    };

    // Validate using built-in validation functions
    assert!(is_valid_lemma(&word), "Generated word should be valid");
    assert!(
        is_valid_definition(&definition),
        "Generated definition should be valid"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation should be valid"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

/// Standardized test data population function for consistent test data across all tests
#[allow(dead_code)]
pub async fn populate_test_data(pool: &Pool<Sqlite>, suffix: &str) -> Result<()> {
    let language = Language::English;
    let test_words = vec![
        UpsertWord {
            word: format!("cat{suffix}"),
            definition: format!("a small domesticated carnivorous mammal {suffix}"),
            pronunciation: format!("/kæt{}/", get_unique_ipa(suffix)),
            word_type: ALLOWED_WORD_TYPES[0].to_string(), // "noun"
        },
        UpsertWord {
            word: format!("run{suffix}"),
            definition: format!("move at a speed faster than a walk {suffix}"),
            pronunciation: format!("/rʌn{}/", get_unique_ipa(suffix)),
            word_type: ALLOWED_WORD_TYPES[1].to_string(), // "verb"
        },
        UpsertWord {
            word: format!("beautiful{suffix}"),
            definition: format!("pleasing to the senses or mind aesthetically {suffix}"),
            pronunciation: format!("/bjuːtɪfəl{}/", get_unique_ipa(suffix)),
            word_type: ALLOWED_WORD_TYPES[2].to_string(), // "adjective"
        },
        UpsertWord {
            word: format!("quickly{suffix}"),
            definition: format!("at a fast speed {suffix}"),
            pronunciation: format!("/kwɪkli{}/", get_unique_ipa(suffix)),
            word_type: ALLOWED_WORD_TYPES[3].to_string(), // "adverb"
        },
    ];

    for word_data in test_words {
        // Validate each word using built-in validation functions
        assert!(
            is_valid_lemma(&word_data.word),
            "Generated word '{}' should be valid",
            word_data.word
        );
        assert!(
            is_valid_definition(&word_data.definition),
            "Generated definition should be valid"
        );
        assert!(
            is_valid_pronunciation(&word_data.pronunciation),
            "Generated pronunciation '{}' should be valid",
            word_data.pronunciation
        );

        let _result = Word::create(pool.clone(), &language.to_string(), word_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert test word: {:?}", e))?;
    }

    Ok(())
}

// === WordFactory - Dynamic Test Data Generation ===

/// Simple factory for generating test data with variations
///
/// Provides methods for creating dynamic test data that avoids conflicts
/// with fixture data and other test instances. All factory methods include
/// mechanisms for uniqueness and validation testing.
#[allow(dead_code)]
pub struct WordFactory;

impl WordFactory {
    /// Create a word with a unique suffix to avoid conflicts
    ///
    /// # Parameters
    /// - `base_word`: The root word to use (e.g., "test")
    /// - `word_type`: The grammatical type ("noun", "verb", "adjective", "adverb")
    /// - `suffix`: Unique identifier to prevent conflicts (e.g., test function name)
    ///
    /// # Example
    /// ```rust
    /// let word = WordFactory::create_with_suffix("example", "noun", "test1");
    /// assert_eq!(word.word, "exampletest1");
    /// ```
    #[allow(dead_code)]
    pub fn create_with_suffix(base_word: &str, word_type: &str, suffix: &str) -> UpsertWord {
        // Validate word_type using ALLOWED_WORD_TYPES
        assert!(
            ALLOWED_WORD_TYPES.contains(&word_type),
            "Word type '{word_type}' must be one of: {ALLOWED_WORD_TYPES:?}"
        );

        // Generate a valid pronunciation using only allowed IPA characters
        let ipa_base = match base_word {
            "test" => "tɛst",
            "example" => "ɪɡzæmpəl",
            "bulktest" => "bʌlktɛst",
            _ => "tɛst", // fallback
        };

        // Create unique pronunciation by adding IPA variation based on suffix hash
        let mut hasher = DefaultHasher::new();
        suffix.hash(&mut hasher);
        let hash = hasher.finish();
        let ipa_chars = ["ə", "ɪ", "ʊ", "ɛ", "ɔ", "æ", "ʌ", "ɑ"];
        let index = (hash as usize) % ipa_chars.len();

        let word = format!("{base_word}{suffix}");
        let definition = format!("Test definition for {base_word}{suffix}");
        let pronunciation = format!("/{}{}/", ipa_base, ipa_chars[index]);

        // Validate using built-in validation functions
        assert!(is_valid_lemma(&word), "Generated word should be valid");
        assert!(
            is_valid_definition(&definition),
            "Generated definition should be valid"
        );
        assert!(
            is_valid_pronunciation(&pronunciation),
            "Generated pronunciation should be valid"
        );

        UpsertWord {
            word,
            definition,
            pronunciation,
            word_type: word_type.to_string(),
        }
    }

    /// Create a basic test word for simple scenarios
    #[allow(dead_code)]
    pub fn create_basic(suffix: &str) -> UpsertWord {
        create_test_word(suffix)
    }

    /// Create a test word of a specific type
    #[allow(dead_code)]
    pub fn create_by_type(word_type: &str, suffix: &str) -> UpsertWord {
        create_test_word_by_type(word_type, suffix)
    }

    /// Create a unique test word with advanced validation
    #[allow(dead_code)]
    pub fn create_unique(word_type: &str, suffix: &str) -> UpsertWord {
        create_unique_test_word(word_type, suffix)
    }
}
