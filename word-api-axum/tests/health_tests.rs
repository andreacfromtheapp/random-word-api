//! Comprehensive health check and basic endpoint integration tests
//!
//! This module contains integration tests for health check endpoints and basic
//! API functionality, combining health checks with fundamental endpoint validation
//! to ensure the test infrastructure and core API endpoints are working correctly.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::create_test_server_streamlined;

#[tokio::test]
async fn test_health_endpoints_parallel() -> Result<()> {
    // Run multiple health checks in parallel for efficiency
    let (basic_result, content_result, db_result, db_detailed_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/alive").await;
            assert_eq!(
                response.status_code(),
                StatusCode::OK,
                "Health endpoint should return 200, got: {}",
                response.status_code()
            );
            let body = response.text();
            assert!(!body.is_empty(), "Health response should not be empty");
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/alive").await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let body = response.text();
            assert!(
                body.contains("API is successfully running"),
                "Health response should indicate API is running"
            );
            let content_type = response
                .headers()
                .get("content-type")
                .expect("Response should have content-type header");
            assert!(
                content_type.to_str().unwrap().contains("text/plain"),
                "Health response should be plain text content type"
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/ready").await;
            assert_eq!(
                response.status_code(),
                StatusCode::OK,
                "Database health endpoint should return 200, got: {}",
                response.status_code()
            );
            let body = response.text();
            assert!(
                body.contains("connection to the database"),
                "DB health response should mention database connection"
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/ready").await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let body = response.text();
            assert!(
                body.contains("database"),
                "DB health should mention database"
            );
            assert!(!body.is_empty(), "DB health response should not be empty");
            Ok::<(), anyhow::Error>(())
        }
    );

    // Check all parallel operations succeeded
    basic_result?;
    content_result?;
    db_result?;
    db_detailed_result?;

    Ok(())
}

#[tokio::test]
async fn test_health_multiple_requests() -> Result<()> {
    let server = create_test_server_streamlined().await?;

    // Test multiple sequential health check requests for consistency
    for i in 0..3 {
        let response = server.get("/health/alive").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Health check {i} should return 200"
        );

        let body = response.text();
        assert!(
            body.contains("API is successfully running"),
            "Health response {i} should indicate API is running"
        );
    }

    Ok(())
}

// === Basic endpoint tests merged from basic_test.rs ===

#[tokio::test]
async fn test_basic_endpoints() -> Result<()> {
    let server = create_test_server_streamlined().await?;

    // Test health endpoint
    let health_response = server.get("/health/alive").await;
    assert_eq!(health_response.status_code(), StatusCode::OK);
    let health_body = health_response.text();
    assert!(health_body.contains("API is successfully running"));

    // Test database health endpoint
    let db_health_response = server.get("/health/ready").await;
    assert_eq!(db_health_response.status_code(), StatusCode::OK);
    let db_body = db_health_response.text();
    assert!(db_body.contains("database"));

    // Test random word endpoint (may be empty database)
    let word_response = server.get("/en/word").await;
    assert!(
        word_response.status_code() >= StatusCode::OK
            && word_response.status_code() < StatusCode::IM_A_TEAPOT
    );

    // Test admin endpoint
    let admin_response = server.get("/admin/en/words").await;
    assert!(
        admin_response.status_code() >= StatusCode::OK
            && admin_response.status_code() < StatusCode::IM_A_TEAPOT
    );

    // Test invalid endpoint
    let invalid_response = server.get("/invalid/endpoint/path").await;
    assert_eq!(invalid_response.status_code(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn test_server_creation_multiple_times() -> Result<()> {
    // Test that we can create multiple test servers without conflicts (reduced iterations)
    for i in 0..2 {
        let server = create_test_server_streamlined().await?;

        let response = server.get("/health/alive").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Health check {i} should work"
        );
    }

    Ok(())
}
