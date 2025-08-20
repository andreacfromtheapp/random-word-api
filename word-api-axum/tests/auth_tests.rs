//! Authentication API contract tests
//!
//! Tests HTTP authentication behavior without database dependencies:
//! - Login endpoint returns JWT tokens
//! - Invalid credentials return proper HTTP status codes
//! - Protected endpoints require authorization headers
//! - JWT token format validation
//!
//! Philosophy: Test auth behavior, not password hashing or database storage

use axum::http::{HeaderName, StatusCode};
use serde_json::{json, Value};

mod common;
use common::{create_mock_server, create_mock_server_with_data, mock_admin_token, mock_user_token};

/// Test login endpoint returns JWT for valid admin credentials
#[tokio::test]
async fn test_login_returns_jwt_for_valid_admin() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "password"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["token"].is_string());
    assert_eq!(json["user"]["username"], "admin");
    assert_eq!(json["user"]["isAdmin"], true);

    let token = json["token"].as_str().unwrap();
    assert!(token.contains("admin"));
}

/// Test login endpoint returns JWT for valid user credentials
#[tokio::test]
async fn test_login_returns_jwt_for_valid_user() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "user",
            "password": "password"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: Value = response.json();
    assert!(json["token"].is_string());
    assert_eq!(json["user"]["username"], "user");
    assert_eq!(json["user"]["isAdmin"], false);
}

/// Test login endpoint returns 401 for invalid credentials
#[tokio::test]
async fn test_login_returns_401_for_invalid_credentials() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "invalid",
            "password": "wrongpassword"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test login endpoint returns 401 for missing username
#[tokio::test]
async fn test_login_returns_401_for_missing_username() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "password": "password"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test login endpoint returns 401 for missing password
#[tokio::test]
async fn test_login_returns_401_for_missing_password() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "admin"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin endpoints require authorization header
#[tokio::test]
async fn test_admin_endpoints_require_authorization() {
    let server = create_mock_server().await;

    // Test admin create word without auth
    let response = server
        .post("/admin/en/words")
        .json(&json!({
            "word": "test",
            "definition": "test definition",
            "wordType": "noun"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test admin list words without auth
    let response = server.get("/admin/en/words").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test admin update word without auth
    let response = server
        .put("/admin/en/words/1")
        .json(&json!({
            "word": "updated",
            "definition": "updated definition"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test admin delete word without auth
    let response = server.delete("/admin/en/words/1").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

/// Test admin endpoints reject non-admin tokens
#[tokio::test]
async fn test_admin_endpoints_reject_non_admin_tokens() {
    let server = create_mock_server().await;
    let user_token = mock_user_token();

    let response = server
        .post("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", user_token),
        )
        .json(&json!({
            "word": "test",
            "definition": "test definition",
            "wordType": "noun"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test admin endpoints accept valid admin tokens
#[tokio::test]
async fn test_admin_endpoints_accept_admin_tokens() {
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
    assert!(json.is_array());
}

/// Test authorization header format validation
#[tokio::test]
async fn test_authorization_header_format_validation() {
    let server = create_mock_server().await;

    // Test invalid Bearer format
    let response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            "InvalidFormat token",
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

    // Test missing Bearer prefix
    let response = server
        .get("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "just.a.token")
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test JWT token contains expected format
#[tokio::test]
async fn test_jwt_token_format() {
    let server = create_mock_server().await;

    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "password"
        }))
        .await;

    let json: Value = response.json();
    let token = json["token"].as_str().unwrap();

    // Mock token should have expected format
    assert!(token.contains("."));
    assert!(token.len() > 10);
}

/// Test multiple login attempts with different credentials
#[tokio::test]
async fn test_multiple_login_attempts() {
    let server = create_mock_server().await;

    // Valid admin login
    let response1 = server
        .post("/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "password"
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::OK);

    // Valid user login
    let response2 = server
        .post("/auth/login")
        .json(&json!({
            "username": "user",
            "password": "password"
        }))
        .await;
    assert_eq!(response2.status_code(), StatusCode::OK);

    // Invalid login
    let response3 = server
        .post("/auth/login")
        .json(&json!({
            "username": "invalid",
            "password": "wrong"
        }))
        .await;
    assert_eq!(response3.status_code(), StatusCode::UNAUTHORIZED);
}
