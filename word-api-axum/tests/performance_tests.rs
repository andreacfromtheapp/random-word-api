//! Performance monitoring and reliability tests
//!
//! This module contains dedicated performance tests to ensure the API meets
//! performance requirements and maintains reliability under various conditions.
//! These tests complement the functional tests by focusing on non-functional requirements.

use anyhow::Result;
use axum::http::StatusCode;
use serial_test::serial;
use std::time::Duration;

mod helpers;
use helpers::{
    assert_test_performance, create_test_server, create_test_server_with_pool,
    database::{cleanup_test_data, count_words, measure_db_operation, populate_test_data},
    measure_test_performance, performance_thresholds, reliability,
};

#[tokio::test]
async fn test_database_setup_performance() -> Result<()> {
    let (_, setup_metrics) = measure_test_performance("database_setup", async {
        let (pool, temp_file) = helpers::create_test_database().await?;
        Ok((pool, temp_file))
    })
    .await?;

    assert_test_performance(&setup_metrics, performance_thresholds::TEST_SETUP);
    Ok(())
}

#[tokio::test]
async fn test_server_startup_performance() -> Result<()> {
    let (_, startup_metrics) = measure_test_performance("server_startup", async {
        helpers::create_test_server().await
    })
    .await?;

    assert_test_performance(&startup_metrics, performance_thresholds::TEST_SETUP);
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_database_operation_performance() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;
    populate_test_data(&pool, "perf1").await?;

    // Test basic query performance
    let (count, db_metrics) = measure_db_operation(count_words(&pool)).await?;
    assert!(count > 0, "Should have test data");
    helpers::database::assert_performance(&db_metrics, performance_thresholds::DATABASE_OPERATION);

    // Test data insertion performance
    let (_, insert_metrics) = measure_db_operation(populate_test_data(&pool, "perf2")).await?;
    helpers::database::assert_performance(&insert_metrics, performance_thresholds::BULK_OPERATION);

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_api_response_time_consistency() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "consistency").await?;

    let mut response_times = Vec::new();

    // Make multiple requests to measure consistency
    for i in 0..10 {
        let (response, metrics) =
            measure_test_performance(&format!("consistency_test_{i}"), async {
                Ok(server.get("/en/word").await)
            })
            .await?;

        assert_eq!(response.status_code(), StatusCode::OK);
        response_times.push(metrics.duration);
    }

    // Check that all responses are within acceptable range
    for (i, duration) in response_times.iter().enumerate() {
        assert!(
            *duration <= performance_thresholds::API_REQUEST,
            "Request {} took {:?}, exceeds threshold {:?}",
            i,
            duration,
            performance_thresholds::API_REQUEST
        );
    }

    // Check response time variance (basic consistency check)
    let avg_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let max_variance = Duration::from_millis(200);

    for (i, duration) in response_times.iter().enumerate() {
        let variance = (*duration).abs_diff(avg_time);

        assert!(
            variance <= max_variance,
            "Request {i} variance {variance:?} exceeds max variance {max_variance:?}"
        );
    }

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_api_load_performance() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "concurrent").await?;

    let load_requests = 5;
    // Removed handles vector as we're using sequential requests instead of concurrent

    let start_time = std::time::Instant::now();

    // Launch sequential requests to test load handling
    let mut all_metrics = Vec::new();
    for i in 0..load_requests {
        let (response, metrics) = measure_test_performance(&format!("load_request_{i}"), async {
            Ok(server.get("/en/word").await)
        })
        .await?;

        assert_eq!(response.status_code(), StatusCode::OK);
        all_metrics.push(metrics);
    }

    let total_time = start_time.elapsed();

    // Validate individual request performance
    for (i, metrics) in all_metrics.iter().enumerate() {
        assert_test_performance(metrics, performance_thresholds::API_REQUEST);
        println!("Load request {} completed in {:?}", i, metrics.duration);
    }

    // Validate overall load handling performance
    assert!(
        total_time <= Duration::from_millis(2000),
        "Load requests took {total_time:?}, expected <= 2000ms"
    );

    println!("Completed {load_requests} load requests in {total_time:?}");

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_database_reliability_under_load() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

    // Test database health before load
    reliability::validate_db_health(&pool).await?;

    // Simulate load with sequential operations for reliability testing
    let mut counts = Vec::new();
    for i in 0..5 {
        let suffix = format!("load{i}");
        populate_test_data(&pool, &suffix).await?;
        let count = count_words(&pool).await?;
        counts.push(count);
    }

    // Verify all operations succeeded and database is still healthy
    for (i, count) in counts.iter().enumerate() {
        assert!(
            *count > 0,
            "Load operation {i} should have inserted data, got count: {count}"
        );
    }

    // Final health check
    reliability::validate_db_health(&pool).await?;

    // Cleanup
    helpers::database::clear_words_table(&pool).await?;
    Ok(())
}

