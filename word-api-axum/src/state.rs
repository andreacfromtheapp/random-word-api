//! Shared application state
//!
//! Thread-safe state container for configuration and database connections,
//! shared across all request handlers.

use std::sync::{Arc, Mutex};

use crate::config::ApiConfig;

/// Central application state shared across all HTTP handlers
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe configuration container for runtime settings
    pub apiconfig: Arc<Mutex<ApiConfig>>,

    /// SQLite database connection pool for efficient query execution
    pub dbpool: sqlx::Pool<sqlx::Sqlite>,
}
