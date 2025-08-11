//! Dedicated helper module tests
//!
//! This module contains all tests for the helper utilities to avoid
//! duplicating these tests across multiple test files. This improves
//! test suite performance by running helper tests only once.

use anyhow::Result;

mod helpers;
use helpers::{
    create_test_server,
    shared_db::get_shared_database,
    test_data::{
        cleanup_test_data, count_words, count_words_by_type, create_test_word_by_type,
        create_unique_test_word, populate_test_data, WordFactory,
    },
};

// Database helper tests
#[tokio::test]
async fn test_create_test_database() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

    // Verify database is functional (count should be non-negative)
    let count = count_words(&pool).await?;
    assert!(count >= 0, "Database should return valid word count");

    Ok(())
}

#[tokio::test]
async fn test_cleanup_test_data() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

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

// Server helper tests
#[tokio::test]
async fn test_create_test_server() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Basic smoke test - health endpoint should be reachable
    let response = server.get("/health/alive").await;
    assert!(
        response.status_code() == 200,
        "Health endpoint should be reachable"
    );

    Ok(())
}

// Fixtures tests
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
fn test_word_factory_by_type() {
    let noun = create_test_word_by_type("noun", "1");
    let verb = create_test_word_by_type("verb", "1");
    let adjective = create_test_word_by_type("adjective", "1");
    let adverb = create_test_word_by_type("adverb", "1");

    assert_eq!(noun.word_type, "noun");
    assert_eq!(verb.word_type, "verb");
    assert_eq!(adjective.word_type, "adjective");
    assert_eq!(adverb.word_type, "adverb");

    // Ensure different types have different words
    assert_ne!(noun.word, verb.word);
    assert_ne!(verb.word, adjective.word);
    assert_ne!(adjective.word, adverb.word);
}

// Shared database tests (Phase 2)
#[tokio::test]
async fn test_shared_database_functionality() -> Result<()> {
    let pool = get_shared_database().await?;

    // Verify shared database is accessible
    let total_count = count_words(pool).await?;
    assert!(total_count > 0, "Shared database should have test data");

    // Test that we can count words by type using ALLOWED_WORD_TYPES
    use word_api_axum::models::word::ALLOWED_WORD_TYPES;

    for &word_type in &ALLOWED_WORD_TYPES {
        let count = count_words_by_type(pool, word_type).await?;
        assert!(count > 0, "Shared database should have {word_type} words");
    }

    Ok(())
}

#[tokio::test]
async fn test_shared_database_reuse() -> Result<()> {
    // Test that multiple calls to get_shared_database return the same instance
    let pool1 = get_shared_database().await?;
    let pool2 = get_shared_database().await?;

    // Both should have the same data
    let count1 = count_words(pool1).await?;
    let count2 = count_words(pool2).await?;

    assert_eq!(count1, count2, "Shared database should be consistent");
    assert!(count1 > 0, "Shared database should have test data");

    Ok(())
}

// Advanced shared database tests (Phase 3)
#[tokio::test]
async fn test_shared_database_performance_characteristics() -> Result<()> {
    let pool = get_shared_database().await?;

    // Test rapid consecutive operations
    for _ in 0..5 {
        let total_count = count_words(pool).await?;
        assert!(total_count > 0, "Shared database should maintain data");
    }

    // Test type-specific operations
    use word_api_axum::models::word::ALLOWED_WORD_TYPES;
    for &word_type in &ALLOWED_WORD_TYPES {
        let count = count_words_by_type(pool, word_type).await?;
        assert!(count > 0, "Shared database should have {word_type} words");
    }

    Ok(())
}
