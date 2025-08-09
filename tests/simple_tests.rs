//! Simplified integration tests
//!
//! This module contains focused integration tests that work with the database
//! constraints and provide good coverage without conflicts.

mod common;
use common::{
    create_test_db, invalid_definition_field, invalid_pronunciation_field, invalid_word,
    invalid_word_field, invalid_word_type_field, sample_word_with_type, validate_test_word,
};
use random_word_api::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, GetWord, UpsertWord, Word,
};

#[tokio::test]
async fn test_database_setup() {
    let pool = create_test_db().await;

    // Test that migrations worked
    let result = sqlx::query("SELECT COUNT(*) as count FROM words")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_word_crud_operations() {
    let pool = create_test_db().await;

    // Create a unique word for this test using validator-verified data
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let word_data = UpsertWord {
        word: format!("testword{unique_id}"),
        definition: format!("test definition {unique_id}"),
        pronunciation: "/təst/".to_string(),
        word_type: "noun".to_string(),
    };

    // Verify test data is valid using validators
    assert!(
        validate_test_word(&word_data),
        "Test data should be valid according to validators"
    );

    // Test create
    let created = Word::create(pool.clone(), "en", word_data).await.unwrap();
    assert_eq!(created.len(), 1);
    let word_id = created[0].id();

    // Test read
    let read_result = Word::read(pool.clone(), "en", word_id).await.unwrap();
    assert_eq!(read_result.len(), 1);
    assert_eq!(read_result[0].word(), &format!("testword{unique_id}"));

    // Test update
    let update_data = UpsertWord {
        word: format!("updated{unique_id}"),
        definition: format!("updated definition {unique_id}"),
        pronunciation: "/ʌpˈdeɪtɪd/".to_string(),
        word_type: "verb".to_string(),
    };

    let updated = Word::update(pool.clone(), "en", word_id, update_data)
        .await
        .unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].word(), &format!("updated{unique_id}"));
    assert_eq!(updated[0].word_type(), "verb");

    // Test delete
    let delete_result = Word::delete(pool.clone(), "en", word_id).await;
    assert!(delete_result.is_ok());

    // Verify deletion
    let read_after_delete = Word::read(pool, "en", word_id).await.unwrap();
    assert_eq!(read_after_delete.len(), 0);
}

#[tokio::test]
async fn test_invalid_language_operations() {
    let pool = create_test_db().await;

    // Test that invalid language codes are rejected
    assert!(Word::list(pool.clone(), "invalid").await.is_err());
    assert!(Word::read(pool.clone(), "invalid", 1).await.is_err());
    assert!(Word::delete(pool.clone(), "invalid", 1).await.is_err());
    assert!(GetWord::random_word(pool.clone(), "invalid").await.is_err());
    assert!(GetWord::random_type(pool, "invalid", "noun").await.is_err());
}

#[tokio::test]
async fn test_random_word_with_existing_data() {
    let pool = create_test_db().await;

    // The database already has words from migrations
    // Test that we can get random words
    let result = GetWord::random_word(pool.clone(), "en").await;
    assert!(result.is_ok());

    let words = result.unwrap();
    if !words.is_empty() {
        // If we got a word, verify it has the required fields
        assert!(!words[0].word().is_empty());
        assert!(!words[0].definition().is_empty());
        assert!(!words[0].pronunciation().is_empty());
    }
}

#[tokio::test]
async fn test_random_word_by_type() {
    let pool = create_test_db().await;

    // Test getting random words by type
    let noun_result = GetWord::random_type(pool.clone(), "en", "noun").await;
    assert!(noun_result.is_ok());

    let verb_result = GetWord::random_type(pool.clone(), "en", "verb").await;
    assert!(verb_result.is_ok());

    let adj_result = GetWord::random_type(pool.clone(), "en", "adjective").await;
    assert!(adj_result.is_ok());

    let adv_result = GetWord::random_type(pool, "en", "adverb").await;
    assert!(adv_result.is_ok());
}

