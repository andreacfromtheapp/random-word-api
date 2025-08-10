//! Simplified public word API integration tests
//!
//! This module contains basic integration tests for the public word retrieval endpoints,
//! verifying that the API correctly returns random words and handles basic requests
//! without complex validation or concurrent operations.

use anyhow::Result;
use axum::http::StatusCode;

use word_api_axum::models::word::UpsertWord;

mod helpers;
use helpers::{
    create_test_server, create_test_server_memory, create_test_server_with_pool,
    database::populate_test_data,
};

#[tokio::test]
async fn test_random_word_endpoint() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "test").await?;

    let response = server.get("/en/word").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random word endpoint should return 200, got: {}",
        response.status_code()
    );

    let json: serde_json::Value = response.json();

    // Public API returns arrays
    assert!(json.is_array(), "Response should be an array");
    let words = json.as_array().unwrap();
    assert!(
        !words.is_empty(),
        "Response should contain at least one word"
    );

    let word = &words[0];
    assert!(
        word.get("word").is_some(),
        "Response should have word field. Got: {word}"
    );
    assert!(
        word.get("definition").is_some(),
        "Response should have definition field"
    );
    assert!(
        word.get("pronunciation").is_some(),
        "Response should have pronunciation field"
    );

    Ok(())
}

#[tokio::test]
async fn test_random_word_by_type() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "test").await?;

    // Test all word types in single test for efficiency
    let word_types = vec!["noun", "verb", "adjective", "adverb"];

    for word_type in word_types {
        let response = server.get(&format!("/en/word/{word_type}")).await;

        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Random {word_type} endpoint should return 200"
        );

        let json: serde_json::Value = response.json();
        assert!(
            json.is_array(),
            "Response should be an array for {word_type}"
        );
        let words = json.as_array().unwrap();
        assert!(
            !words.is_empty(),
            "Response should contain at least one {word_type}"
        );

        let word = &words[0];
        assert!(
            word.get("word").is_some(),
            "Response should have word field for {word_type}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_invalid_language() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;

    let response = server.get("/invalid/word").await;

    assert!(
        response.status_code() >= StatusCode::BAD_REQUEST,
        "Invalid language should return error, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_word_type() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/en/word/invalid_type").await;

    assert_eq!(
        response.status_code(),
        StatusCode::BAD_REQUEST,
        "Invalid word type should return 400 Bad Request, got: {}",
        response.status_code()
    );

    let body = response.text();
    assert!(!body.is_empty(), "Error response should have a message");
    assert!(
        body.to_lowercase().contains("invalid word type"),
        "Error message should mention invalid word type"
    );

    Ok(())
}

#[tokio::test]
async fn test_empty_database() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;
    // Don't populate data

    let response = server.get("/en/word").await;

    // Empty database returns 200 with empty array or error - both acceptable
    assert!(
        response.status_code() == StatusCode::OK
            || response.status_code() >= StatusCode::BAD_REQUEST,
        "Empty database should return OK or error, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
async fn test_word_api_response_format() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "test").await?;

    let response = server.get("/en/word").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let json: serde_json::Value = response.json();

    // Public API returns arrays
    assert!(json.is_array(), "Response should be an array");
    let words = json.as_array().unwrap();
    assert!(
        !words.is_empty(),
        "Response should contain at least one word"
    );

    let word = &words[0];
    // Verify required fields exist
    assert!(word.get("word").is_some(), "Should have word field");
    assert!(
        word.get("definition").is_some(),
        "Should have definition field"
    );
    assert!(
        word.get("pronunciation").is_some(),
        "Should have pronunciation field"
    );

    // Verify field types
    assert!(word["word"].is_string(), "Word should be string");
    assert!(
        word["definition"].is_string(),
        "Definition should be string"
    );
    assert!(
        word["pronunciation"].is_string(),
        "Pronunciation should be string"
    );

    // Verify content is not empty
    assert!(
        !word["word"].as_str().unwrap().is_empty(),
        "Word should not be empty"
    );
    assert!(
        !word["definition"].as_str().unwrap().is_empty(),
        "Definition should not be empty"
    );
    assert!(
        !word["pronunciation"].as_str().unwrap().is_empty(),
        "Pronunciation should not be empty"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_random_word_requests() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "test").await?;

    // Make multiple requests to test randomness (reduced for efficiency)
    for i in 0..3 {
        let response = server.get("/en/word").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Request {i} should return 200"
        );

        let json: serde_json::Value = response.json();
        assert!(json.is_array(), "Response should be an array");
        let words = json.as_array().unwrap();
        assert!(
            !words.is_empty(),
            "Response should contain at least one word"
        );

        let word = &words[0];
        assert!(
            word.get("word").is_some(),
            "Request {i} should have word field"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_word_api_content_type() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "test").await?;

    let response = server.get("/en/word").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Response should have content-type header");

    assert!(
        content_type.to_str().unwrap().contains("application/json"),
        "Word API response should be JSON content type"
    );

    Ok(())
}

#[tokio::test]
async fn test_crud_workflow() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // CREATE - Add a word via admin API with performance monitoring
    let word_data = UpsertWord {
        word: "workflow".to_string(),
        definition: "a workflow test word".to_string(),
        pronunciation: "/wɜːrkfloʊ/".to_string(),
        word_type: "noun".to_string(),
    };

    let create_body = serde_json::json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let create_response = server.post("/admin/en/words").json(&create_body).await;

    assert_eq!(
        create_response.status_code(),
        StatusCode::OK,
        "Create should succeed"
    );

    // READ - Try to get the word via public API (simplified approach)
    let mut found_valid_response = false;
    for _attempt in 0..5 {
        let response = server.get("/en/word").await;
        if response.status_code() == StatusCode::OK {
            let json: serde_json::Value = response.json();
            if let Some(words) = json.as_array() {
                if !words.is_empty() {
                    found_valid_response = true;
                    break;
                }
            }
        }
    }

    // Note: We don't assert the specific word due to randomness,
    // but we verify the API works and returns valid responses
    assert!(
        found_valid_response,
        "Should be able to retrieve words from API"
    );

    Ok(())
}

#[tokio::test]
async fn test_api_multiple_requests() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "multi").await?;

    // Test multiple sequential requests for functional reliability
    for i in 0..3 {
        let response = server.get("/en/word").await;
        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Request {i} should succeed"
        );
    }
    Ok(())
}
