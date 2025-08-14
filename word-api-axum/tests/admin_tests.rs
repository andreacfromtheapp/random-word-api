//! Admin API integration tests
//!
//! Tests for admin endpoints including CRUD operations, validation,
//! error handling, and request format validation.

use anyhow::Result;
use axum::http::StatusCode;
use serde_json::json;

use word_api_axum::models::word::UpsertWord;

mod helpers;
use helpers::{
    create_test_server,             // For write operations requiring isolated database
    create_test_server_streamlined, // For read-only operations using shared database
};
use word_api_axum::models::word::{GrammaticalType, LanguageCode};

/// Create a test word for admin testing
fn create_test_word(suffix: &str) -> UpsertWord {
    helpers::test_data::create_basic_test_word(suffix)
}

/// Create a test word of a specific type for admin testing
fn create_validated_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    helpers::test_data::create_typed_test_word(word_type, suffix)
}

#[tokio::test]
async fn test_admin_create_word_success() -> Result<()> {
    let server = create_test_server().await?;
    let language = LanguageCode::English;

    let word_data = create_test_word("1");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "wordType": word_data.word_type
    });

    let response = server
        .post(&format!("/admin/{language}/words"))
        .json(&body)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json: serde_json::Value = response.json();
    assert!(json.is_array(), "Admin API should return array");

    let words = json.as_array().unwrap();
    assert!(!words.is_empty(), "Response should contain created word");

    let word = &words[0];
    assert_eq!(word["word"], word_data.word.to_lowercase());
    assert_eq!(word["wordType"], word_data.word_type);

    Ok(())
}

#[tokio::test]
async fn test_admin_list_words_optimized() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = LanguageCode::English;

    let response = server.get(&format!("/admin/{language}/words")).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let json: serde_json::Value = response.json();
    assert!(json.is_array(), "Admin list should return array");

    Ok(())
}

// #[tokio::test]
// async fn test_admin_crud_batch_operations() -> Result<()> {
//     let server = create_test_server().await?;
//     let language = LanguageCode::English;

//     // Batch test multiple CRUD operations in single test for efficiency
//     let mut created_ids = Vec::new();

//     // CREATE multiple words with guaranteed uniqueness
//     for i in 0..2 {
//         let word_type_index = i % ALLOWED_WORD_TYPES.len();
//         let word_data =
//             create_validated_test_word(ALLOWED_WORD_TYPES[word_type_index], &format!("batch{i}"));
//         let body = json!({
//             "word": word_data.word,
//             "definition": word_data.definition,
//             "pronunciation": word_data.pronunciation,
//             "wordType": word_data.word_type
//         });

//         let create_response = server
//             .post(&format!("/admin/{language}/words"))
//             .json(&body)
//             .await;

//         if create_response.status_code() != StatusCode::OK {
//             // Log the error response for debugging
//             let error_text = create_response.text();
//             eprintln!(
//                 "Create failed for word '{}': {}",
//                 word_data.word, error_text
//             );
//         }
//         assert_eq!(create_response.status_code(), StatusCode::OK);

//         let create_json: serde_json::Value = create_response.json();
//         let words = create_json.as_array().unwrap();
//         assert!(!words.is_empty());

//         let id = words[0]["id"].as_u64().unwrap() as u32;
//         created_ids.push(id);
//     }

//     // READ operations - verify each created word can be retrieved
//     for id in &created_ids {
//         let get_response = server.get(&format!("/admin/{language}/words/{id}")).await;

//         if get_response.status_code() != StatusCode::OK {
//             let error_text = get_response.text();
//             eprintln!("GET failed for ID {id}: {error_text}");
//         }
//         assert_eq!(get_response.status_code(), StatusCode::OK);
//     }

//     // DELETE operations - clean up created words
//     for id in created_ids {
//         let delete_response = server
//             .delete(&format!("/admin/{language}/words/{id}"))
//             .await;
//         assert_eq!(delete_response.status_code(), StatusCode::OK);
//     }

//     Ok(())
// }

