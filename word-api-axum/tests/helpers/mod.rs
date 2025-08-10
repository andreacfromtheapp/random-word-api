//! Test helper utilities for simple integration testing
//!
//! This module provides essential utilities for setting up test environments
//! for the simple integration tests. It focuses on basic functionality without
//! over-engineering or complex patterns.
//!
//! ## Test Patterns
//!
//! ### Database Setup
//! - Use `create_test_database()` for isolated database instances
//! - Use `create_test_server()` for full application testing
//! - Use `create_test_server_memory()` for fast in-memory testing
//!
//! ### Cleanup
//! - Tests using `populate_test_data()` with suffixes are automatically isolated
//! - Use `database::cleanup_test_data()` for explicit cleanup if needed
//! - Temporary databases are automatically cleaned up via `NamedTempFile`

use anyhow::{Context, Result};
use axum_test::TestServer;
use sqlx::{Pool, Sqlite};
use std::sync::{Arc, Mutex};

use tempfile::NamedTempFile;
use word_api_axum::{
    init_dbpool, models::apiconfig::ApiConfig, routes::create_router, state::AppState,
};

/// Test utilities for database operations
pub mod database;

/// Test data fixtures and factories
pub mod fixtures;

/// Creates a temporary SQLite database for testing with migrations applied
#[allow(dead_code)]
pub async fn create_test_database() -> Result<(Pool<Sqlite>, NamedTempFile)> {
    let temp_file = NamedTempFile::new().context("Failed to create temporary database file")?;

    let db_path = temp_file.path().to_string_lossy();
    let db_url = format!("sqlite://{db_path}");

    let pool = init_dbpool(&db_url)
        .await
        .context("Failed to initialize test database pool")?;

    Ok((pool, temp_file))
}

/// Creates a test server with the full application router
#[allow(dead_code)]
pub async fn create_test_server() -> Result<(TestServer, NamedTempFile)> {
    use std::net::IpAddr;
    use word_api_axum::models::apiconfig::OpenApiDocs;

    let (pool, temp_file) = create_test_database().await?;
    let db_url = format!("sqlite://{}", temp_file.path().to_string_lossy());

    let config = ApiConfig {
        address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 0, // Let the OS assign a port
        database_url: db_url,
        openapi: OpenApiDocs {
            enable_swagger_ui: true,
            enable_redoc: true,
            enable_scalar: true,
            enable_rapidoc: true,
        },
    };

    let state = AppState {
        apiconfig: Arc::new(Mutex::new(config)),
        dbpool: pool,
    };

    let router = create_router(state).await;
    let server = TestServer::new(router).context("Failed to create test server")?;

    Ok((server, temp_file))
}

/// Creates a test server with access to the database pool for tests that need direct DB access
#[allow(dead_code)]
pub async fn create_test_server_with_pool() -> Result<(TestServer, NamedTempFile, Pool<Sqlite>)> {
    use std::net::IpAddr;
    use word_api_axum::models::apiconfig::OpenApiDocs;

    let (pool, temp_file) = create_test_database().await?;
    let db_url = format!("sqlite://{}", temp_file.path().to_string_lossy());

    let config = ApiConfig {
        address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 0, // Let the OS assign a port
        database_url: db_url,
        openapi: OpenApiDocs {
            enable_swagger_ui: true,
            enable_redoc: true,
            enable_scalar: true,
            enable_rapidoc: true,
        },
    };

    let state = AppState {
        apiconfig: Arc::new(Mutex::new(config)),
        dbpool: pool.clone(),
    };

    let router = create_router(state).await;
    let server = TestServer::new(router).context("Failed to create test server")?;

    Ok((server, temp_file, pool))
}

/// Creates a fast test server with in-memory database for read-only testing
#[allow(dead_code)]
pub async fn create_test_server_memory() -> Result<(TestServer, Pool<Sqlite>)> {
    use std::net::IpAddr;
    use word_api_axum::models::apiconfig::OpenApiDocs;

    let db_url = "sqlite://:memory:";
    let pool = init_dbpool(db_url)
        .await
        .context("Failed to initialize in-memory test database pool")?;

    let config = ApiConfig {
        address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 0, // Let the OS assign a port
        database_url: "sqlite://:memory:".to_string(),
        openapi: OpenApiDocs {
            enable_swagger_ui: true,
            enable_redoc: true,
            enable_scalar: true,
            enable_rapidoc: true,
        },
    };

    let state = AppState {
        apiconfig: Arc::new(Mutex::new(config)),
        dbpool: pool.clone(),
    };

    let router = create_router(state).await;
    let server = TestServer::new(router).context("Failed to create test server")?;

    Ok((server, pool))
}
