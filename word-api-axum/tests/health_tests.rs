//! Health endpoint tests
//!
//! Tests HTTP health check endpoint without dependencies:
//! - Health endpoint returns 200 status
//! - Health endpoint returns correct JSON structure
//! - Health endpoint is accessible without authentication
//!
//! Philosophy: Test endpoint availability, not underlying health checks

use axum::http::StatusCode;
use serde_json::Value;

mod common;
use common::{create_mock_server, create_mock_server_with_data};

/// Test health endpoint returns 200 status
#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let server = create_mock_server().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test health endpoint returns correct JSON structure
#[tokio::test]
async fn test_health_endpoint_returns_json_structure() {
    let server = create_mock_server().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["status"].is_string());
    assert_eq!(json["status"], "healthy");

    // Should include database status
    assert!(json["database"].is_string());
}

/// Test health endpoint works with populated data
#[tokio::test]
async fn test_health_endpoint_works_with_data() {
    let server = create_mock_server_with_data().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["database"], "connected");
}

/// Test health endpoint does not require authentication
#[tokio::test]
async fn test_health_endpoint_no_auth_required() {
    let server = create_mock_server().await;

    // No authorization header provided
    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert_eq!(json["status"], "healthy");
}

/// Test health endpoint returns JSON content type
#[tokio::test]
async fn test_health_endpoint_returns_json_content_type() {
    let server = create_mock_server().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());

    let content_type_str = content_type.unwrap().to_str().unwrap();
    assert!(content_type_str.contains("application/json"));
}

/// Test health endpoint handles multiple sequential requests
#[tokio::test]
async fn test_health_endpoint_multiple_requests() {
    let server = create_mock_server().await;

    // Make multiple sequential requests
    for _ in 0..3 {
        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: Value = response.json();
        assert_eq!(json["status"], "healthy");
    }
}

/// Test health endpoint response consistency
#[tokio::test]
async fn test_health_endpoint_response_consistency() {
    let server = create_mock_server().await;

    // Make multiple requests and verify consistent response
    for _ in 0..3 {
        let response = server.get("/health").await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let json: Value = response.json();

        // Verify required fields are present
        assert!(json.get("status").is_some());
        assert!(json.get("database").is_some());

        // Verify field values
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["database"], "connected");
    }
}
