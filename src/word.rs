use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, SqlitePool};

use crate::error::Error;

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
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Self>, Error> {
        query_as("select * from words")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn random(dbpool: SqlitePool) -> Result<Self, Error> {
        query_as("select * from words order by random() limit 1")
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(dbpool: SqlitePool, new_word: UpsertWord) -> Result<Self, Error> {
        query_as("insert into words (word, definition, pronunciation) values (?, ?, ?) returning *")
            .bind(new_word.word())
            .bind(new_word.definition())
            .bind(new_word.pronunciation())
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn read(dbpool: SqlitePool, id: u32) -> Result<Self, Error> {
        query_as("select * from words where id = ?")
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
        query_as(
            "update words set word = ?, definition = ?, pronunciation = ?  where id = ? returning *",
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
        query("delete from words where id = ?")
            .bind(id)
            .execute(&dbpool)
            .await?;
        Ok(())
    }
}

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
