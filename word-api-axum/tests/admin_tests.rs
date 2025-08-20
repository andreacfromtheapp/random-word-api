//! Admin API contract tests
//!
//! Tests HTTP admin endpoints without database dependencies:
//! - Admin CRUD operations return correct HTTP status codes
//! - Request/response JSON structure validation
//! - Authorization and permission checking
//! - Error handling for invalid requests
//!
//! Philosophy: Test HTTP behavior, not database operations

use axum::http::{HeaderName, StatusCode};
use serde_json::{json, Value};

mod common;
use common::{create_mock_server, create_mock_server_with_data, mock_admin_token, mock_user_token};

/// Test admin create word endpoint returns correct structure
#[tokio::test]
async fn test_admin_create_word_returns_array() {
    let server = create_mock_server().await;
    let admin_token = mock_admin_token();

    let word_data = json!({
        "word": "testword1",
        "definition": "test definition 1",
        "pronunciation": "/test1/",
        "wordType": "noun"
    });

    let response = server
        .post("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json.is_array(), "Admin create should return array");

    if let Some(words) = json.as_array() {
        assert!(!words.is_empty(), "Array should contain the created word");

        let word = &words[0];
        assert!(word["id"].is_number());
        assert!(word["word"].is_string());
        assert!(word["definition"].is_string());
        assert!(word["pronunciation"].is_string());
        assert!(word["wordType"].is_string());
    }
}

/// Test admin create word requires authentication
#[tokio::test]
async fn test_admin_create_word_requires_auth() {
    let server = create_mock_server().await;

    let word_data = json!({
        "word": "testword1",
        "definition": "test definition 1",
        "pronunciation": "/test1/",
        "wordType": "noun"
    });

    let response = server.post("/admin/en/words").json(&word_data).await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin create word requires admin privileges
#[tokio::test]
async fn test_admin_create_word_requires_admin_privileges() {
    let server = create_mock_server().await;
    let user_token = mock_user_token();

    let word_data = json!({
        "word": "testword1",
        "definition": "test definition 1",
        "pronunciation": "/test1/",
        "wordType": "noun"
    });

    let response = server
        .post("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", user_token),
        )
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test admin list words endpoint returns array
#[tokio::test]
async fn test_admin_list_words_returns_array() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    let response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json.is_array(), "Admin list should return array");

    if let Some(words) = json.as_array() {
        for word in words {
            assert!(word["id"].is_number());
            assert!(word["word"].is_string());
            assert!(word["definition"].is_string());
            assert!(word["pronunciation"].is_string());
            assert!(word["wordType"].is_string());
        }
    }
}

/// Test admin list words requires authentication
#[tokio::test]
async fn test_admin_list_words_requires_auth() {
    let server = create_mock_server().await;

    let response = server.get("/admin/en/words").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin list words requires admin privileges
#[tokio::test]
async fn test_admin_list_words_requires_admin_privileges() {
    let server = create_mock_server().await;
    let user_token = mock_user_token();

    let response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", user_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test admin update word endpoint returns success message
#[tokio::test]
async fn test_admin_update_word_returns_success() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    let update_data = json!({
        "word": "updated",
        "definition": "updated definition",
        "pronunciation": "/ʌpdeɪtɪd/",
        "wordType": "verb"
    });

    let response = server
        .put("/admin/en/words/1")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .json(&update_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["message"].is_string());
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("updated successfully"));
}

/// Test admin update word returns 404 for non-existent word
#[tokio::test]
async fn test_admin_update_word_returns_404_for_non_existent() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    let update_data = json!({
        "word": "updated",
        "definition": "updated definition"
    });

    let response = server
        .put("/admin/en/words/999")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .json(&update_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test admin update word requires authentication
#[tokio::test]
async fn test_admin_update_word_requires_auth() {
    let server = create_mock_server().await;

    let update_data = json!({
        "word": "updated",
        "definition": "updated definition"
    });

    let response = server.put("/admin/en/words/1").json(&update_data).await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin delete word returns 204 for successful deletion
#[tokio::test]
async fn test_admin_delete_word_returns_204() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    let response = server
        .delete("/admin/en/words/1")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

/// Test admin delete word returns 404 for non-existent word
#[tokio::test]
async fn test_admin_delete_word_returns_404_for_non_existent() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    let response = server
        .delete("/admin/en/words/999")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test admin delete word requires authentication
#[tokio::test]
async fn test_admin_delete_word_requires_auth() {
    let server = create_mock_server().await;

    let response = server.delete("/admin/en/words/1").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin delete word requires admin privileges
#[tokio::test]
async fn test_admin_delete_word_requires_admin_privileges() {
    let server = create_mock_server().await;
    let user_token = mock_user_token();

    let response = server
        .delete("/admin/en/words/1")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", user_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test admin endpoints validate JSON request structure
#[tokio::test]
async fn test_admin_create_validates_json_structure() {
    let server = create_mock_server().await;
    let admin_token = mock_admin_token();

    // Test with complete valid JSON
    let valid_data = json!({
        "word": "test",
        "definition": "a test word",
        "pronunciation": "/tɛst/",
        "wordType": "noun"
    });

    let response = server
        .post("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .json(&valid_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test admin endpoints handle invalid authorization header formats
#[tokio::test]
async fn test_admin_endpoints_handle_invalid_auth_headers() {
    let server = create_mock_server().await;

    let word_data = json!({
        "word": "testword1",
        "definition": "test definition 1",
        "pronunciation": "/test1/",
        "wordType": "noun"
    });

    // Test with malformed Bearer token
    let response = server
        .post("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "InvalidFormat")
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

    // Test with empty Bearer token
    let response = server
        .post("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "Bearer ")
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

    // Test with non-Bearer authorization
    let response = server
        .post("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "Basic sometoken")
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test admin endpoints return consistent JSON structures
#[tokio::test]
async fn test_admin_endpoints_return_consistent_json() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    // Test list endpoint structure
    let response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let json: Value = response.json();
    assert!(json.is_array());

    // Test create endpoint structure
    let word_data = json!({
        "word": "consistency",
        "definition": "test definition consistency",
        "pronunciation": "/kənˈsɪstənsi/",
        "wordType": "noun"
    });
    let response = server
        .post("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", admin_token),
        )
        .json(&word_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let json: Value = response.json();
    assert!(json.is_array());
}

/// Test admin endpoints handle multiple sequential requests
#[tokio::test]
async fn test_admin_endpoints_handle_multiple_requests() {
    let server = create_mock_server_with_data().await;
    let admin_token = mock_admin_token();

    // Make multiple sequential list requests
    for _ in 0..3 {
        let response = server
            .get("/admin/en/words")
            .add_header(
                HeaderName::from_static("authorization"),
                format!("Bearer {}", admin_token),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let json: Value = response.json();
        assert!(json.is_array());
    }
}
