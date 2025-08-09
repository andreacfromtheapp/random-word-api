//! Database integration tests
//!
//! This module tests database operations including CRUD operations,
//! migrations, and data integrity with a real SQLite database.

mod common;
use common::{
    create_test_db, invalid_definition_field, invalid_pronunciation_field, invalid_word_field,
    invalid_word_type_field, populate_test_db, sample_word, sample_word_with_type,
    validate_test_word,
};
use random_word_api::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, GetWord, UpsertWord, Word,
};

#[tokio::test]
async fn test_database_migrations() {
    let pool = create_test_db().await;

    // Test that we can query the words table (migration worked)
    let result = sqlx::query("SELECT COUNT(*) as count FROM words")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_word_create() {
    let pool = create_test_db().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    let result = Word::create(pool.clone(), "en", word_data).await;

    assert!(result.is_ok());
    let created_words = result.unwrap();
    assert_eq!(created_words.len(), 1);

    let created_word = &created_words[0];
    assert!(created_word.word().starts_with("test"));
    assert!(created_word
        .definition()
        .starts_with("a sample word for testing"));
    assert!(created_word.pronunciation().starts_with("/tɛst"));
    assert_eq!(created_word.word_type(), "noun");
}

#[tokio::test]
async fn test_word_create_invalid_language() {
    let pool = create_test_db().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    let result = Word::create(pool, "invalid", word_data).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_word_list() {
    let pool = create_test_db().await;
    populate_test_db(&pool).await;

    let result = Word::list(pool, "en").await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 4); // We inserted 4 sample words
}

#[tokio::test]
async fn test_word_list_empty_database() {
    let pool = create_test_db().await;

    let result = Word::list(pool, "en").await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 0);
}

#[tokio::test]
async fn test_word_read_by_id() {
    let pool = create_test_db().await;
    let word_data = sample_word();

    // Create a word first
    let created = Word::create(pool.clone(), "en", word_data).await.unwrap();
    let word_id = created[0].id();

    // Read it back
    let result = Word::read(pool, "en", word_id).await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].id(), word_id);
    assert!(words[0].word().starts_with("test"));
}

#[tokio::test]
async fn test_word_read_nonexistent() {
    let pool = create_test_db().await;

    let result = Word::read(pool, "en", 99999).await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 0); // No word found
}

#[tokio::test]
async fn test_word_update() {
    let pool = create_test_db().await;
    let word_data = sample_word();

    // Verify initial test data is valid
    assert!(
        validate_test_word(&word_data),
        "Initial test data should be valid"
    );

    // Create a word first
    let created = Word::create(pool.clone(), "en", word_data).await.unwrap();
    let word_id = created[0].id();

    // Update it
    let updated_data = UpsertWord {
        word: "updated".to_string(),
        definition: "an updated definition".to_string(),
        pronunciation: "/ʌpˈdeɪtɪd/".to_string(),
        word_type: "verb".to_string(),
    };

    // Verify update data is valid using validators
    assert!(
        validate_test_word(&updated_data),
        "Update test data should be valid"
    );

    let result = Word::update(pool.clone(), "en", word_id, updated_data).await;

    assert!(result.is_ok());
    let updated_words = result.unwrap();
    assert_eq!(updated_words.len(), 1);
    assert_eq!(updated_words[0].word(), "updated");
    assert_eq!(updated_words[0].definition(), "an updated definition");
    assert_eq!(updated_words[0].word_type(), "verb");
}

#[tokio::test]
async fn test_word_delete() {
    let pool = create_test_db().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    // Create a word first
    let created = Word::create(pool.clone(), "en", word_data).await.unwrap();
    let word_id = created[0].id();

    // Delete it
    let result = Word::delete(pool.clone(), "en", word_id).await;

    assert!(result.is_ok());

    // Verify it's gone
    let read_result = Word::read(pool, "en", word_id).await.unwrap();
    assert_eq!(read_result.len(), 0);
}

#[tokio::test]
async fn test_get_word_random() {
    let pool = create_test_db().await;
    populate_test_db(&pool).await;

    let result = GetWord::random_word(pool, "en").await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 1);

    // Should be one of our sample words - verify it contains "test" which all our test words have
    let word = &words[0];
    let word_text = word.word().to_string();
    assert!(
        word_text.starts_with("test"),
        "Random word should be one of our test words: {word_text}"
    );

    // Verify it's a valid word using our validators
    use random_word_api::models::word::{
        is_valid_definition, is_valid_lemma, is_valid_pronunciation,
    };
    assert!(is_valid_lemma(word.word()));
    assert!(is_valid_definition(word.definition()));
    assert!(is_valid_pronunciation(word.pronunciation()));
}

#[tokio::test]
async fn test_get_word_random_empty_database() {
    let pool = create_test_db().await;

    let result = GetWord::random_word(pool, "en").await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 0); // No words in empty database
}

#[tokio::test]
async fn test_get_word_random_type() {
    let pool = create_test_db().await;
    populate_test_db(&pool).await;

    // Test getting words by type - verify we get the expected types back
    let word_types = ["verb", "noun", "adjective", "adverb"];

    for word_type in word_types {
        let result = GetWord::random_type(pool.clone(), "en", word_type).await;
        assert!(result.is_ok());

        let words = result.unwrap();
        if !words.is_empty() {
            // If we got a word, it should be of the requested type
            assert_eq!(words.len(), 1);
            // We can't predict the exact word, but we know it should contain the type in the name
            assert!(words[0].word().contains(&format!("test{word_type}")));
        }
    }
}

