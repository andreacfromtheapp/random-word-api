//! Test fixture utilities for simple integration tests
//!
//! This module provides basic test data fixtures for the simple test files.
//! It focuses on essential test data without over-engineering or complex patterns.
//!
//! ## Usage Guidelines
//!
//! ### WordFixtures
//! Use `WordFixtures` for consistent, predefined test data:
//! ```rust
//! let nouns = WordFixtures::nouns();
//! let all_words = WordFixtures::all();
//! ```
//!
//! ### WordFactory
//! Use `WordFactory` for dynamic test data generation:
//! ```rust
//! let unique_word = WordFactory::create_with_suffix("test", "noun", "123");
//! let invalid_word = WordFactory::create_invalid("word");
//! ```
//!
//! ### JsonFixtures
//! Use `JsonFixtures` for API request testing:
//! ```rust
//! let valid_request = JsonFixtures::valid_word_request();
//! let incomplete_request = JsonFixtures::incomplete_word_request();
//! ```
//!
//! ## Data Isolation
//! - All fixtures use predictable, non-conflicting data
//! - Factory methods include unique suffixes to prevent conflicts
//! - Invalid data fixtures target specific validation scenarios

use serde_json::{json, Value};

use word_api_axum::models::word::UpsertWord;

/// Basic test word fixtures organized by type
///
/// Provides collections of realistic test words organized by grammatical type.
/// All fixtures use consistent, validated data that matches the API's expectations.
pub struct WordFixtures;

impl WordFixtures {
    /// Get a collection of test nouns
    ///
    /// Returns 3 realistic noun fixtures with valid pronunciations.
    /// Safe for use in any test - no conflicts with other fixture data.
    pub fn nouns() -> Vec<UpsertWord> {
        vec![
            UpsertWord {
                word: "cat".to_string(),
                definition: "A small domesticated carnivorous mammal".to_string(),
                pronunciation: "/kæt/".to_string(),
                word_type: "noun".to_string(),
            },
            UpsertWord {
                word: "house".to_string(),
                definition: "A building for human habitation".to_string(),
                pronunciation: "/haʊs/".to_string(),
                word_type: "noun".to_string(),
            },
            UpsertWord {
                word: "book".to_string(),
                definition: "A written or printed work".to_string(),
                pronunciation: "/bʊk/".to_string(),
                word_type: "noun".to_string(),
            },
        ]
    }

    /// Get a collection of test verbs
    ///
    /// Returns 3 common verb fixtures with appropriate definitions.
    /// All verbs are in base form (infinitive without 'to').
    pub fn verbs() -> Vec<UpsertWord> {
        vec![
            UpsertWord {
                word: "run".to_string(),
                definition: "Move at a speed faster than a walk".to_string(),
                pronunciation: "/rʌn/".to_string(),
                word_type: "verb".to_string(),
            },
            UpsertWord {
                word: "eat".to_string(),
                definition: "Put food into the mouth and chew".to_string(),
                pronunciation: "/iːt/".to_string(),
                word_type: "verb".to_string(),
            },
            UpsertWord {
                word: "sleep".to_string(),
                definition: "Rest with eyes closed and inactive".to_string(),
                pronunciation: "/sliːp/".to_string(),
                word_type: "verb".to_string(),
            },
        ]
    }

    /// Get a collection of test adjectives
    ///
    /// Returns 3 descriptive adjective fixtures covering different concepts.
    /// Useful for testing word type filtering and validation.
    pub fn adjectives() -> Vec<UpsertWord> {
        vec![
            UpsertWord {
                word: "big".to_string(),
                definition: "Of considerable size or extent".to_string(),
                pronunciation: "/bɪɡ/".to_string(),
                word_type: "adjective".to_string(),
            },
            UpsertWord {
                word: "happy".to_string(),
                definition: "Feeling or showing pleasure".to_string(),
                pronunciation: "/ˈhæpi/".to_string(),
                word_type: "adjective".to_string(),
            },
            UpsertWord {
                word: "fast".to_string(),
                definition: "Moving or capable of moving at high speed".to_string(),
                pronunciation: "/fæst/".to_string(),
                word_type: "adjective".to_string(),
            },
        ]
    }

    /// Get a collection of test adverbs
    ///
    /// Returns 3 adverb fixtures with typical '-ly' endings.
    /// Tests adverb recognition and pronunciation handling.
    pub fn adverbs() -> Vec<UpsertWord> {
        vec![
            UpsertWord {
                word: "quickly".to_string(),
                definition: "At a fast speed".to_string(),
                pronunciation: "/ˈkwɪkli/".to_string(),
                word_type: "adverb".to_string(),
            },
            UpsertWord {
                word: "slowly".to_string(),
                definition: "At a slow speed".to_string(),
                pronunciation: "/ˈsloʊli/".to_string(),
                word_type: "adverb".to_string(),
            },
            UpsertWord {
                word: "carefully".to_string(),
                definition: "In a way that avoids danger".to_string(),
                pronunciation: "/ˈkɛrfəli/".to_string(),
                word_type: "adverb".to_string(),
            },
        ]
    }

