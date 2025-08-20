//! Word API contract tests
//!
//! Tests HTTP word endpoints without database dependencies:
//! - Random word endpoints return correct JSON structure
//! - Word type filtering works correctly
//! - Invalid requests return proper HTTP status codes
//! - Response format consistency across all endpoints
//!
//! Philosophy: Test JSON responses, not database queries

use axum::http::StatusCode;
use serde_json::Value;

mod common;
use common::{create_mock_server, create_mock_server_with_data};

/// Test random word endpoint returns correct JSON structure
#[tokio::test]
async fn test_random_word_returns_json_structure() {
    let server = create_mock_server_with_data().await;

    let response = server.get("/en/words/random").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["id"].is_number());
    assert!(json["word"].is_string());
    assert!(json["definition"].is_string());
    assert!(json["pronunciation"].is_string());
    assert!(json["wordType"].is_string());

    // Verify content is not empty
    assert!(!json["word"].as_str().unwrap().is_empty());
    assert!(!json["definition"].as_str().unwrap().is_empty());
    assert!(!json["pronunciation"].as_str().unwrap().is_empty());
    assert!(!json["wordType"].as_str().unwrap().is_empty());
}

/// Test random word endpoint returns 404 when no words available
#[tokio::test]
async fn test_random_word_returns_404_when_empty() {
    let server = create_mock_server().await; // Empty server

    let response = server.get("/en/words/random").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test random word by type endpoint returns correct JSON structure
#[tokio::test]
async fn test_random_word_by_type_returns_json_structure() {
    let server = create_mock_server_with_data().await;

    let response = server.get("/en/words/random/noun").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["id"].is_number());
    assert!(json["word"].is_string());
    assert!(json["definition"].is_string());
    assert!(json["pronunciation"].is_string());
    assert_eq!(json["wordType"], "noun");
}

/// Test random word by type endpoint filters correctly
#[tokio::test]
async fn test_random_word_by_type_filters_correctly() {
    let server = create_mock_server_with_data().await;

    // Test verb filtering
    let response = server.get("/en/words/random/verb").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert_eq!(json["wordType"], "verb");
    assert_eq!(json["word"], "run"); // Our mock verb
}

/// Test random word by type returns 404 for non-existent type
#[tokio::test]
async fn test_random_word_by_type_returns_404_for_missing_type() {
    let server = create_mock_server_with_data().await;

    let response = server.get("/en/words/random/nonexistent").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test all supported word types return correct responses
#[tokio::test]
async fn test_all_word_types_supported() {
    let server = create_mock_server_with_data().await;

    let word_types = [
        "noun",
        "verb",
        "adjective",
        "adverb",
        "pronoun",
        "preposition",
        "conjunction",
        "interjection",
        "article",
    ];

    for word_type in word_types {
        let response = server.get(&format!("/en/words/random/{}", word_type)).await;

        // Should either return a word of that type (200) or not found (404)
        // Both are valid responses depending on available mock data
        assert!(
            response.status_code() == StatusCode::OK
                || response.status_code() == StatusCode::NOT_FOUND,
            "Word type '{}' should return 200 or 404, got {}",
            word_type,
            response.status_code()
        );

        if response.status_code() == StatusCode::OK {
            let json: Value = response.json();
            assert!(json["wordType"].is_string());
        }
    }
}

/// Test word endpoints handle invalid language codes gracefully
#[tokio::test]
async fn test_invalid_language_codes() {
    let server = create_mock_server_with_data().await;

    // These should return 404 since our mock only supports 'en'
    let invalid_languages = ["fr", "es", "de", "invalid"];

    for lang in invalid_languages {
        let response = server.get(&format!("/{}/words/random", lang)).await;

        // Mock server doesn't have these routes, so should be 404
        assert_eq!(
            response.status_code(),
            StatusCode::NOT_FOUND,
            "Invalid language '{}' should return 404",
            lang
        );
    }
}

/// Test word endpoint response headers
#[tokio::test]
async fn test_word_endpoint_response_headers() {
    let server = create_mock_server_with_data().await;

    let response = server.get("/en/words/random").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Check content type
    let content_type = response.headers().get("content-type");
    assert!(
        content_type.is_some(),
        "Response should have content-type header"
    );

    let content_type_str = content_type.unwrap().to_str().unwrap();
    assert!(
        content_type_str.contains("application/json"),
        "Content-type should be JSON, got: {}",
        content_type_str
    );
}

/// Test word endpoints handle malformed requests
#[tokio::test]
async fn test_malformed_word_requests() {
    let server = create_mock_server_with_data().await;

    // Test with extra path segments
    let response = server.get("/en/words/random/noun/extra").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    // Test with missing path segments
    let response = server.get("/en/words").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    // Test with invalid path
    let response = server.get("/en/words/invalid").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test word type case sensitivity
#[tokio::test]
async fn test_word_type_case_sensitivity() {
    let server = create_mock_server_with_data().await;

    // Test uppercase word type
    let response = server.get("/en/words/random/NOUN").await;
    // Should return 404 since our mock is case-sensitive
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    // Test mixed case
    let response = server.get("/en/words/random/Noun").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    // Test correct lowercase
    let response = server.get("/en/words/random/noun").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test multiple sequential word requests
#[tokio::test]
async fn test_multiple_word_requests() {
    let server = create_mock_server_with_data().await;

    // Make multiple sequential requests
    for _ in 0..3 {
        let response = server.get("/en/words/random").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: Value = response.json();
        assert!(json["word"].is_string());
        assert!(json["definition"].is_string());
        assert!(json["wordType"].is_string());
    }
}

/// Test word response consistency
#[tokio::test]
async fn test_word_response_consistency() {
    let server = create_mock_server_with_data().await;

    // Make multiple requests and verify consistent structure
    for _ in 0..3 {
        let response = server.get("/en/words/random").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: Value = response.json();

        // Verify all required fields are present
        assert!(json.get("id").is_some());
        assert!(json.get("word").is_some());
        assert!(json.get("definition").is_some());
        assert!(json.get("pronunciation").is_some());
        assert!(json.get("wordType").is_some());

        // Verify no unexpected fields
        let expected_fields = ["id", "word", "definition", "pronunciation", "wordType"];
        if let Some(obj) = json.as_object() {
            for key in obj.keys() {
                assert!(
                    expected_fields.contains(&key.as_str()),
                    "Unexpected field in response: {}",
                    key
                );
            }
        }
    }
}
