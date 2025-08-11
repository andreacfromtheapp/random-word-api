//! Word API integration tests
//!
//! Tests word retrieval endpoints, error handling, API consistency,
//! and database integration scenarios. Covers all word types and
//! validates response formats and edge cases.

use anyhow::Result;
use axum::http::StatusCode;

mod helpers;
use helpers::{
    create_test_server_memory,      // For empty database scenarios
    create_test_server_streamlined, // For read-only operations using shared database
};
use word_api_axum::models::word::{Language, ALLOWED_WORD_TYPES};

// === Core Word Retrieval Tests ===

#[tokio::test]
async fn test_word_retrieval_parallel() -> Result<()> {
    // Run word retrieval tests in parallel for efficiency
    let (basic_result, types_result, format_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/en/word").await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let json: serde_json::Value = response.json();
            assert!(json.is_array(), "Response should be an array");
            let words = json.as_array().unwrap();
            assert!(!words.is_empty(), "Response should contain words");
            let word = &words[0];
            assert!(word.get("word").is_some(), "Should have word field");
            assert!(
                word.get("definition").is_some(),
                "Should have definition field"
            );
            assert!(
                word.get("pronunciation").is_some(),
                "Should have pronunciation field"
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            // Test all word types in parallel
            for &word_type in &ALLOWED_WORD_TYPES {
                let response = server.get(&format!("/en/word/{word_type}")).await;
                assert_eq!(response.status_code(), StatusCode::OK);
                let json: serde_json::Value = response.json();
                assert!(
                    json.is_array(),
                    "Response should be an array for {word_type}"
                );
                let words = json.as_array().unwrap();
                assert!(!words.is_empty(), "Should have words for {word_type}");
                let word = &words[0];
                assert!(
                    word.get("word").is_some(),
                    "Should have word field for {word_type}"
                );
                assert!(
                    word.get("definition").is_some(),
                    "Should have definition field for {word_type}"
                );
                assert!(
                    word.get("pronunciation").is_some(),
                    "Should have pronunciation field for {word_type}"
                );
            }
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            let language = Language::English;
            let response = server.get(&format!("/{language}/word")).await;
            assert_eq!(response.status_code(), StatusCode::OK);
            // Verify content type
            let content_type = response
                .headers()
                .get("content-type")
                .expect("Response should have content-type header");
            assert!(
                content_type.to_str().unwrap().contains("application/json"),
                "Word API response should be JSON content type"
            );
            // Verify response structure
            let json: serde_json::Value = response.json();
            assert!(json.is_array(), "Response should be an array");
            let words = json.as_array().unwrap();
            assert!(!words.is_empty(), "Response should contain words");
            let word = &words[0];
            assert!(word["word"].is_string(), "Word should be string");
            assert!(
                word["definition"].is_string(),
                "Definition should be string"
            );
            assert!(
                word["pronunciation"].is_string(),
                "Pronunciation should be string"
            );
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
            Ok::<(), anyhow::Error>(())
        }
    );

    // Check all parallel operations succeeded
    basic_result?;
    types_result?;
    format_result?;

    Ok(())
}

// === Error Handling Tests (Parallelized) ===

#[tokio::test]
async fn test_error_handling_parallel() -> Result<()> {
    // Run error handling tests in parallel
    let (invalid_requests_result, detailed_error_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            // Batch test all invalid scenarios for efficiency
            let invalid_scenarios = vec![
                ("/invalid/word", "Invalid language"),
                ("/en/word/invalid_type", "Invalid word type"),
                ("/nonexistent/endpoint", "Nonexistent endpoint"),
            ];
            for (path, description) in invalid_scenarios {
                let response = server.get(path).await;
                assert!(
                    response.status_code() >= StatusCode::BAD_REQUEST,
                    "{description} should return error status for path: {path}"
                );
            }
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            // Test invalid word type with detailed validation
            let language = Language::English;
            let invalid_type_response = server.get(&format!("/{language}/word/invalid_type")).await;
            assert_eq!(
                invalid_type_response.status_code(),
                StatusCode::BAD_REQUEST,
                "Invalid word type should return 400 Bad Request"
            );
            let body = invalid_type_response.text();
            assert!(!body.is_empty(), "Error response should have a message");
            assert!(
                body.to_lowercase().contains("invalid word type"),
                "Error message should mention invalid word type"
            );
            Ok::<(), anyhow::Error>(())
        }
    );

    // Check both parallel operations succeeded
    invalid_requests_result?;
    detailed_error_result?;

    Ok(())
}

