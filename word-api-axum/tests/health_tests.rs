//! Health endpoint integration tests
//!
//! Tests health check endpoints including basic API health,
//! database connectivity, and error handling for invalid endpoints.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::create_test_server_streamlined;

#[tokio::test]
async fn test_health_endpoints_comprehensive() -> Result<()> {
    // Consolidated health endpoint testing - both /health/alive and /health/ready
    let (alive_result, ready_result) = tokio::join!(
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
            assert!(!body.is_empty(), "DB health response should not be empty");
            assert!(
                body.contains("database"),
                "DB health should mention database"
            );
            Ok::<(), anyhow::Error>(())
        }
    );

    // Check both parallel operations succeeded
    alive_result?;
    ready_result?;

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

#[tokio::test]
async fn test_invalid_endpoints() -> Result<()> {
    let server = create_test_server_streamlined().await?;

    // Test invalid endpoint (non-health endpoint testing for this module)
    let invalid_response = server.get("/invalid/endpoint/path").await;
    assert_eq!(invalid_response.status_code(), StatusCode::NOT_FOUND);

    Ok(())
}
