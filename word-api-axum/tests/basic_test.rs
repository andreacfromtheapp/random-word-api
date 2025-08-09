//! Basic integration test to verify test infrastructure
//!
//! This is a simple test to ensure the test setup is working correctly
//! before running more complex integration tests.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::create_test_server;

#[tokio::test]
async fn test_basic_health_check() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/health/alive").await;

    // Basic assertions
    assert!(
        response.status_code() == StatusCode::OK,
        "Health endpoint should return 200, got: {}",
        response.status_code()
    );

    let body = response.text();
    assert!(!body.is_empty(), "Health response should not be empty");

    // Health endpoint returns plain text
    assert!(
        body.contains("API is successfully running"),
        "Health response should indicate API is running"
    );

    Ok(())
}

#[tokio::test]
async fn test_basic_database_health() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/health/ready").await;

    assert!(
        response.status_code() == StatusCode::OK,
        "Database health endpoint should return 200, got: {}",
        response.status_code()
    );

    let body = response.text();
    assert!(
        !body.is_empty(),
        "Database health response should not be empty"
    );

    // Database health endpoint returns plain text about connection
    assert!(
        body.contains("database"),
        "Database health response should mention database"
    );

    Ok(())
}

#[tokio::test]
async fn test_basic_random_word() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/en/word").await;

    // The response might be 200 (if there are words) or 404/500 (if database is empty)
    // Both are acceptable for this basic test
    assert!(
        response.status_code() >= StatusCode::OK
            && response.status_code() < StatusCode::IM_A_TEAPOT,
        "Random word endpoint should return valid HTTP status, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_basic_admin_endpoint() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/admin/en/words").await;

    // Should return 200 (empty list) or some other valid status
    assert!(
        response.status_code() >= StatusCode::OK
            && response.status_code() < StatusCode::IM_A_TEAPOT,
        "Admin words endpoint should return valid HTTP status, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_endpoint() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/invalid/endpoint/path").await;

    assert!(
        response.status_code() == StatusCode::NOT_FOUND,
        "Invalid endpoint should return 404, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_server_creation_multiple_times() -> Result<()> {
    // Test that we can create multiple test servers without conflicts
    for i in 0..3 {
        let (server, _temp_file) = create_test_server().await?;

        let response = server.get("/health/alive").await;
        assert!(
            response.status_code() == StatusCode::OK,
            "Health check {} should work, got: {}",
            i,
            response.status_code()
        );
    }

    Ok(())
}
