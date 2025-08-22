//! Shared application state
//!
//! Thread-safe state container for configuration and database connections,
//! shared across all request handlers.

use std::sync::{Arc, Mutex};

use crate::config::ApiConfig;
use crate::error::SqlxError;

/// Central application state shared across all HTTP handlers
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe configuration container for runtime settings
    pub apiconfig: Arc<Mutex<ApiConfig>>,

    /// SQLite database connection pool for efficient query execution
    pub dbpool: sqlx::Pool<sqlx::Sqlite>,
}

/// Configure the database pool with optimized settings
///
/// Creates a SQLite connection pool with:
/// - WAL mode for better concurrency
/// - Connection pooling with timeouts
/// - Automatic database creation
/// - Migration execution
pub async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, SqlxError> {
    use sqlx::sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
    };
    use std::str::FromStr;
    use std::time::Duration;

    let dbpool = SqlitePoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(10)))
        .connect_with(
            SqliteConnectOptions::from_str(db_url)?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .pragma("cache_size", "1000")
                .pragma("temp_store", "memory"),
        )
        .await?;

    sqlx::migrate!("./migrations").run(&dbpool).await?;

    Ok(dbpool)
}
