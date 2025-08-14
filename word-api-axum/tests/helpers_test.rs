//! Helper function tests
//!
//! Essential tests for helper utilities including server creation,
//! database operations, and test data functionality.

use anyhow::Result;

mod helpers;
use helpers::{
    create_test_server_with_pool,
    test_data::{count_words, create_basic_test_word, create_typed_test_word},
};

#[tokio::test]
async fn test_server_creation() -> Result<()> {
    let (_server, pool) = create_test_server_with_pool().await?;

    let count = count_words(&pool).await?;
    assert!(count >= 0, "Database should return valid word count");

    Ok(())
}

#[tokio::test]
async fn test_in_memory_database() -> Result<()> {
    let (_server, pool) = create_test_server_with_pool().await?;
    let count = count_words(&pool).await?;
    assert!(
        count > 0,
        "In-memory database should have migrated test data"
    );

    Ok(())
}

#[test]
fn test_word_creation() {
    let basic_word = create_basic_test_word("test");
    let typed_word = create_typed_test_word("noun", "test");

    assert_ne!(basic_word.word, typed_word.word);
    assert_eq!(typed_word.word_type, "noun");
}

#[test]
fn test_all_grammatical_types_supported() {
    // Test that helper functions work with all supported grammatical types
    let supported_types = [
        "noun",
        "verb",
        "adjective",
        "adverb",
        "pronoun",
        "preposition",
        "conjunction",
        "interjection",
        "article",
    ];

    for word_type in supported_types {
        let typed_word = create_typed_test_word(word_type, "test");
        assert_eq!(typed_word.word_type, word_type);
        assert!(!typed_word.word.is_empty());
        assert!(!typed_word.definition.is_empty());
        assert!(!typed_word.pronunciation.is_empty());
    }
}
