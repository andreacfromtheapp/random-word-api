// API handlers for word retrieval from the publicly exposed endpoints
use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::model::word::Word;

/// Return a random word as JSON object
#[utoipa::path(
    get,
    path = "/word",
    operation_id = "custom_read_random_word",
    tag = "publicly_exposed_endpoints",
    responses(
        (status = 200, description = "A random word returned successfully", body = Word),
        (status = 404, description = "Couldn't return a random word. Does your database contain any?"),
    ),
)]
pub async fn word_random(State(dbpool): State<SqlitePool>) -> Result<Json<Word>, AppError> {
    Word::random(dbpool).await.map(Json::from)
}
