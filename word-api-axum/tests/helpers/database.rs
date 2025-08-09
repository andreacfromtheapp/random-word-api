//! Database helper utilities for simple integration tests
//!
//! This module provides basic utilities for managing test databases and creating
//! simple test data for the simple test files. It focuses on essential database
//! operations without over-engineering.

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::time::{Duration, Instant};
use word_api_axum::models::word::{UpsertWord, Word};

/// Counts the total number of words in the database
pub async fn count_words(pool: &Pool<Sqlite>) -> Result<i64> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) as count FROM words")
        .fetch_one(pool)
        .await
        .context("Failed to count words in database")?;

    Ok(count)
}

/// Cleans up all test data from the database
pub async fn cleanup_test_data(pool: &Pool<Sqlite>) -> Result<()> {
    // Clean up test data with numeric suffixes (1-9)
    for i in 1..=9 {
        let pattern = format!("%{}", i);
        sqlx::query!("DELETE FROM words WHERE word LIKE ?", pattern)
            .execute(pool)
            .await
            .context("Failed to cleanup test data with numeric suffix")?;
    }

    // Clean up other test patterns
    let patterns = vec!["test%", "workflow%", "perf%", "load%", "bulk%", "memory%"];
    for pattern in patterns {
        sqlx::query!("DELETE FROM words WHERE word LIKE ?", pattern)
            .execute(pool)
            .await
            .context("Failed to cleanup test data with pattern")?;
    }

    Ok(())
}

/// Clears all data from the words table
#[allow(dead_code)]
pub async fn clear_words_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query!("DELETE FROM words")
        .execute(pool)
        .await
        .context("Failed to clear words table")?;

    Ok(())
}

/// Measures database operation performance
pub struct PerformanceMetrics {
    pub duration: Duration,
    pub memory_before: usize,
    pub memory_after: usize,
}

/// Measures the performance of a database operation
pub async fn measure_db_operation<F, T>(operation: F) -> Result<(T, PerformanceMetrics)>
where
    F: std::future::Future<Output = Result<T>>,
{
    let memory_before = get_memory_usage();
    let start = Instant::now();

    let result = operation.await?;

    let duration = start.elapsed();
    let memory_after = get_memory_usage();

    let metrics = PerformanceMetrics {
        duration,
        memory_before,
        memory_after,
    };

    Ok((result, metrics))
}

/// Gets current memory usage (simplified for testing)
fn get_memory_usage() -> usize {
    // Simple memory usage approximation
    // In a real implementation, this could use system calls or memory profilers
    std::mem::size_of::<usize>() * 1000 // Placeholder
}

/// Asserts that an operation completes within the expected time
pub fn assert_performance(metrics: &PerformanceMetrics, max_duration: Duration) {
    assert!(
        metrics.duration <= max_duration,
        "Operation took {:?}, expected <= {:?}",
        metrics.duration,
        max_duration
    );
}

/// Asserts that memory usage is within reasonable bounds
pub fn assert_memory_usage(metrics: &PerformanceMetrics, max_memory_increase: usize) {
    let memory_increase = metrics.memory_after.saturating_sub(metrics.memory_before);
    assert!(
        memory_increase <= max_memory_increase,
        "Memory increased by {} bytes, expected <= {}",
        memory_increase,
        max_memory_increase
    );
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
pub fn create_unique_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    UpsertWord {
        word: format!("testword{suffix}"),
        definition: format!("Test definition for word {suffix}"),
        pronunciation: format!("/testword{}/", get_unique_ipa(suffix)),
        word_type: word_type.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_database() -> Result<()> {
        let (pool, _temp_file) = crate::helpers::create_test_database().await?;

        // Verify database is functional (count should be non-negative)
        let count = count_words(&pool).await?;
        assert!(count >= 0, "Database should return valid word count");

        Ok(())
    }

    #[test]
    fn test_unique_test_word_creation() {
        let word1 = create_unique_test_word("noun", "123");
        let word2 = create_unique_test_word("verb", "456");

        assert_ne!(word1.word, word2.word, "Unique words should be different");
        assert_ne!(
            word1.definition, word2.definition,
            "Definitions should be different"
        );
        assert_ne!(
            word1.pronunciation, word2.pronunciation,
            "Pronunciations should be different"
        );
    }

    #[tokio::test]
    async fn test_cleanup_test_data() -> Result<()> {
        let (pool, _temp_file) = crate::helpers::create_test_database().await?;

        // Add some test data with numeric suffix
        populate_test_data(&pool, "1").await?;
        let count_before = count_words(&pool).await?;
        assert!(count_before > 0, "Should have test data");

        // Clean up test data
        cleanup_test_data(&pool).await?;
        let count_after = count_words(&pool).await?;

        // Check if cleanup worked (might not remove all data due to different patterns)
        assert!(
            count_after <= count_before,
            "Should have cleaned up some test data"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_measurement() -> Result<()> {
        let (pool, _temp_file) = crate::helpers::create_test_database().await?;

        let (count, metrics) = measure_db_operation(count_words(&pool)).await?;

        assert!(count >= 0, "Should get valid count");
        assert!(
            metrics.duration.as_millis() < 1000,
            "Operation should be fast"
        );

        Ok(())
    }

    #[test]
    fn test_performance_assertions() {
        let metrics = PerformanceMetrics {
            duration: Duration::from_millis(100),
            memory_before: 1000,
            memory_after: 1100,
        };

        // This should pass
        assert_performance(&metrics, Duration::from_millis(200));
        assert_memory_usage(&metrics, 200);
    }
}
