// API handlers for general routes
use axum::extract::State;
use sqlx::SqlitePool;

use crate::error::AppError;

/// Test the database connection from /ready
pub async fn ping(State(dbpool): State<SqlitePool>) -> Result<String, AppError> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "ok".to_string())
        .map_err(Into::into)
}
