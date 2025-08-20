//! Test helper utilities for integration testing
//!
//! Provides optimized server creation functions for different test scenarios:
//! - `create_test_server()` - Isolated in-memory database for write operations
//! - `create_test_server_streamlined()` - In-memory database for read-only tests
//! - `create_test_server_memory()` - In-memory database with direct pool access
//! - `create_test_server_with_pool()` - Server creation with pool return for advanced testing
//!
//! Each function creates a complete test environment with database migrations
//! and proper state initialization for comprehensive API testing.

use anyhow::{Context, Result};
use axum_test::TestServer;
use sqlx::{Pool, Sqlite};
use std::sync::{Arc, Mutex};

use word_api_axum::{config::ApiConfig, init_dbpool, routes::create_router, state::AppState};

/// Test data utilities
pub mod test_data;

/// Creates test server with isolated in-memory database for write operations
#[allow(dead_code)]
pub async fn create_test_server() -> Result<TestServer> {
    let (server, _pool) = create_test_server_memory().await?;
    Ok(server)
}

/// Creates test server with isolated in-memory database and direct pool access
#[allow(dead_code)]
pub async fn create_test_server_with_pool() -> Result<(TestServer, Pool<Sqlite>)> {
    create_test_server_memory().await
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

/// Creates test server with in-memory database for read-only tests
#[allow(dead_code)]
pub async fn create_test_server_streamlined() -> Result<TestServer> {
    let (server, _pool) = create_test_server_memory().await?;
    Ok(server)
}

/// Internal helper for creating test servers with database pool
async fn create_server_with_pool(pool: Pool<Sqlite>) -> Result<TestServer> {
    use std::net::IpAddr;
    use word_api_axum::config::OpenApiDocs;

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
        jwt_secret: "test_jwt_secret".to_string(),
    };

    let state = AppState {
        apiconfig: Arc::new(Mutex::new(config)),
        dbpool: pool,
    };

    let router = create_router(state).await;
    TestServer::new(router).context("Failed to create test server")
}
