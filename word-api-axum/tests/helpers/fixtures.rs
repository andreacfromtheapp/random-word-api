//! Test fixture utilities for simple integration tests
//!
//! This module provides basic test data fixtures for the simple test files.
//! It focuses on essential test data without over-engineering or complex patterns.
//!
//! ## Usage Guidelines
//!
//! ### WordFactory
//! Use `WordFactory` for dynamic test data generation:
//! ```rust
//! let unique_word = WordFactory::create_with_suffix("example", "noun", "test1");
//! assert_eq!(word.word, "exampletest1");
//! ```
//!
//! ## Data Isolation
//! - Factory methods include unique suffixes to prevent conflicts
//! - All generated data uses valid IPA pronunciations

use word_api_axum::models::word::UpsertWord;

/// Create a simple test word with basic fields
#[allow(dead_code)]
pub fn create_test_word(suffix: &str) -> UpsertWord {
    UpsertWord {
        word: format!("test{suffix}"),
        definition: format!("Test definition {suffix}"),
        pronunciation: format!("/tɛst{suffix}/"),
        word_type: "noun".to_string(),
    }
}

/// Creates test words for different word types
#[allow(dead_code)]
pub fn create_test_word_by_type(word_type: &str, suffix: &str) -> UpsertWord {
    let (word, definition, pronunciation) = match word_type {
        "noun" => (
            format!("cat{suffix}"),
            format!("a small domesticated carnivorous mammal {suffix}"),
            format!("/kæt{suffix}/"),
        ),
        "verb" => (
            format!("run{suffix}"),
            format!("move at a speed faster than a walk {suffix}"),
            format!("/rʌn{suffix}/"),
        ),
        "adjective" => (
            format!("beautiful{suffix}"),
            format!("pleasing to the senses or mind aesthetically {suffix}"),
            format!("/beautiful{suffix}/"),
        ),
        "adverb" => (
            format!("quickly{suffix}"),
            format!("at a fast speed {suffix}"),
            format!("/quickly{suffix}/"),
        ),
        _ => (
            format!("word{suffix}"),
            format!("definition {suffix}"),
            format!("/word{suffix}/"),
        ),
    };

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

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
        // Generate a valid pronunciation using only allowed IPA characters
        let ipa_base = match base_word {
            "test" => "tɛst",
            "example" => "ɪɡzæmpəl",
            "bulktest" => "bʌlktɛst",
            _ => "tɛst", // fallback
        };

        // Create unique pronunciation by adding IPA variation based on suffix hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        suffix.hash(&mut hasher);
        let hash = hasher.finish();
        let ipa_chars = ["ə", "ɪ", "ʊ", "ɛ", "ɔ", "æ", "ʌ", "ɑ"];
        let index = (hash as usize) % ipa_chars.len();

        UpsertWord {
            word: format!("{base_word}{suffix}"),
            definition: format!("Test definition for {base_word}{suffix}"),
            pronunciation: format!("/{}{}/", ipa_base, ipa_chars[index]),
            word_type: word_type.to_string(),
        }
    }
}
