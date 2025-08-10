//! Application state module
//!
//! This module defines the central application state structure that is shared
//! across all HTTP handlers and middleware. It provides thread-safe access to
//! configuration settings and database connections through Axum's state system.
//!
//! # State Management
//!
//! The application state uses Rust's ownership and concurrency primitives to
//! ensure safe access from multiple handlers simultaneously:
//! - `Arc<Mutex<T>>` for thread-safe configuration access
//! - Connection pooling for efficient database access
//! - Clone trait for easy state sharing across handlers
//!
//! # Integration with Axum
//!
//! The state is designed to work seamlessly with Axum's dependency injection
//! system, allowing handlers to access shared resources through the `State`
//! extractor without manual resource management.
//!
//! # Concurrency
//!
//! All state components are designed for concurrent access:
//! - Configuration changes are protected by mutex locks
//! - Database connections use internal pooling for concurrent queries
//! - State cloning is lightweight and safe for multi-threaded environments

use std::sync::{Arc, Mutex};

use crate::models::apiconfig::ApiConfig;

/// Central application state shared across all HTTP handlers.
///
/// This structure contains all shared resources needed by the API handlers,
/// including configuration settings and database connectivity. It is designed
/// to be thread-safe and efficiently cloneable for use with Axum's state
/// management system.
///
/// # Thread Safety
///
/// All fields are wrapped in appropriate thread-safe containers:
/// - Configuration uses `Arc<Mutex<T>>` for shared, mutable access
/// - Database pool has internal synchronization for concurrent connections
///
/// # Cloning Behavior
///
/// The `Clone` implementation creates lightweight references to the same
/// underlying data rather than deep copies:
/// - `Arc` reference counting for shared ownership
/// - Pool connections are shared, not duplicated
///
/// # Usage with Axum
///
/// This state is passed to Axum routers and extracted in handlers using
/// the `State` extractor, providing clean dependency injection without
/// manual resource management.
///
/// # Fields
///
/// - `config`: Thread-safe access to runtime configuration that can be
///   modified during application lifetime for dynamic reconfiguration
/// - `dbpool`: SQLite connection pool for efficient database access with
///   automatic connection management and reuse
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe configuration container for runtime settings.
    ///
    /// Uses `Arc<Mutex<ApiConfig>>` to allow multiple handlers to safely
    /// read and potentially modify configuration settings. The Arc provides
    /// shared ownership while Mutex ensures exclusive access for modifications.
    ///
    /// # Access Pattern
    ///
    /// Handlers should acquire the lock only for the duration needed:
    /// - Read operations: Short-lived lock acquisition
    /// - Write operations: Minimal critical sections
    /// - Configuration queries: Clone values outside the lock
    ///
    /// # Dynamic Reconfiguration
    ///
    /// The mutex allows for runtime configuration changes without requiring
    /// application restarts, enabling dynamic feature toggling and parameter
    /// adjustment.
    pub apiconfig: Arc<Mutex<ApiConfig>>,

    /// SQLite database connection pool for efficient query execution.
    ///
    /// Provides a managed pool of database connections that are automatically
    /// created, reused, and cleaned up as needed. The pool handles connection
    /// lifecycle and provides concurrent access for multiple handlers.
    ///
    /// # Connection Management
    ///
    /// The pool automatically manages:
    /// - Connection creation and initialization
    /// - Connection reuse and pooling
    /// - Connection cleanup and closure
    /// - Error recovery and reconnection
    ///
    /// # Concurrent Access
    ///
    /// Multiple handlers can safely acquire connections simultaneously
    /// without manual synchronization, as the pool handles all concurrency
    /// internally.
    pub dbpool: sqlx::Pool<sqlx::Sqlite>,
}
