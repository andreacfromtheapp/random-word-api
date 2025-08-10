//! Simplified admin API integration tests
//!
//! This module contains basic integration tests for the admin endpoints that handle
//! CRUD operations for words. These tests verify core functionality without
//! complex concurrent operations that cause lifetime issues.

use anyhow::Result;
use axum::http::StatusCode;
use serde_json::json;

use word_api_axum::models::word::UpsertWord;

mod helpers;
use helpers::{create_test_server, create_test_server_memory};

/// Create a test word
fn create_test_word(suffix: &str) -> UpsertWord {
    UpsertWord {
        word: format!("testword{suffix}"),
        definition: format!("test definition {suffix}"),
        pronunciation: "/tɛst/".to_string(),
        word_type: "noun".to_string(),
    }
}

#[tokio::test]
async fn test_admin_create_word_success() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let word_data = create_test_word("1");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let response = server.post("/admin/en/words").json(&body).await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Create word should return 200, got: {} - body: {}",
        response.status_code(),
        response.text()
    );

    let json: serde_json::Value = response.json();

    // Admin API returns arrays
    if let Some(words) = json.as_array() {
        assert!(!words.is_empty(), "Response should contain created word");
        let word = &words[0];
        assert_eq!(word["word"], word_data.word.to_lowercase());
        assert_eq!(word["definition"], word_data.definition.to_lowercase());
        assert_eq!(word["word_type"], word_data.word_type);
    } else {
        panic!("Expected array response from admin API, got: {json}");
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_get_all_words() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/admin/en/words").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Get all words should return 200, got: {}",
        response.status_code()
    );

    let json: serde_json::Value = response.json();
    assert!(
        json.is_array(),
        "Get all words should return array, got: {json}"
    );

    Ok(())
}

#[tokio::test]
async fn test_admin_create_and_retrieve() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Create a word
    let word_data = create_test_word("retrieve");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let create_response = server.post("/admin/en/words").json(&body).await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let words = create_json.as_array().unwrap();
    let created_word = &words[0];
    let id = created_word["id"].as_u64().unwrap() as u32;

    // Retrieve the word by ID
    let get_response = server.get(&format!("/admin/en/words/{id}")).await;
    assert_eq!(
        get_response.status_code(),
        StatusCode::OK,
        "Get word by ID should return 200"
    );

    let get_json: serde_json::Value = get_response.json();
    if let Some(words_array) = get_json.as_array() {
        assert!(!words_array.is_empty(), "Should return the requested word");
        let retrieved_word = &words_array[0];
        assert_eq!(retrieved_word["word"], word_data.word.to_lowercase());
        assert_eq!(
            retrieved_word["definition"],
            word_data.definition.to_lowercase()
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_create_word_validation() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test with empty word
    let invalid_body = json!({
        "word": "",
        "definition": "valid definition",
        "pronunciation": "/vælɪd/",
        "word_type": "noun"
    });

    let response = server.post("/admin/en/words").json(&invalid_body).await;

    assert!(
        response.status_code() >= StatusCode::BAD_REQUEST,
        "Empty word should return validation error, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_admin_update_word() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // First create a word
    let word_data = create_test_word("update");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let create_response = server.post("/admin/en/words").json(&body).await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let words = create_json.as_array().unwrap();
    let created_word = &words[0];
    let id = created_word["id"].as_u64().unwrap() as u32;

    // Update the word
    let updated_body = json!({
        "word": "updatedword",
        "definition": "updated definition",
        "pronunciation": "/ʌpdeɪtəd/",
        "word_type": "verb"
    });

    let update_response = server
        .put(&format!("/admin/en/words/{id}"))
        .json(&updated_body)
        .await;

    assert_eq!(
        update_response.status_code(),
        StatusCode::OK,
        "Update word should return 200"
    );

    let update_json: serde_json::Value = update_response.json();
    if let Some(words_array) = update_json.as_array() {
        assert!(!words_array.is_empty(), "Should return updated word");
        let updated_word = &words_array[0];
        assert_eq!(updated_word["word"], "updatedword");
        assert_eq!(updated_word["definition"], "updated definition");
        assert_eq!(updated_word["word_type"], "verb");
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_delete_word() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // First create a word
    let word_data = create_test_word("delete");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let create_response = server.post("/admin/en/words").json(&body).await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let words = create_json.as_array().unwrap();
    let created_word = &words[0];
    let id = created_word["id"].as_u64().unwrap() as u32;

    // Delete the word
    let delete_response = server.delete(&format!("/admin/en/words/{id}")).await;

    assert_eq!(
        delete_response.status_code(),
        StatusCode::OK,
        "Delete word should return 200"
    );

    // Verify the word is gone
    let get_response = server.get(&format!("/admin/en/words/{id}")).await;
    assert_eq!(
        get_response.status_code(),
        StatusCode::OK,
        "Get after delete should return 200"
    );

    let get_json: serde_json::Value = get_response.json();
    assert!(get_json.is_array(), "Response should be an array");
    assert!(
        get_json.as_array().unwrap().is_empty(),
        "Response should be empty array for deleted word"
    );

    Ok(())
}

#[tokio::test]
async fn test_admin_nonexistent_word_operations() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test GET nonexistent word
    let get_response = server.get("/admin/en/words/99999").await;
    assert_eq!(get_response.status_code(), StatusCode::OK);
    let json: serde_json::Value = get_response.json();
    assert!(json.is_array() && json.as_array().unwrap().is_empty());

    // Test UPDATE nonexistent word
    let update_body = json!({
        "word": "nonexistent",
        "definition": "definition",
        "pronunciation": "/nɑnɪɡzɪstənt/",
        "word_type": "noun"
    });
    let update_response = server.put("/admin/en/words/99999").json(&update_body).await;
    assert_eq!(
        update_response.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );

    // Test DELETE nonexistent word
    let delete_response = server.delete("/admin/en/words/99999").await;
    assert_eq!(delete_response.status_code(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn test_admin_request_validation() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

    // Test invalid JSON
    let invalid_json_response = server
        .post("/admin/en/words")
        .text("{ invalid json")
        .content_type("application/json")
        .await;
    assert!(invalid_json_response.status_code() >= StatusCode::BAD_REQUEST);

    // Test missing content type
    let valid_data = json!({
        "word": "test",
        "definition": "test definition",
        "pronunciation": "/tɛst/",
        "word_type": "noun"
    });
    let missing_content_type_response = server
        .post("/admin/en/words")
        .text(valid_data.to_string())
        .await;
    assert!(missing_content_type_response.status_code() >= StatusCode::BAD_REQUEST);

    Ok(())
}
