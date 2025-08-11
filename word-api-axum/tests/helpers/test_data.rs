//! Test data utilities for integration tests
//!
//! Provides database operations and streamlined word creation functions
//! for integration tests. Uses source validation and types directly.

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use word_api_axum::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, validate_word_type, UpsertWord,
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

// === Test Word Creation ===

/// Generate unique IPA suffix for pronunciations
fn get_unique_ipa(suffix: &str) -> String {
    let mut hasher = DefaultHasher::new();
    suffix.hash(&mut hasher);
    let hash = hasher.finish();

    let ipa_chars = ["ə", "ɪ", "ʊ", "ɛ", "ɔ", "æ", "ʌ", "ɑ"];
    let index = (hash as usize) % ipa_chars.len();
    ipa_chars[index].to_string()
}

/// Creates a basic test word (noun type)
#[allow(dead_code)]
pub fn create_basic_test_word(suffix: &str) -> UpsertWord {
    let clean_suffix = suffix.replace(['_', '-', ' '], "");
    let word = format!("test{clean_suffix}");
    let definition = format!("a test word {clean_suffix}");
    let pronunciation = format!("/tɛst{}/", get_unique_ipa(suffix));

    let upsert = UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: "noun".to_string(),
    };

    // Validate using source validation functions
    assert!(
        is_valid_lemma(&upsert.word),
        "Generated word '{}' should pass source lemma validation",
        upsert.word
    );
    assert!(
        is_valid_definition(&upsert.definition),
        "Generated definition should pass source validation"
    );
    assert!(
        is_valid_pronunciation(&upsert.pronunciation),
        "Generated pronunciation '{}' should pass source validation",
        upsert.pronunciation
    );
    assert!(
        validate_word_type(&upsert.word_type).is_ok(),
        "Generated word type should pass source validation"
    );

    upsert
}

/// Creates a test word of a specific type
#[allow(dead_code)]
pub fn create_typed_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    assert!(
        ALLOWED_WORD_TYPES.contains(&word_type),
        "Word type must be one of: {ALLOWED_WORD_TYPES:?}"
    );

    let clean_suffix = suffix.replace(['_', '-', ' '], "");
    let unique_ipa = get_unique_ipa(suffix);

    let (word_base, def_pattern, pron_base) = match word_type {
        "noun" => ("cat", "a test animal", "kæt"),
        "verb" => ("run", "to move quickly", "rʌn"),
        "adjective" => ("quick", "fast in nature", "kwɪk"),
        "adverb" => ("quickly", "at fast speed", "kwɪkli"),
        _ => ("word", "a test word", "wɜːrd"),
    };

    let word = format!("{word_base}{clean_suffix}");
    let definition = format!("{def_pattern} {clean_suffix}");
    let pronunciation = format!("/{pron_base}{unique_ipa}/");

    let upsert = UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    };

    // Validate using source validation functions
    assert!(
        is_valid_lemma(&upsert.word),
        "Generated word '{}' should pass source lemma validation",
        upsert.word
    );
    assert!(
        is_valid_definition(&upsert.definition),
        "Generated definition should pass source validation"
    );
    assert!(
        is_valid_pronunciation(&upsert.pronunciation),
        "Generated pronunciation '{}' should pass source validation",
        upsert.pronunciation
    );
    assert!(
        validate_word_type(&upsert.word_type).is_ok(),
        "Generated word type '{}' should pass source validation",
        upsert.word_type
    );

    upsert
}
