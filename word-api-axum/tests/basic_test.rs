//! Basic integration test to verify test infrastructure
//!
//! This is a simple test to ensure the test setup is working correctly
//! before running more complex integration tests.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::create_test_server_memory;

#[tokio::test]
async fn test_basic_endpoints() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

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
        let (server, _pool) = create_test_server_memory().await?;

        let response = server.get("/health/alive").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Health check {i} should work"
        );
    }

    Ok(())
}