    /// Get all fixture words as a single collection
    ///
    /// Returns all 12 fixture words (3 of each type) in a single vector.
    /// Useful for bulk test data setup and comprehensive API testing.
    pub fn all() -> Vec<UpsertWord> {
        let mut all_words = Vec::new();
        all_words.extend(Self::nouns());
        all_words.extend(Self::verbs());
        all_words.extend(Self::adjectives());
        all_words.extend(Self::adverbs());
        all_words
    }
}

/// Simple factory for generating test data with variations
///
/// Provides methods for creating dynamic test data that avoids conflicts
/// with fixture data and other test instances. All factory methods include
/// mechanisms for uniqueness and validation testing.
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

    /// Create a word with invalid data for validation testing
    ///
    /// # Parameters
    /// - `field`: Which field to make invalid ("word", "definition", "pronunciation", "word_type")
    ///
    /// # Returns
    /// A word with the specified field invalid (empty or incorrect value),
    /// while other fields remain valid for isolated validation testing.
    ///
    /// # Example
    /// ```rust
    /// let invalid_word = WordFactory::create_invalid("word");
    /// assert!(invalid_word.word.is_empty());
    /// ```
    pub fn create_invalid(field: &str) -> UpsertWord {
        match field {
            "word" => UpsertWord {
                word: "".to_string(), // Empty word
                definition: "Valid definition".to_string(),
                pronunciation: "/valid/".to_string(),
                word_type: "noun".to_string(),
            },
            "definition" => UpsertWord {
                word: "validword".to_string(),
                definition: "".to_string(), // Empty definition
                pronunciation: "/valid/".to_string(),
                word_type: "noun".to_string(),
            },
            "pronunciation" => UpsertWord {
                word: "validword".to_string(),
                definition: "Valid definition".to_string(),
                pronunciation: "".to_string(), // Empty pronunciation
                word_type: "noun".to_string(),
            },
            "word_type" => UpsertWord {
                word: "validword".to_string(),
                definition: "Valid definition".to_string(),
                pronunciation: "/valid/".to_string(),
                word_type: "invalid_type".to_string(), // Invalid word type
            },
            _ => UpsertWord {
                word: "validword".to_string(),
                definition: "Valid definition".to_string(),
                pronunciation: "/valid/".to_string(),
                word_type: "noun".to_string(),
            },
        }
    }
}

/// Simple JSON fixtures for API request bodies
///
/// Provides pre-built JSON values for testing API endpoints that accept
/// JSON payloads. Includes both valid and invalid request scenarios.
pub struct JsonFixtures;

impl JsonFixtures {
    /// Valid word creation request body
    ///
    /// Returns a complete, valid JSON object for word creation requests.
    /// All fields are properly formatted and pass validation.
    pub fn valid_word_request() -> Value {
        json!({
            "word": "testword",
            "definition": "A word used for testing",
            "pronunciation": "/testword/",
            "word_type": "noun"
        })
    }

    /// Word creation request with missing fields
    ///
    /// Returns a JSON object missing required fields (pronunciation and word_type).
    /// Useful for testing API validation and error handling.
    pub fn incomplete_word_request() -> Value {
        json!({
            "word": "incomplete",
            "definition": "Missing pronunciation and type"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_fixtures_basic() {
        let nouns = WordFixtures::nouns();
        let verbs = WordFixtures::verbs();
        let adjectives = WordFixtures::adjectives();
        let adverbs = WordFixtures::adverbs();

        assert!(!nouns.is_empty(), "Should have noun fixtures");
        assert!(!verbs.is_empty(), "Should have verb fixtures");
        assert!(!adjectives.is_empty(), "Should have adjective fixtures");
        assert!(!adverbs.is_empty(), "Should have adverb fixtures");

        // Verify all words have required fields
        let all_fixture_words = WordFixtures::all();
        for word in &all_fixture_words {
            assert!(!word.word.is_empty(), "Word should not be empty");
            assert!(
                !word.definition.is_empty(),
                "Definition should not be empty"
            );
            assert!(
                !word.pronunciation.is_empty(),
                "Pronunciation should not be empty"
            );
            assert!(!word.word_type.is_empty(), "Word type should not be empty");
        }
    }

    #[test]
    fn test_word_factory_basic() {
        let word = WordFactory::create_with_suffix("test", "noun", "123");
        assert_eq!(word.word, "test123");
        assert!(word.definition.contains("test123"));
        assert!(word.pronunciation.starts_with("/tɛst"));
        assert!(word.pronunciation.ends_with("/"));
        assert_eq!(word.word_type, "noun");
    }

    #[test]
    fn test_invalid_word_factory() {
        let invalid_word = WordFactory::create_invalid("word");
        assert!(invalid_word.word.is_empty());

        let invalid_definition = WordFactory::create_invalid("definition");
        assert!(invalid_definition.definition.is_empty());
    }

    #[test]
    fn test_json_fixtures_basic() {
        let valid_request = JsonFixtures::valid_word_request();
        assert!(valid_request.get("word").is_some());
        assert!(valid_request.get("definition").is_some());
        assert!(valid_request.get("pronunciation").is_some());
        assert!(valid_request.get("word_type").is_some());

        let incomplete_request = JsonFixtures::incomplete_word_request();
        assert!(incomplete_request.get("word").is_some());
        assert!(incomplete_request.get("pronunciation").is_none());
    }
}
