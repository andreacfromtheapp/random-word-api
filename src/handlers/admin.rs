// API handlers for word management
use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::word::{UpsertWord, Word};

/// List all words as JSON array of objects (requires auth and admin privileges)
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/words",
    operation_id = "custom_read_all_words_listed",
    tag = "administration_endpoints",
    responses(
        (status = 200, description = "Listed every single word successfully", body = [Word]),
        (status = 404, description = "Couldn't list words. Does your database contain any?"),
    )
)]
pub async fn word_list(State(dbpool): State<SqlitePool>) -> Result<Json<Vec<Word>>, AppError> {
    Word::list(dbpool).await.map(Json::from)
}

/// Add a new word to the database (requires auth and admin privileges)
#[utoipa::path(
    post,
    context_path = "/admin",
    path = "/words",
    operation_id = "custom_post_word",
    tag = "administration_endpoints",
    request_body(content = UpsertWord, description = "Word to add to the database", content_type = "application/json"),
    responses(
        (status = 200, description = "Word with {id} successfully added to the database", body = Word),
        (status = 415, description = "Please provide a valid word definition in your JSON body"),
        (status = 422, description = "Please provide a valid word definition in your JSON body"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn word_create(
    State(dbpool): State<SqlitePool>,
    Json(new_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::create(dbpool, new_word).await.map(Json::from)
}

/// Get a single word by id (requires auth and admin privileges)
#[utoipa::path(
    get,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "custom_read_word",
    tag = "administration_endpoints",
    responses (
        (status = 200, description = "Word with {id} returned successfully", body = Word),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word database id to get Word for"),
    )
)]
pub async fn word_read(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<Json<Word>, AppError> {
    Word::read(dbpool, id).await.map(Json::from)
}

/// Update an existing word by id (requires auth and admin privileges)
#[utoipa::path(
    put,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "custom_put_word",
    tag = "administration_endpoints",
    request_body(content = UpsertWord, description = "Word to update in the database", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} updated successfully", body = Word),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word id to update Word for"),
    )
)]
pub async fn word_update(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
    Json(updated_word): Json<UpsertWord>,
) -> Result<Json<Word>, AppError> {
    Word::update(dbpool, id, updated_word).await.map(Json::from)
}

/// Delete a word from the database by id (requires auth and admin privileges)
#[utoipa::path(
    delete,
    context_path = "/admin",
    path = "/words/{id}",
    operation_id = "custom_delete_word",
    tag = "administration_endpoints",
    request_body(content = u32, description = "Word to delete from the database", content_type = "application/json"),
    responses (
        (status = 200, description = "Word with {id} deleted successfully"),
        (status = 404, description = "Couldn't find the word with {id}"),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("id" = u32, Path, description = "Word id to delete Word for"),
    )
)]
pub async fn word_delete(
    State(dbpool): State<SqlitePool>,
    Path(id): Path<u32>,
) -> Result<(), AppError> {
    Word::delete(dbpool, id).await
}
