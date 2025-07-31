// API handler for general routes
use axum::extract::State;
use sqlx::SqlitePool;

use crate::error::Error;

/// Test the database connection from /ready
pub async fn ping(State(dbpool): State<SqlitePool>) -> Result<String, Error> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "ok".to_string())
        .map_err(Into::into)
}
