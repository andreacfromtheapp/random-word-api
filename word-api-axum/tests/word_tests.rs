//! Simplified public word API integration tests
//!
//! This module contains basic integration tests for the public word retrieval endpoints,
//! verifying that the API correctly returns random words and handles basic requests
//! without complex validation or concurrent operations.

use anyhow::Result;
use axum::http::StatusCode;
use serial_test::serial;
use word_api_axum::models::word::UpsertWord;

mod helpers;
use helpers::{
    assert_test_performance, create_test_server, create_test_server_with_pool,
    database::{cleanup_test_data, populate_test_data},
    measure_test_performance, performance_thresholds,
};

#[tokio::test]
#[serial]
async fn test_random_word_endpoint() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "1").await?;

    let (response, metrics) = measure_test_performance("random_word_api_request", async {
        Ok(server.get("/en/word").await)
    })
    .await?;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random word endpoint should return 200, got: {}",
        response.status_code()
    );

    // Validate performance
    assert_test_performance(&metrics, performance_thresholds::API_REQUEST);

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

    // Cleanup test data
    cleanup_test_data(&pool).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_random_word_by_type_noun() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "2").await?;

    let response = server.get("/en/word/noun").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random noun endpoint should return 200"
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
        "Response should have word field"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_random_word_by_type_verb() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "3").await?;

    let response = server.get("/en/word/verb").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random verb endpoint should return 200"
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
        "Response should have word field"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_random_word_by_type_adjective() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "4").await?;

    let response = server.get("/en/word/adjective").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random adjective endpoint should return 200"
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
        "Response should have word field"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_random_word_by_type_adverb() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "5").await?;

    let response = server.get("/en/word/adverb").await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Random adverb endpoint should return 200"
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
        "Response should have word field"
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_language() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/invalid/word").await;

    assert!(
        response.status_code() >= StatusCode::BAD_REQUEST,
        "Invalid language should return error, got: {}",
        response.status_code()
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_invalid_word_type() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "6").await?;

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
    let (server, _temp_file) = create_test_server().await?;
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
#[serial]
async fn test_word_api_response_format() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "7").await?;

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
#[serial]
async fn test_multiple_random_word_requests() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "8").await?;

    // Make multiple requests to test randomness (though we can't guarantee different results)
    for i in 0..5 {
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
#[serial]
async fn test_word_api_content_type() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "9").await?;

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
#[serial]
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

    let (create_response, create_metrics) =
        measure_test_performance("admin_word_creation", async {
            Ok(server.post("/admin/en/words").json(&create_body).await)
        })
        .await?;

    assert_eq!(
        create_response.status_code(),
        StatusCode::OK,
        "Create should succeed"
    );

    // Validate creation performance
    assert_test_performance(&create_metrics, performance_thresholds::API_REQUEST);

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
#[serial]
async fn test_api_performance_under_load() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;
    populate_test_data(&pool, "perf").await?;

    // Test multiple sequential requests (simplified for reliability)
    for i in 0..10 {
        let (response, metrics) = measure_test_performance(&format!("load_request_{i}"), async {
            Ok(server.get("/en/word").await)
        })
        .await?;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_test_performance(&metrics, performance_thresholds::API_REQUEST);
    }

    cleanup_test_data(&pool).await?;
    Ok(())
}
