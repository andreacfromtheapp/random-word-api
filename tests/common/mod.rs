//! Common test utilities and fixtures
//!
//! This module provides shared testing infrastructure including database setup,
//! test data generation, and helper functions for integration tests.

use axum::Router;
use random_word_api::models::{
    apiconfig::{ApiConfig, OpenApiDocs},
    word::{is_valid_definition, is_valid_lemma, is_valid_pronunciation, UpsertWord},
};
use random_word_api::routes::create_router;
use random_word_api::state::AppState;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use std::sync::Mutex;

/// Creates a test database pool with only schema migrations applied (no data)
#[allow(dead_code)]
pub async fn create_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run only schema migrations, not data migrations
    // We manually create the schema instead of running all migrations
    // to avoid inserting the large dataset for tests
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS words (
          id INTEGER PRIMARY KEY NOT NULL,
          word_type TEXT NOT NULL CHECK (
            word_type IN ("noun", "verb", "adjective", "adverb")
          ),
          word TEXT NOT NULL UNIQUE,
          definition TEXT NOT NULL UNIQUE,
          pronunciation TEXT NOT NULL UNIQUE,
          created_at TEXT,
          updated_at TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create words table");

    // Add triggers for created_at and updated_at
    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS words_created_at
        AFTER INSERT ON words
        BEGIN
          UPDATE words SET created_at = datetime('now') WHERE id = NEW.id;
        END;
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create created_at trigger");

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS words_updated_at
        AFTER UPDATE ON words
        BEGIN
          UPDATE words SET updated_at = datetime('now') WHERE id = NEW.id;
        END;
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create updated_at trigger");

    pool
}

/// Creates a test application with in-memory database
#[allow(dead_code)]
pub async fn create_test_app() -> Router {
    let dbpool = create_test_db().await;
    let config = test_config();

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
        dbpool,
    };

    create_router(state).await
}

/// Creates a test configuration with default values
#[allow(dead_code)]
pub fn test_config() -> ApiConfig {
    ApiConfig::new(
        "127.0.0.1".parse().unwrap(),
        0, // Let the OS choose a port for tests
        "sqlite::memory:".to_string(),
        OpenApiDocs::new(false, false, false, false),
    )
}

/// Creates a valid test word for database operations
/// Uses validators from word.rs to ensure validity
#[allow(dead_code)]
pub fn sample_word() -> UpsertWord {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);

    let word = format!("test{id}");
    let definition = format!("a sample word for testing {id}");
    let pronunciation = match id {
        0 => "/tɛst/".to_string(),
        1 => "/tɛstɪ/".to_string(),
        2 => "/tɛstə/".to_string(),
        3 => "/tɛstʊ/".to_string(),
        4 => "/tɛstɔ/".to_string(),
        5 => "/tɛstɑ/".to_string(),
        _ => {
            let vowel = match id % 6 {
                0 => "ɪ",
                1 => "ʊ",
                2 => "ɔ",
                3 => "ɑ",
                4 => "æ",
                _ => "ɛ",
            };
            format!("/tɛstə{vowel}/")
        }
    };
    let word_type = "noun".to_string();

    // Validate that our generated data passes all validators
    assert!(
        is_valid_lemma(&word),
        "Generated word '{word}' failed validation"
    );
    assert!(
        is_valid_definition(&definition),
        "Generated definition '{definition}' failed validation"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation '{pronunciation}' failed validation"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type,
    }
}