#[tokio::test]
async fn test_get_word_random_type_no_matches() {
    let pool = create_test_db().await;
    // Only add one word of type "noun"
    let word_data = sample_word(); // This is a noun
    Word::create(pool.clone(), "en", word_data).await.unwrap();

    // Try to get a preposition (which doesn't exist in our data)
    let result = GetWord::random_type(pool, "en", "preposition").await;

    assert!(result.is_ok());
    let words = result.unwrap();
    assert_eq!(words.len(), 0); // No prepositions in database
}

#[tokio::test]
async fn test_word_operations_invalid_language() {
    let pool = create_test_db().await;

    // Test all operations with invalid language
    assert!(Word::list(pool.clone(), "invalid").await.is_err());
    assert!(Word::read(pool.clone(), "invalid", 1).await.is_err());
    assert!(Word::delete(pool.clone(), "invalid", 1).await.is_err());
    assert!(GetWord::random_word(pool.clone(), "invalid").await.is_err());
    assert!(GetWord::random_type(pool, "invalid", "noun").await.is_err());
}

#[tokio::test]
async fn test_concurrent_database_access() {
    let pool = create_test_db().await;

    // Create multiple concurrent database operations
    let mut handles = vec![];

    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let word_data = UpsertWord {
                word: format!("concurrent-word-{i}"),
                definition: format!("concurrent definition {i}"),
                pronunciation: match i {
                    0 => "/kənˈkʌrənt/".to_string(),
                    1 => "/kənˈkʌrɪnt/".to_string(),
                    2 => "/kənˈkʌrʊnt/".to_string(),
                    3 => "/kənˈkʌrɔnt/".to_string(),
                    4 => "/kənˈkʌrɑnt/".to_string(),
                    5 => "/kənˈkʌrænt/".to_string(),
                    6 => "/kənˈkʌrɛnt/".to_string(),
                    7 => "/kənˈkʌrəntɪ/".to_string(),
                    8 => "/kənˈkʌrəntʊ/".to_string(),
                    _ => {
                        let vowel = match i % 5 {
                            0 => "ɪ",
                            1 => "ʊ",
                            2 => "ɔ",
                            3 => "ɑ",
                            _ => "æ",
                        };
                        format!("/kənˈkʌrəntˈɛkstrə{vowel}ə/")
                    }
                },
                word_type: "noun".to_string(),
            };

            // Validate test data before using it
            assert!(
                validate_test_word(&word_data),
                "Concurrent test data should be valid"
            );

            Word::create(pool_clone, "en", word_data).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all words were created
    let all_words = Word::list(pool, "en").await.unwrap();
    assert_eq!(all_words.len(), 10);
}

#[tokio::test]
async fn test_word_validation_functions_directly() {
    // Test the validation functions directly

    // Test valid lemma/word validation
    assert!(is_valid_lemma("test"));
    assert!(is_valid_lemma("hello-world"));
    assert!(is_valid_lemma("test123"));
    assert!(!is_valid_lemma("invalid word")); // Contains space
    assert!(!is_valid_lemma("test@invalid")); // Contains @
    assert!(!is_valid_lemma("")); // Empty string

    // Test valid definition validation
    assert!(is_valid_definition("a test definition"));
    assert!(is_valid_definition("Test with punctuation, numbers 123!"));
    assert!(!is_valid_definition("test@invalid.com")); // Contains @
    assert!(!is_valid_definition("")); // Empty string

    // Test valid pronunciation validation
    assert!(is_valid_pronunciation("/tɛst/"));
    assert!(is_valid_pronunciation("/həˈloʊ/"));
    assert!(!is_valid_pronunciation("invalid")); // Missing slashes
    assert!(!is_valid_pronunciation("/invalid@/")); // Contains @
    assert!(!is_valid_pronunciation("")); // Empty string
}

#[tokio::test]
async fn test_word_create_with_specific_invalid_fields() {
    let pool = create_test_db().await;

    // Test invalid word field
    let invalid_word_data = invalid_word_field();
    let result = Word::create(pool.clone(), "en", invalid_word_data).await;
    assert!(result.is_err(), "Should fail with invalid word field");

    // Test invalid definition field
    let invalid_def_data = invalid_definition_field();
    let result = Word::create(pool.clone(), "en", invalid_def_data).await;
    assert!(result.is_err(), "Should fail with invalid definition field");

    // Test invalid pronunciation field
    let invalid_pron_data = invalid_pronunciation_field();
    let result = Word::create(pool.clone(), "en", invalid_pron_data).await;
    assert!(
        result.is_err(),
        "Should fail with invalid pronunciation field"
    );

    // Test invalid word type field
    let invalid_type_data = invalid_word_type_field();
    let result = Word::create(pool.clone(), "en", invalid_type_data).await;
    assert!(result.is_err(), "Should fail with invalid word type field");
}

#[tokio::test]
async fn test_word_create_with_different_types() {
    let pool = create_test_db().await;

    // Test creating words of each valid type using validator-verified data
    let word_types = ["noun", "verb", "adjective", "adverb"];

    for word_type in word_types {
        let word_data = sample_word_with_type(word_type);

        // Verify the test data is valid
        assert!(
            validate_test_word(&word_data),
            "Test data for {word_type} should be valid"
        );

        let result = Word::create(pool.clone(), "en", word_data).await;
        assert!(result.is_ok(), "Should succeed with valid {word_type}");

        let created_words = result.unwrap();
        assert_eq!(created_words.len(), 1);
        assert_eq!(created_words[0].word_type(), word_type);
    }
}