#[tokio::test]
async fn test_admin_validation_batch() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = LanguageCode::English;
    let word_type = GrammaticalType::Noun;

    // Use source validation by testing cases that should fail according to source ALLOWED_WORD_TYPES
    let invalid_bodies = vec![
        json!({ "word": "", "definition": "valid", "pronunciation": "/vælɪd/", "wordType": word_type.type_name() }),
        json!({ "word": "valid", "definition": "", "pronunciation": "/vælɪd/", "wordType": word_type.type_name() }),
        json!({ "word": "valid", "definition": "valid", "pronunciation": "", "wordType": word_type.type_name() }),
        // Use source validation - test invalid word type not in ALLOWED_WORD_TYPES
        json!({ "word": "valid", "definition": "valid", "pronunciation": "/vælɪd/", "wordType": "invalid" }),
        json!({ "word": "valid", "definition": "valid", "pronunciation": "/vælɪd/", "wordType": "determiner" }),
    ];

    for invalid_body in invalid_bodies {
        let response = server
            .post(&format!("/admin/{language}/words"))
            .json(&invalid_body)
            .await;
        assert!(response.status_code() >= StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_update_streamlined() -> Result<()> {
    let server = create_test_server().await?;
    let language = LanguageCode::English;
    let type_noun = GrammaticalType::Noun;
    let type_verb = GrammaticalType::Verb;

    // Streamlined create-update-verify workflow
    let word_data = create_validated_test_word(type_noun.type_name(), "update");
    let create_body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "wordType": word_data.word_type
    });

    // CREATE
    let create_response = server
        .post(&format!("/admin/{language}/words"))
        .json(&create_body)
        .await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let id = create_json.as_array().unwrap()[0]["id"].as_u64().unwrap() as u32;

    // UPDATE with guaranteed unique data
    let update_word = create_validated_test_word(type_verb.type_name(), "updated");
    let update_body = json!({
        "word": update_word.word,
        "definition": update_word.definition,
        "pronunciation": update_word.pronunciation,
        "wordType": update_word.word_type
    });

    let update_response = server
        .put(&format!("/admin/{language}/words/{id}"))
        .json(&update_body)
        .await;
    assert_eq!(update_response.status_code(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn test_admin_delete_streamlined() -> Result<()> {
    let server = create_test_server().await?;
    let language = LanguageCode::English;
    let type_noun = GrammaticalType::Noun;

    // Streamlined create-delete workflow
    let word_data = create_validated_test_word(type_noun.type_name(), "delete");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "wordType": word_data.word_type
    });

    // CREATE
    let create_response = server
        .post(&format!("/admin/{language}/words"))
        .json(&body)
        .await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let id = create_json.as_array().unwrap()[0]["id"].as_u64().unwrap() as u32;

    // DELETE
    let delete_response = server
        .delete(&format!("/admin/{language}/words/{id}"))
        .await;
    assert_eq!(delete_response.status_code(), StatusCode::OK);

    // VERIFY deletion (should return empty array or appropriate response)
    let verify_response = server.get(&format!("/admin/{language}/words/{id}")).await;
    assert_eq!(verify_response.status_code(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn test_admin_nonexistent_operations_batch() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = LanguageCode::English;
    let word_type = GrammaticalType::Noun;

    // Batch test all nonexistent operations for efficiency
    let nonexistent_id = 99999;

    // GET nonexistent
    let get_response = server
        .get(&format!("/admin/{language}/words/{nonexistent_id}"))
        .await;
    assert_eq!(get_response.status_code(), StatusCode::OK);

    // DELETE nonexistent (should succeed)
    let delete_response = server
        .delete(&format!("/admin/{language}/words/{nonexistent_id}"))
        .await;
    assert_eq!(delete_response.status_code(), StatusCode::OK);

    // UPDATE nonexistent (should fail)
    let update_body = json!({
        "word": "nonexistent",
        "definition": "nonexistent definition",
        "pronunciation": "/nɑnɪɡzɪstənt/",
        "wordType": word_type.type_name(),
    });
    let update_response = server
        .put(&format!("/admin/{language}/words/{nonexistent_id}"))
        .json(&update_body)
        .await;
    assert_eq!(
        update_response.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );

    Ok(())
}

#[tokio::test]
async fn test_admin_request_validation_batch() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = LanguageCode::English;

    // Test request format validation (distinct from data validation)
    let test_cases = vec![
        // Invalid JSON syntax
        ("{ invalid json", "application/json"),
        // Wrong content type (valid JSON, wrong header)
        (
            r#"{"word":"test","definition":"test","pronunciation":"/tɛst/","wordType":"noun"}"#,
            "text/plain",
        ),
        // Valid content type but malformed JSON
        ("not json at all", "application/json"),
    ];

    for (body, content_type) in test_cases {
        let response = server
            .post(&format!("/admin/{language}/words"))
            .text(body)
            .content_type(content_type)
            .await;
        assert!(
            response.status_code() >= StatusCode::BAD_REQUEST,
            "Should reject malformed request with body='{body}' and content_type='{content_type}'"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_duplicate_prevention() -> Result<()> {
    let server = create_test_server().await?;
    let language = LanguageCode::English;

    // Create a word
    let word_data = create_test_word("duplicate_test");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "wordType": word_data.word_type
    });

    let first_response = server
        .post(&format!("/admin/{language}/words"))
        .json(&body)
        .await;
    assert_eq!(first_response.status_code(), StatusCode::OK);

    // Try to create the same word again - should fail due to UNIQUE constraints
    let duplicate_response = server
        .post(&format!("/admin/{language}/words"))
        .json(&body)
        .await;

    // Should return an error status due to UNIQUE constraint violation
    assert!(duplicate_response.status_code() >= StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_source_validation_integration() -> Result<()> {
    // Test that admin endpoints leverage source ALLOWED_WORD_TYPES validation
    let server = create_test_server_streamlined().await?;
    let language = LanguageCode::English;

    // Test that types not in source ALLOWED_WORD_TYPES are rejected
    let invalid_types = ["determiner", "particle"];
    for invalid_type in invalid_types {
        let invalid_body = json!({
            "word": "valid",
            "definition": "valid definition",
            "pronunciation": "/vælɪd/",
            "wordType": invalid_type
        });

        let response = server
            .post(&format!("/admin/{language}/words"))
            .json(&invalid_body)
            .await;

        assert!(
            response.status_code() >= StatusCode::BAD_REQUEST,
            "Invalid word type '{invalid_type}' should be rejected by source validation"
        );
    }

    Ok(())
}