#[tokio::test]
async fn test_word_validation() {
    let pool = create_test_db().await;

    // Test that invalid words are rejected using validator-generated invalid data
    let invalid_data = invalid_word();

    // Verify test data is actually invalid using validators
    assert!(
        !validate_test_word(&invalid_data),
        "Test data should be invalid according to validators"
    );

    let result = Word::create(pool, "en", invalid_data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_direct_validator_functions() {
    // Test the validation functions directly with various inputs

    // Test valid lemma/word validation
    assert!(is_valid_lemma("test"));
    assert!(is_valid_lemma("hello-world"));
    assert!(is_valid_lemma("word123"));
    assert!(!is_valid_lemma("invalid word")); // Contains space
    assert!(!is_valid_lemma("test@invalid")); // Contains @
    assert!(!is_valid_lemma("")); // Empty string

    // Test valid definition validation
    assert!(is_valid_definition("a test definition"));
    assert!(is_valid_definition("Test with punctuation, numbers 123!"));
    assert!(is_valid_definition("Definition with apostrophe's"));
    assert!(!is_valid_definition("test@invalid.com")); // Contains @
    assert!(!is_valid_definition("")); // Empty string

    // Test valid pronunciation validation
    assert!(is_valid_pronunciation("/tɛst/"));
    assert!(is_valid_pronunciation("/həˈloʊ/"));
    assert!(is_valid_pronunciation("/ˈwɜːrd/"));
    assert!(!is_valid_pronunciation("invalid")); // Missing slashes
    assert!(!is_valid_pronunciation("/invalid@/")); // Contains @
    assert!(!is_valid_pronunciation("")); // Empty string
    assert!(!is_valid_pronunciation("/")); // Only slashes, no content
}

#[tokio::test]
async fn test_validation_edge_cases() {
    let pool = create_test_db().await;

    // Test each type of validation failure separately

    // Invalid word field only
    let invalid_word_data = invalid_word_field();
    assert!(!validate_test_word(&invalid_word_data));
    let result = Word::create(pool.clone(), "en", invalid_word_data).await;
    assert!(result.is_err(), "Should fail with invalid word field");

    // Invalid definition field only
    let invalid_def_data = invalid_definition_field();
    assert!(!validate_test_word(&invalid_def_data));
    let result = Word::create(pool.clone(), "en", invalid_def_data).await;
    assert!(result.is_err(), "Should fail with invalid definition field");

    // Invalid pronunciation field only
    let invalid_pron_data = invalid_pronunciation_field();
    assert!(!validate_test_word(&invalid_pron_data));
    let result = Word::create(pool.clone(), "en", invalid_pron_data).await;
    assert!(
        result.is_err(),
        "Should fail with invalid pronunciation field"
    );

    // Invalid word type field only
    let invalid_type_data = invalid_word_type_field();
    assert!(!validate_test_word(&invalid_type_data));
    let result = Word::create(pool.clone(), "en", invalid_type_data).await;
    assert!(result.is_err(), "Should fail with invalid word type field");
}

#[tokio::test]
async fn test_word_creation_all_types() {
    let pool = create_test_db().await;

    // Test creating words of each valid type using validator-verified data
    let word_types = ["noun", "verb", "adjective", "adverb"];

    for word_type in word_types {
        let word_data = sample_word_with_type(word_type);

        // Verify the test data is valid using validators
        assert!(
            validate_test_word(&word_data),
            "Test data for {word_type} should be valid according to validators"
        );

        let result = Word::create(pool.clone(), "en", word_data).await;
        assert!(result.is_ok(), "Should succeed with valid {word_type}");

        let created_words = result.unwrap();
        assert_eq!(created_words.len(), 1);
        assert_eq!(created_words[0].word_type(), word_type);
    }
}

#[tokio::test]
async fn test_concurrent_reads() {
    let pool = create_test_db().await;

    // Test multiple concurrent read operations
    let mut handles = vec![];

    for _ in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move { GetWord::random_word(pool_clone, "en").await });
        handles.push(handle);
    }

    // All reads should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