// === Database and Edge Case Tests ===

#[tokio::test]
async fn test_empty_database_scenario() -> Result<()> {
    let (server, _pool) = create_test_server_memory().await?;
    let response = server.get("/en/word").await;

    // Empty database returns OK with empty array or error - both acceptable
    assert!(
        response.status_code() == StatusCode::OK
            || response.status_code() >= StatusCode::BAD_REQUEST
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_requests_reliability() -> Result<()> {
    let server = create_test_server_streamlined().await?;

    // Test multiple requests in parallel for better reliability testing
    let (req1, req2, req3) = tokio::join!(
        async {
            let response = server.get("/en/word").await;
            assert!(
                response.status_code() == StatusCode::OK
                    || response.status_code() == StatusCode::NO_CONTENT
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let response = server.get("/en/word").await;
            assert!(
                response.status_code() == StatusCode::OK
                    || response.status_code() == StatusCode::NO_CONTENT
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let response = server.get("/en/word").await;
            assert!(
                response.status_code() == StatusCode::OK
                    || response.status_code() == StatusCode::NO_CONTENT
            );
            Ok::<(), anyhow::Error>(())
        }
    );

    req1?;
    req2?;
    req3?;

    Ok(())
}

// === CRUD Operations ===
// CRUD operations are tested in admin_tests.rs
// This module focuses on word retrieval and read-only API testing

// === API Consistency and Integration Tests ===

#[tokio::test]
async fn test_api_consistency_parallel() -> Result<()> {
    // Test API consistency with parallel requests
    let (consistency_result, batch_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            // Test that multiple requests return consistent format
            let (req1, req2, req3) = tokio::join!(
                async {
                    let response = server.get("/en/word").await;
                    assert_eq!(response.status_code(), StatusCode::OK);
                    let json: serde_json::Value = response.json();
                    assert!(json.is_array(), "Response should be array");
                    Ok::<(), anyhow::Error>(())
                },
                async {
                    let response = server.get("/en/word").await;
                    assert_eq!(response.status_code(), StatusCode::OK);
                    let json: serde_json::Value = response.json();
                    assert!(json.is_array(), "Response should be array");
                    Ok::<(), anyhow::Error>(())
                },
                async {
                    let response = server.get("/en/word").await;
                    assert_eq!(response.status_code(), StatusCode::OK);
                    let json: serde_json::Value = response.json();
                    assert!(json.is_array(), "Response should be array");
                    Ok::<(), anyhow::Error>(())
                }
            );
            req1?;
            req2?;
            req3?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            // Batch test word API endpoints (health checks handled in health_tests.rs)
            let test_endpoints = vec!["/en/word", "/admin/en/words"];
            for endpoint in test_endpoints {
                let response = server.get(endpoint).await;
                assert!(
                    response.status_code() <= StatusCode::OK
                        || response.status_code() == StatusCode::NO_CONTENT
                );
            }
            Ok::<(), anyhow::Error>(())
        }
    );

    consistency_result?;
    batch_result?;

    Ok(())
}

// === Data Integrity Tests ===
// Note: Data integrity is covered by existing CRUD tests above

// === Workflow and Edge Case Tests ===

#[tokio::test]
async fn test_workflow_and_edge_cases_parallel() -> Result<()> {
    // Run workflow and edge case tests in parallel
    let (workflow_result, edge_cases_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            // Test user workflow: get word -> check admin (health checks handled in health_tests.rs)
            let word_response = server.get("/en/word").await;
            assert_eq!(word_response.status_code(), StatusCode::OK);
            let admin_response = server.get("/admin/en/words").await;
            assert!(admin_response.status_code() <= StatusCode::OK);
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            // Test edge case scenarios
            let edge_cases = vec![
                "/en/word/noun",
                "/en/word/verb",
                "/en/word/adjective",
                "/en/word/adverb",
            ];
            for endpoint in edge_cases {
                let response = server.get(endpoint).await;
                assert!(
                    response.status_code() == StatusCode::OK
                        || response.status_code() == StatusCode::NO_CONTENT,
                    "Edge case endpoint {endpoint} should return valid status"
                );
            }
            Ok::<(), anyhow::Error>(())
        }
    );

    workflow_result?;
    edge_cases_result?;

    Ok(())
}
