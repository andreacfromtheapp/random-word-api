//! Shared database utilities for integration tests
//!
//! Provides a shared database instance with minimal test fixtures
//! for read-only tests that don't require isolated databases.

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::sync::OnceLock;
use tempfile::NamedTempFile;
use word_api_axum::models::word::{GrammaticalType, LanguageCode};

/// Global shared database pool for read-heavy tests
static SHARED_DB: OnceLock<(Pool<Sqlite>, NamedTempFile)> = OnceLock::new();

/// Gets or creates the shared database with minimal test data
pub async fn get_shared_database() -> Result<&'static Pool<Sqlite>> {
    if let Some((pool, _)) = SHARED_DB.get() {
        return Ok(pool);
    }

    let (pool, temp_file) = create_and_populate_shared_db().await?;
    let _ = SHARED_DB.set((pool, temp_file));
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

    populate_minimal_fixtures(&pool).await?;
    Ok((pool, temp_file))
}

/// Populates the shared database with minimal test fixtures
async fn populate_minimal_fixtures(pool: &Pool<Sqlite>) -> Result<()> {
    let language = LanguageCode::English;
    let noun = GrammaticalType::Noun;
    let verb = GrammaticalType::Verb;
    let adjective = GrammaticalType::Adjective;
    let adverb = GrammaticalType::Adverb;
    let allowed_word_types = [
        noun.type_name(),
        verb.type_name(),
        adjective.type_name(),
        adverb.type_name(),
    ];

    // Create one word per type for comprehensive read testing
    for (index, &word_type) in allowed_word_types.iter().enumerate() {
        let fixture =
            super::test_data::create_typed_test_word(word_type, &format!("fixture{index}"));

        let _result =
            word_api_axum::models::word::Word::create(pool.clone(), &language.to_string(), fixture)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to insert shared fixture: {:?}", e))?;
    }

    Ok(())
}
