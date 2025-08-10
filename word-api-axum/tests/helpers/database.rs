//! Database helper utilities for simple integration tests
//!
//! This module provides basic utilities for managing test databases and creating
//! simple test data for the simple test files. It focuses on essential database
//! operations without over-engineering.

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use word_api_axum::models::word::{UpsertWord, Word};

/// Counts the total number of words in the database
#[allow(dead_code)]
pub async fn count_words(pool: &Pool<Sqlite>) -> Result<i64> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) as count FROM words")
        .fetch_one(pool)
        .await
        .context("Failed to count words in database")?;

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

/// Convert suffix to valid IPA characters for unique pronunciations
#[allow(dead_code)]
fn get_unique_ipa(suffix: &str) -> String {
    // Hash the suffix to get a consistent, unique identifier
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    suffix.hash(&mut hasher);
    let hash = hasher.finish();

    // Use hash to select from valid IPA characters, ensuring uniqueness
    // Only use pure IPA characters without numbers or invalid symbols
    let ipa_chars = ["ə", "ɪ", "ʊ", "ɛ", "ɔ", "æ", "ʌ", "ɑ", "ɒ"];
    let index = (hash as usize) % ipa_chars.len();

    // Use only the IPA character for uniqueness
    ipa_chars[index].to_string()
}

/// Standardized test data population function for consistent test data across all tests
#[allow(dead_code)]
pub async fn populate_test_data(pool: &Pool<Sqlite>, suffix: &str) -> Result<()> {
    let test_words = vec![
        UpsertWord {
            word: format!("cat{suffix}"),
            definition: format!("a small domesticated carnivorous mammal {suffix}"),
            pronunciation: format!("/kæt{}/", get_unique_ipa(suffix)),
            word_type: "noun".to_string(),
        },
        UpsertWord {
            word: format!("run{suffix}"),
            definition: format!("move at a speed faster than a walk {suffix}"),
            pronunciation: format!("/rʌn{}/", get_unique_ipa(suffix)),
            word_type: "verb".to_string(),
        },
        UpsertWord {
            word: format!("beautiful{suffix}"),
            definition: format!("pleasing to the senses or mind aesthetically {suffix}"),
            pronunciation: format!("/beautiful{}/", get_unique_ipa(suffix)),
            word_type: "adjective".to_string(),
        },
        UpsertWord {
            word: format!("quickly{suffix}"),
            definition: format!("at a fast speed {suffix}"),
            pronunciation: format!("/quickly{}/", get_unique_ipa(suffix)),
            word_type: "adverb".to_string(),
        },
    ];

    for word_data in test_words {
        let _result = Word::create(pool.clone(), "en", word_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert test word: {:?}", e))?;
    }

    Ok(())
}

/// Creates a test word with unique values to avoid conflicts
#[allow(dead_code)]
pub fn create_unique_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    UpsertWord {
        word: format!("testword{suffix}"),
        definition: format!("Test definition for word {suffix}"),
        pronunciation: format!("/testword{}/", get_unique_ipa(suffix)),
        word_type: word_type.to_string(),
    }
}