/// Creates a valid test word with a specific word type
/// Uses validators to ensure validity
#[allow(dead_code)]
pub fn sample_word_with_type(word_type: &str) -> UpsertWord {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1000);
    let _id = COUNTER.fetch_add(1, Ordering::SeqCst);

    let word = format!("test{word_type}");
    let definition = format!("a sample {word_type} for testing purposes");

    // Generate valid IPA pronunciations based on word type
    let pronunciation = match word_type {
        "noun" => "/tɛstnaʊn/".to_string(),
        "verb" => "/tɛstvɜːb/".to_string(),
        "adjective" => "/tɛstæʤəktɪv/".to_string(),
        "adverb" => "/tɛstædvɜːb/".to_string(),
        _ => "/tɛst/".to_string(),
    };

    // Validate generated data
    assert!(
        is_valid_lemma(&word),
        "Generated word '{word}' failed validation"
    );
    assert!(
        is_valid_definition(&definition),
        "Generated definition '{definition}' failed validation"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation '{pronunciation}' failed validation"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

/// Populates the test database with sample words
/// Uses validators to ensure all test data is valid
#[allow(dead_code)]
pub async fn populate_test_db(pool: &SqlitePool) {
    use random_word_api::models::word::Word;

    let sample_words = vec![
        sample_word_with_type("verb"),
        sample_word_with_type("adjective"),
        sample_word_with_type("adverb"),
        sample_word_with_type("noun"),
    ];

    // Validate all test data before inserting
    for word_data in &sample_words {
        assert!(
            is_valid_lemma(&word_data.word),
            "Test word '{}' failed validation",
            word_data.word
        );
        assert!(
            is_valid_definition(&word_data.definition),
            "Test definition '{}' failed validation",
            word_data.definition
        );
        assert!(
            is_valid_pronunciation(&word_data.pronunciation),
            "Test pronunciation '{}' failed validation",
            word_data.pronunciation
        );
    }

    for word_data in sample_words {
        Word::create(pool.clone(), "en", word_data)
            .await
            .expect("Failed to insert test word");
    }
}

/// Creates an invalid word for testing validation
/// Uses validators to ensure the data is actually invalid
#[allow(dead_code)]
pub fn invalid_word() -> UpsertWord {
    let invalid_word = "invalid word"; // Contains space - should be invalid
    let invalid_definition = "test@invalid.com"; // Contains @ - should be invalid
    let invalid_pronunciation = "invalid"; // Missing slashes - should be invalid
    let invalid_word_type = "invalid"; // Not in allowed types - should be invalid

    // Verify that our invalid data actually fails validation
    assert!(
        !is_valid_lemma(invalid_word),
        "Test word '{invalid_word}' should be invalid but passed validation"
    );
    assert!(
        !is_valid_definition(invalid_definition),
        "Test definition '{invalid_definition}' should be invalid but passed validation"
    );
    assert!(
        !is_valid_pronunciation(invalid_pronunciation),
        "Test pronunciation '{invalid_pronunciation}' should be invalid but passed validation"
    );

    UpsertWord {
        word: invalid_word.to_string(),
        definition: invalid_definition.to_string(),
        pronunciation: invalid_pronunciation.to_string(),
        word_type: invalid_word_type.to_string(),
    }
}

/// Creates an invalid word that fails only word validation
#[allow(dead_code)]
pub fn invalid_word_field() -> UpsertWord {
    let mut word = sample_word();
    word.word = "invalid word".to_string(); // Contains space

    // Verify it fails word validation but passes others
    assert!(!is_valid_lemma(&word.word));
    assert!(is_valid_definition(&word.definition));
    assert!(is_valid_pronunciation(&word.pronunciation));

    word
}

/// Creates an invalid word that fails only definition validation
#[allow(dead_code)]
pub fn invalid_definition_field() -> UpsertWord {
    let mut word = sample_word();
    word.definition = "test@invalid.com".to_string(); // Contains @

    // Verify it fails definition validation but passes others
    assert!(is_valid_lemma(&word.word));
    assert!(!is_valid_definition(&word.definition));
    assert!(is_valid_pronunciation(&word.pronunciation));

    word
}

/// Creates an invalid word that fails only pronunciation validation
#[allow(dead_code)]
pub fn invalid_pronunciation_field() -> UpsertWord {
    let mut word = sample_word();
    word.pronunciation = "invalid".to_string(); // Missing slashes

    // Verify it fails pronunciation validation but passes others
    assert!(is_valid_lemma(&word.word));
    assert!(is_valid_definition(&word.definition));
    assert!(!is_valid_pronunciation(&word.pronunciation));

    word
}

/// Creates an invalid word that fails only word_type validation
#[allow(dead_code)]
pub fn invalid_word_type_field() -> UpsertWord {
    let mut word = sample_word();
    word.word_type = "invalid".to_string(); // Not in allowed types

    // Verify other fields are still valid
    assert!(is_valid_lemma(&word.word));
    assert!(is_valid_definition(&word.definition));
    assert!(is_valid_pronunciation(&word.pronunciation));

    word
}

/// Validates that a given UpsertWord passes all validation rules
#[allow(dead_code)]
pub fn validate_test_word(word: &UpsertWord) -> bool {
    is_valid_lemma(&word.word)
        && is_valid_definition(&word.definition)
        && is_valid_pronunciation(&word.pronunciation)
        && ["noun", "verb", "adjective", "adverb"].contains(&word.word_type.as_str())
}
