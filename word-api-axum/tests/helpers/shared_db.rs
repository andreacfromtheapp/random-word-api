//! Streamlined shared database helper for Phase 3 test optimization
//!
//! This module provides a minimal, high-performance shared database that can be
//! reused across read-heavy tests to eliminate database creation overhead.
//!
//! Phase 3 optimizations:
//! - Minimal fixture set focused on essential test coverage
//! - Streamlined database initialization
//! - Reduced memory footprint
//! - Fast access patterns for performance testing

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::sync::OnceLock;
use tempfile::NamedTempFile;
use word_api_axum::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, Language, UpsertWord,
    ALLOWED_WORD_TYPES,
};

/// Global shared database pool for read-heavy tests
static SHARED_DB: OnceLock<(Pool<Sqlite>, NamedTempFile)> = OnceLock::new();

/// Gets or creates the shared database with minimal test data
pub async fn get_shared_database() -> Result<&'static Pool<Sqlite>> {
    if let Some((pool, _)) = SHARED_DB.get() {
        return Ok(pool);
    }

    // Initialize the shared database once
    let (pool, temp_file) = create_and_populate_shared_db().await?;

    // Store in static - this will only happen once
    let _ = SHARED_DB.set((pool, temp_file));

    // Return reference to the pool
    Ok(&SHARED_DB.get().unwrap().0)
}

/// Creates and populates a shared database with minimal test fixtures
async fn create_and_populate_shared_db() -> Result<(Pool<Sqlite>, NamedTempFile)> {
    let temp_file = NamedTempFile::new().context("Failed to create shared database file")?;
    let db_path = temp_file.path().to_string_lossy();
    let db_url = format!("sqlite://{db_path}");

    let pool = word_api_axum::init_dbpool(&db_url)
        .await
        .context("Failed to initialize shared database pool")?;

    // Populate with minimal test data for performance
    populate_minimal_fixtures(&pool).await?;

    Ok((pool, temp_file))
}

/// Populates the shared database with minimal test fixtures for fast testing
async fn populate_minimal_fixtures(pool: &Pool<Sqlite>) -> Result<()> {
    let language = Language::English;

    // Create one representative word per type - minimal but sufficient for read tests
    let fixtures = create_minimal_fixtures();

    for fixture in fixtures {
        // Validate each fixture using built-in validation functions
        assert!(
            is_valid_lemma(&fixture.word),
            "Fixture word '{}' should be valid",
            fixture.word
        );
        assert!(
            is_valid_definition(&fixture.definition),
            "Fixture definition should be valid"
        );
        assert!(
            is_valid_pronunciation(&fixture.pronunciation),
            "Fixture pronunciation '{}' should be valid",
            fixture.pronunciation
        );

        // Insert using Language enum's display implementation
        let _result =
            word_api_axum::models::word::Word::create(pool.clone(), &language.to_string(), fixture)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to insert shared fixture: {:?}", e))?;
    }

    Ok(())
}

/// Creates minimal test fixtures - one per word type for efficient testing
fn create_minimal_fixtures() -> Vec<UpsertWord> {
    ALLOWED_WORD_TYPES
        .iter()
        .enumerate()
        .map(|(index, &word_type)| create_fixture_by_type(word_type, index))
        .collect()
}

/// Creates a single fixture word of a specific type with proper validation
fn create_fixture_by_type(word_type: &str, index: usize) -> UpsertWord {
    let (base_word, base_definition, base_pronunciation) = match word_type {
        "noun" => ("testcat", "a test feline animal", "/tɛstkæt/"),
        "verb" => ("testrun", "to move quickly in tests", "/tɛstrʌn/"),
        "adjective" => (
            "testbeautiful",
            "pleasing in test scenarios",
            "/tɛstbjuːtɪfəl/",
        ),
        "adverb" => ("testquickly", "at test speed", "/tɛstkwɪkli/"),
        _ => ("testword", "a basic test word", "/tɛstwɜːrd/"),
    };

    // Map index to valid IPA endings for unique pronunciations
    let ipa_endings = ["ər", "əl", "əs", "əd"];
    let ipa_ending = ipa_endings[index % ipa_endings.len()];

    // Add index to ensure uniqueness across types
    UpsertWord {
        word: format!("{base_word}{index}"),
        definition: format!("{base_definition} {index}"),
        pronunciation: format!(
            "{}{}{}",
            &base_pronunciation[..base_pronunciation.len() - 1],
            ipa_ending,
            "/"
        ),
        word_type: word_type.to_string(),
    }
}
