//! Simple mock framework for HTTP API contract testing
//!
//! Provides lightweight test utilities focused on HTTP behavior:
//! - Mock servers without database dependencies
//! - Pre-defined test responses for predictable testing
//! - JWT token generation for auth testing
//! - Static test data for consistent results
//!
//! Philosophy: Test the API contract, not the database implementation

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use axum_test::TestServer;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple in-memory state for mock testing
#[derive(Clone, Default)]
pub struct MockState {
    pub words: Arc<Mutex<HashMap<String, MockWord>>>,
    pub users: Arc<Mutex<HashMap<String, MockUser>>>,
}

/// Mock word data structure
#[derive(Clone, Debug)]
pub struct MockWord {
    pub id: u32,
    pub word: String,
    pub definition: String,
    pub pronunciation: String,
    pub word_type: String,
}

/// Mock user data structure
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MockUser {
    pub username: String,
    pub is_admin: bool,
}

/// Create a mock server with no data (for testing empty states)
pub async fn create_mock_server() -> TestServer {
    let state = MockState::default();
    let router = create_mock_router(state);
    TestServer::new(router).unwrap()
}

/// Create a mock server pre-populated with test data
pub async fn create_mock_server_with_data() -> TestServer {
    let state = MockState::default();

    // Add some test words
    let mut words = state.words.lock().unwrap();
    words.insert(
        "test1".to_string(),
        MockWord {
            id: 1,
            word: "test".to_string(),
            definition: "a test word".to_string(),
            pronunciation: "/tɛst/".to_string(),
            word_type: "noun".to_string(),
        },
    );
    words.insert(
        "run2".to_string(),
        MockWord {
            id: 2,
            word: "run".to_string(),
            definition: "to move quickly".to_string(),
            pronunciation: "/rʌn/".to_string(),
            word_type: "verb".to_string(),
        },
    );
    drop(words);

    // Add test admin user
    let mut users = state.users.lock().unwrap();
    users.insert(
        "admin".to_string(),
        MockUser {
            username: "admin".to_string(),
            is_admin: true,
        },
    );
    drop(users);

    let router = create_mock_router(state);
    TestServer::new(router).unwrap()
}

/// Create the mock router with simplified endpoints
fn create_mock_router(state: MockState) -> Router {
    Router::new()
        // Health endpoint
        .route("/health", get(mock_health))
        // Auth endpoints
        .route("/auth/login", post(mock_login))
        // Word endpoints
        .route("/en/words/random", get(mock_random_word))
        .route(
            "/en/words/random/{word_type}",
            get(mock_random_word_by_type),
        )
        // Admin endpoints
        .route("/admin/en/words", post(mock_admin_create_word))
        .route("/admin/en/words", get(mock_admin_list_words))
        .route("/admin/en/words/{id}", put(mock_admin_update_word))
        .route("/admin/en/words/{id}", delete(mock_admin_delete_word))
        .with_state(state)
}

/// Mock health endpoint - always returns healthy
async fn mock_health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "database": "connected"
    }))
}

