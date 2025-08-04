// Model for Word and its methods to use with handlers
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};
use utoipa::ToSchema;
// use std::any::{Any, TypeId};

use crate::error::AppError;

/// Represents a word in the database and in API responses
#[derive(ToSchema, Serialize, Clone, sqlx::FromRow)]
pub struct Word {
    id: u32,
    word: String,
    definition: String,
    pronunciation: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl Word {
    // fn check_input(check_word: &UpsertWord) -> Result<bool, AppError> {
    //     fn is_string(s: &dyn Any) -> bool {
    //         TypeId::of::<String>() == s.type_id()
    //     }

    //     let check = is_string(&check_word.word)
    //         && is_string(&check_word.definition)
    //         && is_string(&check_word.pronunciation);

    //     match check {
    //         false => Err(AppError::BadArgument),
    //         true => Ok(true),
    //     }
    // }

    /// Return a random word from /word
    pub async fn random(dbpool: SqlitePool) -> Result<Self, AppError> {
        query_as("SELECT * FROM words ORDER BY random() LIMIT 1")
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// List all words (admin only)
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Self>, AppError> {
        query_as("SELECT * FROM words")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Add a new word to the database (admin only)
    pub async fn create(dbpool: SqlitePool, new_word: UpsertWord) -> Result<Self, AppError> {
        // Self::check_input(&new_word)?;

        query_as(
            "INSERT INTO words (word, definition, pronunciation) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(new_word.word())
        .bind(new_word.definition())
        .bind(new_word.pronunciation())
        .fetch_one(&dbpool)
        .await
        .map_err(Into::into)
    }

    /// Get a single word by id (admin only)
    pub async fn read(dbpool: SqlitePool, id: u32) -> Result<Self, AppError> {
        query_as("SELECT * FROM words WHERE id = $1")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    /// Update an existing word (admin only)
    pub async fn update(
        dbpool: SqlitePool,
        id: u32,
        updated_word: UpsertWord,
    ) -> Result<Self, AppError> {
        // Self::check_input(&updated_word)?;

        query_as(
            "UPDATE words SET word = $1, definition = $2, pronunciation = $3 WHERE id = $4 RETURNING *",
        )
        .bind(updated_word.word())
        .bind(updated_word.definition())
        .bind(updated_word.pronunciation())
        .bind(id)
        .fetch_one(&dbpool)
        .await
        .map_err(Into::into)
    }

    /// Delete a word from the database (admin only)
    pub async fn delete(dbpool: SqlitePool, id: u32) -> Result<(), AppError> {
        query("DELETE FROM words WHERE id = $1")
            .bind(id)
            .execute(&dbpool)
            .await?;
        Ok(())
    }
}

/// Represents a word with create and update
#[derive(ToSchema, Deserialize)]
pub struct UpsertWord {
    word: String,
    definition: String,
    pronunciation: String,
}

/// Accessors (getters) helpers
impl UpsertWord {
    pub fn word(&self) -> &str {
        self.word.as_ref()
    }
    pub fn definition(&self) -> &str {
        self.definition.as_ref()
    }
    pub fn pronunciation(&self) -> &str {
        self.pronunciation.as_ref()
    }
}
