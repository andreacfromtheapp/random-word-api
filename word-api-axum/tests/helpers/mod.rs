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
//! - Use `create_test_server_with_pool()` when direct DB access is needed
//!
//! ### Performance Testing
//! - Use `measure_test_performance()` to monitor test execution times
//! - Use `assert_test_performance()` to validate performance requirements
//! - Database operations should complete within 100ms for unit tests
//!
//! ### Cleanup
//! - Tests using `populate_test_data()` with suffixes are automatically isolated
//! - Use `database::cleanup_test_data()` for explicit cleanup if needed
//! - Temporary databases are automatically cleaned up via `NamedTempFile`

use anyhow::{Context, Result};
use axum_test::TestServer;
use sqlx::{Pool, Sqlite};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use word_api_axum::{
    init_dbpool, models::apiconfig::ApiConfig, routes::create_router, state::AppState,
};

/// Test utilities for database operations
pub mod database;

/// Test data fixtures and factories
pub mod fixtures;

/// Creates a temporary SQLite database for testing with migrations applied
pub async fn create_test_database() -> Result<(Pool<Sqlite>, NamedTempFile)> {
    let temp_file = NamedTempFile::new().context("Failed to create temporary database file")?;

    let db_path = temp_file.path().to_string_lossy();
    let db_url = format!("sqlite://{db_path}");

    let pool = init_dbpool(&db_url)
        .await
        .context("Failed to initialize test database pool")?;

    Ok((pool, temp_file))
}

/// Creates a test API configuration with default values
pub fn create_test_config(database_url: String) -> ApiConfig {
    use std::net::IpAddr;
    use word_api_axum::models::apiconfig::OpenApiDocs;

    ApiConfig {
        address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 0, // Let the OS assign a port
        database_url,
        openapi: OpenApiDocs {
            enable_swagger_ui: true,
            enable_redoc: true,
            enable_scalar: true,
            enable_rapidoc: true,
        },
    }
}

/// Creates a test application state with a temporary database
pub async fn create_test_app_state() -> Result<(AppState, NamedTempFile)> {
    let (pool, temp_file) = create_test_database().await?;
    let db_url = format!("sqlite://{}", temp_file.path().to_string_lossy());
    let config = create_test_config(db_url);

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
        dbpool: pool,
    };

    Ok((state, temp_file))
}

/// Creates a test server with the full application router
pub async fn create_test_server() -> Result<(TestServer, NamedTempFile)> {
    let (state, temp_file) = create_test_app_state().await?;
    let router = create_router(state).await;
    let server = TestServer::new(router).context("Failed to create test server")?;

    Ok((server, temp_file))
}

/// Creates a test server with access to the database pool for tests that need direct DB access
#[allow(dead_code)]
pub async fn create_test_server_with_pool() -> Result<(TestServer, NamedTempFile, Pool<Sqlite>)> {
    let (pool, temp_file) = create_test_database().await?;
    let db_url = format!("sqlite://{}", temp_file.path().to_string_lossy());
    let config = create_test_config(db_url);

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
        dbpool: pool.clone(),
    };

    let router = create_router(state).await;
    let server = TestServer::new(router).context("Failed to create test server")?;

    Ok((server, temp_file, pool))
}

/// Performance monitoring for test operations
pub struct TestPerformanceMetrics {
    pub duration: Duration,
    pub operation_name: String,
}

/// Measures the performance of a test operation
pub async fn measure_test_performance<F, T>(
    operation_name: &str,
    operation: F,
) -> Result<(T, TestPerformanceMetrics)>
where
    F: std::future::Future<Output = Result<T>>,
{
    let start = Instant::now();
    let result = operation.await?;
    let duration = start.elapsed();

    let metrics = TestPerformanceMetrics {
        duration,
        operation_name: operation_name.to_string(),
    };

    Ok((result, metrics))
}

/// Asserts that a test operation meets performance requirements
pub fn assert_test_performance(metrics: &TestPerformanceMetrics, max_duration: Duration) {
    assert!(
        metrics.duration <= max_duration,
        "Test operation '{}' took {:?}, expected <= {:?}",
        metrics.operation_name,
        metrics.duration,
        max_duration
    );
}

/// Standard performance thresholds for different operation types
pub mod performance_thresholds {
    use std::time::Duration;

    /// Database operations should complete quickly
    pub const DATABASE_OPERATION: Duration = Duration::from_millis(100);

    /// API requests should be fast
    #[allow(dead_code)]
    pub const API_REQUEST: Duration = Duration::from_millis(500);

    /// Test setup should be efficient
    pub const TEST_SETUP: Duration = Duration::from_millis(2000);

    /// Bulk operations have more relaxed timing
    #[allow(dead_code)]
    pub const BULK_OPERATION: Duration = Duration::from_millis(1000);
}

/// Test reliability utilities
pub mod reliability {
    use super::*;

    /// Retries an operation up to max_attempts times
    pub async fn retry_operation<F, T, E>(operation: F, max_attempts: usize) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug + Send + 'static,
    {
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_attempts {
                        tokio::time::sleep(Duration::from_millis(10 * attempt as u64)).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Operation failed after {} attempts. Last error: {:?}",
            max_attempts,
            last_error.unwrap()
        ))
    }

    /// Validates that a database connection is healthy
    pub async fn validate_db_health(pool: &Pool<Sqlite>) -> Result<()> {
        // Simple health check query
        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_database() -> Result<()> {
        let (pool, _temp_file) = create_test_database().await?;

        // Verify database is functional (count should be non-negative)
        let count = database::count_words(&pool).await?;
        assert!(count >= 0, "Database should return valid word count");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_test_server() -> Result<()> {
        let (server, _temp_file) = create_test_server().await?;

        // Basic smoke test - health endpoint should be reachable
        let response = server.get("/health/alive").await;
        assert!(
            response.status_code() == 200,
            "Health endpoint should be reachable"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_measurement() -> Result<()> {
        let (pool, _temp_file) = create_test_database().await?;

        let (count, metrics) = measure_test_performance("count_words_operation", async {
            database::count_words(&pool).await
        })
        .await?;

        assert!(count >= 0, "Should get valid count");
        assert_test_performance(&metrics, performance_thresholds::DATABASE_OPERATION);

        Ok(())
    }

    #[tokio::test]
    async fn test_server_creation_performance() -> Result<()> {
        let ((_server, _temp_file), metrics) =
            measure_test_performance("create_test_server", create_test_server()).await?;

        assert_test_performance(&metrics, performance_thresholds::TEST_SETUP);

        Ok(())
    }

    #[tokio::test]
    async fn test_database_health_validation() -> Result<()> {
        let (pool, _temp_file) = create_test_database().await?;

        reliability::validate_db_health(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_retry_operation() -> Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let attempt_count = Arc::new(AtomicUsize::new(0));
        let count_clone = attempt_count.clone();

        let result = reliability::retry_operation(
            move || {
                let count = count_clone.clone();
                Box::pin(async move {
                    let current = count.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        // Fail first 2 attempts
                        Err("Simulated failure")
                    } else {
                        Ok("Success")
                    }
                })
            },
            3,
        )
        .await?;

        assert_eq!(result, "Success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);

        Ok(())
    }
}
