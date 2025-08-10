//! Simplified health check integration tests
//!
//! This module contains basic integration tests for the health check endpoints,
//! verifying that the API can properly report its health status and database
//! connectivity without complex concurrent testing that causes lifetime issues.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::create_test_server_memory;

#[tokio::test]
async fn test_health_endpoint_returns_200() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

    let response = server.get("/health/alive").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Health endpoint should return 200, got: {}",
        response.status_code()
    );

    let body = response.text();
    assert!(!body.is_empty(), "Health response should not be empty");

    Ok(())
}

#[tokio::test]
async fn test_health_endpoint_content() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

    let response = server.get("/health/alive").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.text();
    assert!(
        body.contains("API is successfully running"),
        "Health response should indicate API is running"
    );

    // Check content type in same test for efficiency
    let content_type = response
        .headers()
        .get("content-type")
        .expect("Response should have content-type header");

    assert!(
        content_type.to_str().unwrap().contains("text/plain"),
        "Health response should be plain text content type"
    );

    Ok(())
}

#[tokio::test]
async fn test_health_db_endpoint() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

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

    Ok(())
}

#[tokio::test]
async fn test_health_db_detailed() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

    let response = server.get("/health/ready").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.text();

    // Should indicate database connection status
    assert!(
        body.contains("database"),
        "DB health should mention database"
    );
    assert!(!body.is_empty(), "DB health response should not be empty");

    Ok(())
}

#[tokio::test]
async fn test_health_multiple_requests() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

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
