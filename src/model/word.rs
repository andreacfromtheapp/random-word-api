use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};
// use std::any::{Any, TypeId};

use crate::error::Error;

/// Represents a word
#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Word {
    id: u32,
    word: String,
    definition: String,
    pronunciation: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl Word {
    // fn check_input(check_word: &UpsertWord) -> Result<bool, Error> {
    //     fn is_string(s: &dyn Any) -> bool {
    //         TypeId::of::<String>() == s.type_id()
    //     }

    //     let check = is_string(&check_word.word)
    //         && is_string(&check_word.definition)
    //         && is_string(&check_word.pronunciation);

    //     match check {
    //         false => Err(Error::BadArgument),
    //         true => Ok(true),
    //     }
    // }

    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Self>, Error> {
        query_as("SELECT * FROM words")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn random(dbpool: SqlitePool) -> Result<Self, Error> {
        query_as("SELECT * FROM words ORDER BY random() LIMIT 1")
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(dbpool: SqlitePool, new_word: UpsertWord) -> Result<Self, Error> {
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

    pub async fn read(dbpool: SqlitePool, id: u32) -> Result<Self, Error> {
        query_as("SELECT * FROM words WHERE id = $1")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        dbpool: SqlitePool,
        id: u32,
        updated_word: UpsertWord,
    ) -> Result<Self, Error> {
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

    pub async fn delete(dbpool: SqlitePool, id: u32) -> Result<(), Error> {
        query("DELETE FROM words WHERE id = $1")
            .bind(id)
            .execute(&dbpool)
            .await?;
        Ok(())
    }
}

/// Represents a word with create and update
#[derive(Deserialize)]
pub struct UpsertWord {
    word: String,
    definition: String,
    pronunciation: String,
}

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
