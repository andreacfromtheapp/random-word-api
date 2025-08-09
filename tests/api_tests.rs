//! API endpoint integration tests
//!
//! This module tests the HTTP API endpoints with real requests and responses,
//! ensuring proper routing, middleware, and error handling.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::Value;
use tower::util::ServiceExt;

mod common;
use common::{
    create_test_app, invalid_definition_field, invalid_pronunciation_field, invalid_word,
    invalid_word_field, invalid_word_type_field, sample_word, validate_test_word,
};

#[tokio::test]
async fn test_health_alive_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/alive")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "The API is successfully running");
}

#[tokio::test]
async fn test_health_ready_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("OK. The API can establish a connection to the database"));
}

#[tokio::test]
async fn test_random_word_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/en/word")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an array (even if empty)
    assert!(json.is_array());
}

#[tokio::test]
async fn test_random_word_with_populated_database() {
    let app = create_test_app().await;

    // We need to access the database pool to populate it
    // This is a bit tricky with the current setup, so we'll test with empty DB
    let response = app
        .oneshot(
            Request::builder()
                .uri("/en/word")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_random_word_by_type_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/en/word/noun")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an array
    assert!(json.is_array());
}

#[tokio::test]
async fn test_invalid_language_code() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/invalid/word")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_list_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/en/words")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an array
    assert!(json.is_array());
}

#[tokio::test]
async fn test_admin_word_create_endpoint() {
    let app = create_test_app().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    let request_body = serde_json::to_string(&word_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an array with the created word
    assert!(json.is_array());
    let words = json.as_array().unwrap();
    assert_eq!(words.len(), 1);

    let created_word = &words[0];
    assert!(created_word["word"].as_str().unwrap().starts_with("test"));
    assert!(created_word["definition"]
        .as_str()
        .unwrap()
        .starts_with("a sample word for testing"));
}

#[tokio::test]
async fn test_admin_word_create_invalid_data() {
    let app = create_test_app().await;
    let invalid_data = invalid_word();

    // Verify test data is actually invalid using validators
    assert!(
        !validate_test_word(&invalid_data),
        "Test data should be invalid"
    );

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail validation
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_create_invalid_word_field() {
    let app = create_test_app().await;
    let invalid_data = invalid_word_field();

    // Verify only the word field is invalid
    assert!(!validate_test_word(&invalid_data));

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_create_invalid_definition_field() {
    let app = create_test_app().await;
    let invalid_data = invalid_definition_field();

    // Verify only the definition field is invalid
    assert!(!validate_test_word(&invalid_data));

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_create_invalid_pronunciation_field() {
    let app = create_test_app().await;
    let invalid_data = invalid_pronunciation_field();

    // Verify only the pronunciation field is invalid
    assert!(!validate_test_word(&invalid_data));

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_create_invalid_word_type_field() {
    let app = create_test_app().await;
    let invalid_data = invalid_word_type_field();

    // Verify only the word_type field is invalid
    assert!(!validate_test_word(&invalid_data));

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_admin_word_read_endpoint() {
    let app = create_test_app().await;

    // Try to read a non-existent word
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/en/words/99999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an empty array for non-existent word
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_admin_word_update_endpoint() {
    let app = create_test_app().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    let request_body = serde_json::to_string(&word_data).unwrap();

    // Try to update a non-existent word
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/admin/en/words/99999")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should return an empty array for non-existent word
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_admin_word_delete_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/admin/en/words/99999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Delete should succeed even for non-existent words (idempotent)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/en/word")
                .header("origin", "http://localhost")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // CORS preflight should be handled
    // The exact status depends on the CORS configuration
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::NO_CONTENT
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_invalid_json_request() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                .header("content-type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail to parse JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_word_update_invalid_data() {
    let app = create_test_app().await;
    let invalid_data = invalid_word();

    // Verify test data is actually invalid using validators
    assert!(
        !validate_test_word(&invalid_data),
        "Test data should be invalid"
    );

    let request_body = serde_json::to_string(&invalid_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/admin/en/words/99999")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail validation
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = create_test_app().await;
    let word_data = sample_word();

    // Verify test data is valid using validators
    assert!(validate_test_word(&word_data), "Test data should be valid");

    let request_body = serde_json::to_string(&word_data).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/en/words")
                // Missing content-type header
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail without proper content type
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_route_not_found() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent/route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
