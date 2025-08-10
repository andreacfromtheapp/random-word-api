//! Performance monitoring and reliability tests
//!
//! This module contains dedicated performance tests to ensure the API meets
//! performance requirements and maintains reliability under various conditions.
//! These tests complement the functional tests by focusing on non-functional requirements.

use anyhow::Result;
use axum::http::StatusCode;
use sqlx::{Pool, Sqlite};
use std::time::{Duration, Instant};

mod helpers;
use helpers::{
    create_test_server,
    database::{cleanup_test_data, count_words, populate_test_data},
};

/// Simple performance measurement utility
async fn measure_operation<F, T>(operation: F) -> Result<(T, Duration)>
where
    F: std::future::Future<Output = Result<T>>,
{
    let start = Instant::now();
    let result = operation.await?;
    let duration = start.elapsed();
    Ok((result, duration))
}

/// Assert that operation duration is within threshold
fn assert_duration(duration: Duration, max_duration: Duration, operation_name: &str) {
    assert!(
        duration <= max_duration,
        "{operation_name} took {duration:?}, expected <= {max_duration:?}"
    );
}

/// Validate database health with a simple query
async fn validate_db_health(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database health check failed: {}", e))?;
    Ok(())
}

#[tokio::test]
async fn test_infrastructure_setup_performance() -> Result<()> {
    // Test database setup performance
    let (_, db_duration) = measure_operation(helpers::create_test_database()).await?;
    assert_duration(db_duration, Duration::from_millis(2000), "database_setup");

    // Test server startup performance
    let (_, server_duration) = measure_operation(create_test_server()).await?;
    assert_duration(
        server_duration,
        Duration::from_millis(2000),
        "server_startup",
    );

    Ok(())
}

#[tokio::test]
async fn test_database_operation_performance() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;
    populate_test_data(&pool, "perf1").await?;

    // Test basic query performance
    let (count, query_duration) = measure_operation(count_words(&pool)).await?;
    assert!(count > 0, "Should have test data");
    assert_duration(query_duration, Duration::from_millis(100), "count_query");

    // Test data insertion performance
    let (_, insert_duration) = measure_operation(populate_test_data(&pool, "perf2")).await?;
    assert_duration(
        insert_duration,
        Duration::from_millis(1000),
        "data_insertion",
    );

    Ok(())
}

#[tokio::test]
async fn test_api_response_performance() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let mut response_times = Vec::new();

    // Test multiple requests for consistency and load handling
    for i in 0..5 {
        let (response, duration) =
            measure_operation(async { Ok(server.get("/en/word").await) }).await?;

        assert!(
            response.status_code() <= StatusCode::OK
                || response.status_code() == StatusCode::NO_CONTENT
        );
        response_times.push(duration);

        // Validate individual request performance
        assert_duration(
            duration,
            Duration::from_millis(500),
            &format!("api_request_{i}"),
        );
    }

    // Check response time consistency
    let avg_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let max_variance = Duration::from_millis(200);

    for (i, duration) in response_times.iter().enumerate() {
        let variance = (*duration).abs_diff(avg_time);
        assert!(
            variance <= max_variance,
            "Request {i} variance {variance:?} exceeds max variance {max_variance:?}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_database_reliability_under_load() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

    // Test database health before load
    validate_db_health(&pool).await?;

    // Simulate load with fewer operations for faster execution
    for i in 0..3 {
        let suffix = format!("load{i}");
        populate_test_data(&pool, &suffix).await?;
        let count = count_words(&pool).await?;
        assert!(
            count > 0,
            "Load operation {i} should have inserted data, got count: {count}"
        );
    }

    // Final health check
    validate_db_health(&pool).await?;

    // Cleanup
    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling_performance() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test that error responses are also fast
    let (response, error_duration) =
        measure_operation(async { Ok(server.get("/invalid/endpoint").await) }).await?;

    assert!(response.status_code() >= StatusCode::BAD_REQUEST);
    assert_duration(error_duration, Duration::from_millis(500), "error_response");

    Ok(())
}

#[tokio::test]
async fn test_health_check_performance() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test single health check performance
    let (response, health_duration) =
        measure_operation(async { Ok(server.get("/health/alive").await) }).await?;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_duration(health_duration, Duration::from_millis(50), "health_check");

    // Test multiple health checks performance
    let (_, multiple_duration) = measure_operation(async {
        for _ in 0..3 {
            let response = server.get("/health/alive").await;
            assert_eq!(response.status_code(), StatusCode::OK);
        }
        Ok(())
    })
    .await?;

    // Multiple health checks should still be very fast
    assert_duration(
        multiple_duration,
        Duration::from_secs(1),
        "multiple_health_checks",
    );

    Ok(())
}
