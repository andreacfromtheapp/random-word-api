use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::Error;
use crate::word::{UpsertWord, Word};

pub async fn ping(State(dbpool): State<SqlitePool>) -> Result<String, Error> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "ok".to_string())
        .map_err(Into::into)
}

pub async fn word_list(State(dbpool): State<SqlitePool>) -> Result<Json<Vec<Word>>, Error> {
    Word::list(dbpool).await.map(Json::from)
}

pub async fn word_create(
    State(dbpool): State<SqlitePool>,
    Json(new_word): Json<UpsertWord>,
) -> Result<Json<Word>, Error> {
    Word::create(dbpool, new_word).await.map(Json::from)
}

pub async fn word_read(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<Json<Word>, Error> {
    Word::read(dbpool, id).await.map(Json::from)
}

pub async fn word_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
    Json(updated_word): Json<UpsertWord>,
) -> Result<Json<Word>, Error> {
    Word::update(dbpool, id, updated_word).await.map(Json::from)
}

pub async fn word_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<(), Error> {
    Word::delete(dbpool, id).await
}
