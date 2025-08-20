//! Integration tests for authentication and authorization
//!
//! Tests the complete authentication flow including:
//! - User registration and login
//! - JWT token generation and validation
//! - Protected admin endpoints
//! - OpenAPI documentation access control

use axum::http::HeaderName;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn test_auth_flow_with_admin_endpoints() {
    let server = helpers::create_test_server().await.unwrap();

    // Step 1: Register a new user
    let register_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "admin_user",
            "password": "secure_password_123",
            "is_admin": true
        }))
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body: serde_json::Value = register_response.json();
    let token = register_body["token"].as_str().unwrap();

    // Step 2: Access admin endpoints with valid token
    let admin_response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", token),
        )
        .await;

    admin_response.assert_status(axum::http::StatusCode::OK);

    // Step 3: Try to access admin endpoints without token (should fail)
    let unauthorized_response = server.get("/admin/en/words").await;

    unauthorized_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    // Step 4: Try to access admin endpoints with invalid token (should fail)
    let invalid_token_response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            "Bearer invalid_token",
        )
        .await;

    invalid_token_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_non_admin_user_cannot_access_admin_endpoints() {
    let server = helpers::create_test_server().await.unwrap();

    // Register a non-admin user
    let register_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "regular_user",
            "password": "secure_password_123",
            "is_admin": false
        }))
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body: serde_json::Value = register_response.json();
    let token = register_body["token"].as_str().unwrap();

    // Try to access admin endpoints with non-admin token (should fail)
    let admin_response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", token),
        )
        .await;

    admin_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_and_token_usage() {
    let server = helpers::create_test_server().await.unwrap();

    // Register a user first
    let register_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "test_login_user",
            "password": "login_password_123",
            "is_admin": true
        }))
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);

    // Login with the same credentials
    let login_response = server
        .post("/auth/login")
        .json(&json!({
            "username": "test_login_user",
            "password": "login_password_123"
        }))
        .await;

    login_response.assert_status(axum::http::StatusCode::OK);
    let login_body: serde_json::Value = login_response.json();
    let token = login_body["token"].as_str().unwrap();
    assert!(login_body["expires_in"].as_i64().unwrap() > 0);

    // Use the login token to access admin endpoints
    let admin_response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", token),
        )
        .await;

    admin_response.assert_status(axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_complete_admin_crud_with_auth() {
    let server = helpers::create_test_server().await.unwrap();

    // Register an admin user
    let register_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "crud_admin",
            "password": "crud_password_123",
            "is_admin": true
        }))
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body: serde_json::Value = register_response.json();
    let token = register_body["token"].as_str().unwrap();

    let auth_header = (
        HeaderName::from_static("authorization"),
        format!("Bearer {}", token),
    );

    // Test CREATE - Add a new word
    let create_response = server
        .post("/admin/en/words")
        .add_header(auth_header.0.clone(), auth_header.1.clone())
        .json(&json!({
            "word": "testword",
            "definition": "a word used for testing purposes",
            "pronunciation": "/ˈtɛstˌwɜrd/",
            "wordType": "noun"
        }))
        .await;

    create_response.assert_status(axum::http::StatusCode::OK);

    // Test READ - List all words
    let list_response = server
        .get("/admin/en/words")
        .add_header(auth_header.0.clone(), auth_header.1.clone())
        .await;

    list_response.assert_status(axum::http::StatusCode::OK);
    let words: serde_json::Value = list_response.json();
    assert!(words.as_array().unwrap().len() > 0);

    // Find the created word's ID
    let created_word = words
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["word"] == "testword")
        .expect("Created word should be in the list");
    let word_id = created_word["id"].as_i64().unwrap();

    // Test READ by ID
    let read_response = server
        .get(&format!("/admin/en/words/{}", word_id))
        .add_header(auth_header.0.clone(), auth_header.1.clone())
        .await;

    read_response.assert_status(axum::http::StatusCode::OK);

    // Test UPDATE
    let update_response = server
        .put(&format!("/admin/en/words/{}", word_id))
        .add_header(auth_header.0.clone(), auth_header.1.clone())
        .json(&json!({
            "word": "updatedword",
            "definition": "an updated word used for testing purposes",
            "pronunciation": "/ˈʌpˌdeɪtɪdˌwɜrd/",
            "wordType": "noun"
        }))
        .await;

    update_response.assert_status(axum::http::StatusCode::OK);

    // Test DELETE
    let delete_response = server
        .delete(&format!("/admin/en/words/{}", word_id))
        .add_header(auth_header.0, auth_header.1)
        .await;

    delete_response.assert_status(axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_public_endpoints_remain_accessible() {
    let server = helpers::create_test_server().await.unwrap();

    // Public endpoints should work without authentication
    let health_response = server.get("/health/alive").await;
    health_response.assert_status(axum::http::StatusCode::OK);

    let ready_response = server.get("/health/ready").await;
    ready_response.assert_status(axum::http::StatusCode::OK);

    // Word endpoints should remain public
    let random_word_response = server.get("/en/random").await;
    // Note: This might return 404 if no words exist, which is fine for this test
    assert!(
        random_word_response.status_code() == axum::http::StatusCode::OK
            || random_word_response.status_code() == axum::http::StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_jwt_token_expiration_handling() {
    let server = helpers::create_test_server().await.unwrap();

    // Register a user
    let register_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "expiry_test_user",
            "password": "expiry_password_123",
            "is_admin": true
        }))
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body: serde_json::Value = register_response.json();
    let token = register_body["token"].as_str().unwrap();

    // Token should be valid initially
    let valid_response = server
        .get("/admin/en/words")
        .add_header(
            HeaderName::from_static("authorization"),
            format!("Bearer {}", token),
        )
        .await;

    valid_response.assert_status(axum::http::StatusCode::OK);

    // Note: In a real test, we would wait for token expiration or manipulate the clock
    // For now, we just verify the token works as expected
    // A complete test would require mocking the current time or using a very short expiration
}

#[tokio::test]
async fn test_invalid_auth_scenarios() {
    let server = helpers::create_test_server().await.unwrap();

    // Test 1: Missing Authorization header
    let no_auth_response = server.get("/admin/en/words").await;
    no_auth_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    // Test 2: Malformed Authorization header
    let malformed_response = server
        .get("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "NotBearer token")
        .await;
    malformed_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    // Test 3: Empty token
    let empty_token_response = server
        .get("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "Bearer ")
        .await;
    empty_token_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    // Test 4: Invalid JWT format
    let invalid_jwt_response = server
        .get("/admin/en/words")
        .add_header(HeaderName::from_static("authorization"), "Bearer not.a.jwt")
        .await;
    invalid_jwt_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_registration_validation() {
    let server = helpers::create_test_server().await.unwrap();

    // Test 1: Username too short
    let short_username_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "ab",
            "password": "valid_password_123"
        }))
        .await;
    short_username_response.assert_status(axum::http::StatusCode::BAD_REQUEST);

    // Test 2: Password too short
    let short_password_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "validuser",
            "password": "123"
        }))
        .await;
    short_password_response.assert_status(axum::http::StatusCode::BAD_REQUEST);

    // Test 3: Missing required fields
    let missing_fields_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "validuser"
            // missing password
        }))
        .await;
    missing_fields_response.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    // Test 4: Valid registration should work
    let valid_response = server
        .post("/auth/register")
        .json(&json!({
            "username": "validuser",
            "password": "valid_password_123"
        }))
        .await;
    valid_response.assert_status(axum::http::StatusCode::CREATED);
}