#[tokio::test]
async fn test_memory_usage_monitoring() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

    // Test memory usage during bulk operations
    let (_, db_metrics) = measure_db_operation(async {
        for i in 0..100 {
            let word = helpers::fixtures::WordFactory::create_with_suffix(
                "memory",
                "noun",
                &format!("test{i}"),
            );
            let _ = word_api_axum::models::word::Word::create(pool.clone(), "en", word).await;
        }
        Ok(())
    })
    .await?;

    // Assert reasonable memory usage (10MB max increase)
    helpers::database::assert_memory_usage(&db_metrics, 10_000_000);

    helpers::database::clear_words_table(&pool).await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling_performance() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test that error responses are also fast
    let (response, error_metrics) = measure_test_performance("error_response", async {
        Ok(server.get("/invalid/endpoint").await)
    })
    .await?;

    assert!(response.status_code() >= StatusCode::BAD_REQUEST);
    assert_test_performance(&error_metrics, performance_thresholds::API_REQUEST);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_retry_mechanism_reliability() -> Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let attempt_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = attempt_counter.clone();

    // Test the retry mechanism with a flaky operation
    let result = reliability::retry_operation(
        move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let attempt = counter.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err("Simulated failure")
                } else {
                    Ok("Success after retries")
                }
            })
        },
        3,
    )
    .await?;

    assert_eq!(result, "Success after retries");
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);

    Ok(())
}

#[tokio::test]
async fn test_health_check_performance() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let (response, health_metrics) = measure_test_performance("health_check", async {
        Ok(server.get("/health/alive").await)
    })
    .await?;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Health checks should be very fast
    assert_test_performance(&health_metrics, Duration::from_millis(50));

    Ok(())
}

#[tokio::test]
async fn test_configuration_load_performance() -> Result<()> {
    let (_, config_metrics) = measure_test_performance("config_creation", async {
        let config = helpers::create_test_config("sqlite://test.db".to_string());
        Ok(config)
    })
    .await?;

    // Configuration creation should be instant
    assert_test_performance(&config_metrics, Duration::from_millis(10));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_bulk_data_insertion_performance() -> Result<()> {
    let (pool, _temp_file) = helpers::create_test_database().await?;

    let (_, bulk_metrics) = measure_db_operation(async {
        // Insert test data with predefined valid pronunciations
        let valid_pronunciations = [
            "/bʌlkə/",
            "/bʌlkɪ/",
            "/bʌlkʊ/",
            "/bʌlkɛ/",
            "/bʌlkɔ/",
            "/bʌlkæ/",
            "/bʌlkʌ/",
            "/bʌlkɑ/",
            "/bʌlkɒ/",
            "/bʌlkɜ/",
        ];

        for (i, e) in valid_pronunciations.iter().enumerate() {
            let word = word_api_axum::models::word::UpsertWord {
                word: format!("bulkword{i}"),
                definition: format!("Bulk test word number {i}"),
                pronunciation: e.to_string(),
                word_type: "noun".to_string(),
            };
            let _ = word_api_axum::models::word::Word::create(pool.clone(), "en", word)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to insert bulk word: {:?}", e))?;
        }
        Ok(())
    })
    .await?;

    helpers::database::assert_performance(&bulk_metrics, performance_thresholds::BULK_OPERATION);

    // Verify data was inserted
    let final_count = count_words(&pool).await?;
    assert!(final_count >= 10, "Should have inserted bulk data"); // 1 word per iteration

    helpers::database::clear_words_table(&pool).await?;
    Ok(())
}
