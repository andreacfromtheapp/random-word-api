// API handlers for word management
use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::word::{UpsertWord, Word};

/// List all words (admin only)
pub async fn word_list(State(dbpool): State<SqlitePool>) -> Result<Json<Vec<Word>>, AppError> {
    Word::list(dbpool).await.map(Json::from)
}

/// Return a random word from /word
pub async fn word_random(State(dbpool): State<SqlitePool>) -> Result<Json<Word>, AppError> {
    Word::random(dbpool).await.map(Json::from)
}

/// Add a new word to the database (admin only)
pub async fn word_create(
    State(dbpool): State<SqlitePool>,
    Json(new_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::create(dbpool, new_word).await.map(Json::from)
}

/// Get a single word by id (admin only)
pub async fn word_read(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<Json<Word>, AppError> {
    Word::read(dbpool, id).await.map(Json::from)
}

/// Update an existing word (admin only)
pub async fn word_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
    Json(updated_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::update(dbpool, id, updated_word).await.map(Json::from)
}

/// Delete a word from the database (admin only)
pub async fn word_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<(), AppError> {
    Word::delete(dbpool, id).await
}
