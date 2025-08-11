//! Test helper utilities for integration testing
//!
//! Provides streamlined server creation functions for different test scenarios:
//! - `create_test_server()` - Isolated database for write operations
//! - `create_test_server_streamlined()` - Shared database for read-only tests
//! - `create_test_server_memory()` - In-memory database for empty scenarios

use anyhow::{Context, Result};
use axum_test::TestServer;
use sqlx::{Pool, Sqlite};
use std::sync::{Arc, Mutex};

use tempfile::NamedTempFile;
use word_api_axum::{
    init_dbpool, models::apiconfig::ApiConfig, routes::create_router, state::AppState,
};

/// Test data utilities
pub mod test_data;

/// Shared database utilities
pub mod shared_db;

/// Creates a temporary SQLite database for testing with migrations applied
#[allow(dead_code)]
async fn create_test_database() -> Result<(Pool<Sqlite>, NamedTempFile)> {
    let temp_file = NamedTempFile::new().context("Failed to create temporary database file")?;

    let db_path = temp_file.path().to_string_lossy();
    let db_url = format!("sqlite://{db_path}");

    let pool = init_dbpool(&db_url)
        .await
        .context("Failed to initialize test database pool")?;

    Ok((pool, temp_file))
}

/// Creates test server with isolated database for write operations
#[allow(dead_code)]
pub async fn create_test_server() -> Result<(TestServer, NamedTempFile)> {
    let (pool, temp_file) = create_test_database().await?;
    let server = create_server_with_pool(pool).await?;
    Ok((server, temp_file))
}

/// Creates test server with isolated database and direct pool access
#[allow(dead_code)]
pub async fn create_test_server_with_pool() -> Result<(TestServer, NamedTempFile, Pool<Sqlite>)> {
    let (pool, temp_file) = create_test_database().await?;
    let server = create_server_with_pool(pool.clone()).await?;
    Ok((server, temp_file, pool))
}

/// Creates test server with in-memory database for empty scenarios
#[allow(dead_code)]
pub async fn create_test_server_memory() -> Result<(TestServer, Pool<Sqlite>)> {
    let db_url = "sqlite://:memory:";
    let pool = init_dbpool(db_url)
        .await
        .context("Failed to initialize in-memory test database pool")?;

    let server = create_server_with_pool(pool.clone()).await?;
    Ok((server, pool))
}

/// Creates test server with shared database for read-only tests
#[allow(dead_code)]
pub async fn create_test_server_streamlined() -> Result<TestServer> {
    let pool = shared_db::get_shared_database().await?.clone();
    create_server_with_pool(pool).await
}

/// Internal helper for creating test servers with database pool
async fn create_server_with_pool(pool: Pool<Sqlite>) -> Result<TestServer> {
    use std::net::IpAddr;
    use word_api_axum::models::apiconfig::OpenApiDocs;

    let config = ApiConfig {
        address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 0,
        database_url: "sqlite://test".to_string(),
        openapi: OpenApiDocs {
            enable_swagger_ui: false,
            enable_redoc: false,
            enable_scalar: false,
            enable_rapidoc: false,
        },
    };

    let state = AppState {
        apiconfig: Arc::new(Mutex::new(config)),
        dbpool: pool,
    };

    let router = create_router(state).await;
    TestServer::new(router).context("Failed to create test server")
}
