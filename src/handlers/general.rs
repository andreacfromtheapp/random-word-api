// API handlers for general routes
use axum::extract::State;
use sqlx::SqlitePool;

use crate::error::AppError;

/// Test the database connection from the API
#[utoipa::path(
    get,
    path = "/ready",
    operation_id = "api_db_connection_test",
    tag = "generic_healthcheck_handlers",
    responses(
        (status = 200, description = "OK. The API can connect to the database", body = String),
        (status = 500, description = "Internal server error. Couldn't connect to the database"),
    )
)]
pub async fn ping(State(dbpool): State<SqlitePool>) -> Result<String, AppError> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "OK. The API can connect to the database".to_string())
        .map_err(Into::into)
}