/// Mock login endpoint - returns JWT for valid credentials
async fn mock_login(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let username = payload.get("username").and_then(|v| v.as_str());
    let password = payload.get("password").and_then(|v| v.as_str());

    if username == Some("admin") && password == Some("password") {
        Ok(Json(json!({
            "token": "mock.jwt.token.for.admin",
            "user": {
                "username": "admin",
                "isAdmin": true
            }
        })))
    } else if username == Some("user") && password == Some("password") {
        Ok(Json(json!({
            "token": "mock.jwt.token.for.user",
            "user": {
                "username": "user",
                "isAdmin": false
            }
        })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Mock random word endpoint
async fn mock_random_word(State(state): State<MockState>) -> Result<Json<Value>, StatusCode> {
    let words = state.words.lock().unwrap();
    if let Some(word) = words.values().next() {
        Ok(Json(json!({
            "id": word.id,
            "word": word.word,
            "definition": word.definition,
            "pronunciation": word.pronunciation,
            "wordType": word.word_type
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Mock random word by type endpoint
async fn mock_random_word_by_type(
    axum::extract::Path(word_type): axum::extract::Path<String>,
    State(state): State<MockState>,
) -> Result<Json<Value>, StatusCode> {
    let words = state.words.lock().unwrap();
    if let Some(word) = words.values().find(|w| w.word_type == word_type) {
        Ok(Json(json!({
            "id": word.id,
            "word": word.word,
            "definition": word.definition,
            "pronunciation": word.pronunciation,
            "wordType": word.word_type
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Mock admin create word - requires auth header
async fn mock_admin_create_word(
    headers: axum::http::HeaderMap,
    State(state): State<MockState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Check for auth header
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth = headers.get("authorization").unwrap().to_str().unwrap();
    if !auth.starts_with("Bearer ") || !auth.contains("admin") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Extract word data
    let word = payload
        .get("word")
        .and_then(|v| v.as_str())
        .unwrap_or("newword");
    let definition = payload
        .get("definition")
        .and_then(|v| v.as_str())
        .unwrap_or("new definition");
    let pronunciation = payload
        .get("pronunciation")
        .and_then(|v| v.as_str())
        .unwrap_or("/new/");
    let word_type = payload
        .get("wordType")
        .and_then(|v| v.as_str())
        .unwrap_or("noun");

    // Add to mock state
    let mut words = state.words.lock().unwrap();
    let new_id = words.len() as u32 + 1;
    words.insert(
        format!("{}{}", word, new_id),
        MockWord {
            id: new_id,
            word: word.to_string(),
            definition: definition.to_string(),
            pronunciation: pronunciation.to_string(),
            word_type: word_type.to_string(),
        },
    );

    // Return all words as array (matching current API)
    let all_words: Vec<Value> = words
        .values()
        .map(|w| {
            json!({
                "id": w.id,
                "word": w.word,
                "definition": w.definition,
                "pronunciation": w.pronunciation,
                "wordType": w.word_type
            })
        })
        .collect();

    Ok(Json(Value::Array(all_words)))
}

/// Mock admin list words
async fn mock_admin_list_words(
    headers: axum::http::HeaderMap,
    State(state): State<MockState>,
) -> Result<Json<Value>, StatusCode> {
    // Check for auth header
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth = headers.get("authorization").unwrap().to_str().unwrap();
    if !auth.starts_with("Bearer ") || !auth.contains("admin") {
        return Err(StatusCode::FORBIDDEN);
    }

    let words = state.words.lock().unwrap();
    let all_words: Vec<Value> = words
        .values()
        .map(|w| {
            json!({
                "id": w.id,
                "word": w.word,
                "definition": w.definition,
                "pronunciation": w.pronunciation,
                "wordType": w.word_type
            })
        })
        .collect();

    Ok(Json(Value::Array(all_words)))
}

/// Mock admin update word
async fn mock_admin_update_word(
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<u32>,
    State(state): State<MockState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Check for auth header
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth = headers.get("authorization").unwrap().to_str().unwrap();
    if !auth.starts_with("Bearer ") || !auth.contains("admin") {
        return Err(StatusCode::FORBIDDEN);
    }

    let words = state.words.lock().unwrap();
    if words.values().any(|w| w.id == id) {
        Ok(Json(json!({"message": "Word updated successfully"})))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Mock admin delete word
async fn mock_admin_delete_word(
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<u32>,
    State(state): State<MockState>,
) -> StatusCode {
    // Check for auth header
    if !headers.contains_key("authorization") {
        return StatusCode::UNAUTHORIZED;
    }

    let auth = headers.get("authorization").unwrap().to_str().unwrap();
    if !auth.starts_with("Bearer ") || !auth.contains("admin") {
        return StatusCode::FORBIDDEN;
    }

    let words = state.words.lock().unwrap();
    if words.values().any(|w| w.id == id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Generate a mock JWT token for testing
#[allow(dead_code)]
pub fn mock_admin_token() -> String {
    "mock.jwt.token.for.admin".to_string()
}

/// Generate a mock JWT token for regular user
#[allow(dead_code)]
pub fn mock_user_token() -> String {
    "mock.jwt.token.for.user".to_string()
}
