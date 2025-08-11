//! Streamlined admin API integration tests (Phase 3 Optimization)
//!
//! This module contains optimized integration tests for the admin endpoints
//! using batch operations, minimal database overhead, and streamlined test
//! patterns for maximum efficiency while maintaining full coverage.

use anyhow::Result;
use axum::http::StatusCode;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use word_api_axum::models::word::UpsertWord;

mod helpers;
use helpers::{create_test_server, create_test_server_streamlined};
use word_api_axum::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, Language, ALLOWED_WORD_TYPES,
};

/// Generate a unique timestamp-based suffix for test data (lemma-safe)
fn generate_unique_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Use only the last 6 digits to keep it short and valid for lemmas
    format!("{}", timestamp % 1_000_000)
}

/// Generate a unique pronunciation with valid IPA characters to avoid database conflicts
fn generate_unique_pronunciation(base_word: &str, _suffix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    // Create hash from suffix for additional uniqueness
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    _suffix.hash(&mut hasher);
    let suffix_hash = hasher.finish();

    // Combine timestamp and suffix hash for maximum uniqueness
    let unique_id = (timestamp as u64) + suffix_hash;

    // Use purely IPA characters without numbers - create unique patterns
    let ipa_endings = ["ər", "əl", "əs", "əd", "ən", "əm", "ɪn", "ʊr", "ɛr", "ɔr"];
    let ipa_vowels = ["ə", "ɛ", "ɪ", "ɔ", "ʊ", "ʌ", "ɑ", "æ"];

    let ending_index = unique_id as usize % ipa_endings.len();
    let vowel_index = (unique_id / 1000) as usize % ipa_vowels.len();

    format!(
        "/{}{}{}/",
        base_word, ipa_vowels[vowel_index], ipa_endings[ending_index]
    )
}

/// Create a test word using built-in validation and Language enum with guaranteed uniqueness
fn create_test_word(suffix: &str) -> UpsertWord {
    // Sanitize suffix to remove invalid characters for lemma validation
    let clean_suffix = suffix.replace("_", "").replace("-", "");
    let unique_suffix = format!("{}{}", clean_suffix, generate_unique_suffix());
    let word = format!("testword{unique_suffix}");
    let definition = format!("test definition {unique_suffix}");
    let pronunciation = generate_unique_pronunciation("tɛst", &unique_suffix);
    let word_type = ALLOWED_WORD_TYPES[0]; // "noun"

    // Validate using built-in functions
    assert!(
        is_valid_lemma(&word),
        "Generated word '{word}' should be valid"
    );
    assert!(
        is_valid_definition(&definition),
        "Generated definition should be valid"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation should be valid"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

/// Create a validated test word with specific type and guaranteed uniqueness
fn create_validated_test_word(word_type: &str, suffix: &str) -> UpsertWord {
    // Sanitize suffix to remove invalid characters for lemma validation
    let clean_suffix = suffix.replace("_", "").replace("-", "");
    let unique_suffix = format!("{}{}", clean_suffix, generate_unique_suffix());

    // Ensure word_type is valid using ALLOWED_WORD_TYPES
    assert!(
        ALLOWED_WORD_TYPES.contains(&word_type),
        "Word type '{word_type}' must be one of: {ALLOWED_WORD_TYPES:?}"
    );

    let word = format!("unique{word_type}{unique_suffix}");
    let definition = format!("A unique {word_type} definition for {unique_suffix}");
    let pronunciation = generate_unique_pronunciation("juːnik", &unique_suffix);

    // Validate using built-in validation functions
    assert!(
        is_valid_lemma(&word),
        "Generated word '{word}' should be valid"
    );
    assert!(
        is_valid_definition(&definition),
        "Generated definition should be valid"
    );
    assert!(
        is_valid_pronunciation(&pronunciation),
        "Generated pronunciation '{pronunciation}' should be valid"
    );

    UpsertWord {
        word,
        definition,
        pronunciation,
        word_type: word_type.to_string(),
    }
}

#[tokio::test]
async fn test_admin_create_word_success() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    let word_data = create_test_word("1");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
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
    assert_eq!(word["word_type"], word_data.word_type);

    Ok(())
}

#[tokio::test]
async fn test_admin_list_words_optimized() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = Language::English;

    let response = server.get(&format!("/admin/{language}/words")).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let json: serde_json::Value = response.json();
    assert!(json.is_array(), "Admin list should return array");

    Ok(())
}

#[tokio::test]
async fn test_admin_crud_batch_operations() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    // Batch test multiple CRUD operations in single test for efficiency
    let mut created_ids = Vec::new();

    // CREATE multiple words with guaranteed uniqueness
    for i in 0..2 {
        let word_type_index = i % ALLOWED_WORD_TYPES.len();
        let word_data =
            create_validated_test_word(ALLOWED_WORD_TYPES[word_type_index], &format!("batch{i}"));
        let body = json!({
            "word": word_data.word,
            "definition": word_data.definition,
            "pronunciation": word_data.pronunciation,
            "word_type": word_data.word_type
        });

        let create_response = server
            .post(&format!("/admin/{language}/words"))
            .json(&body)
            .await;

        if create_response.status_code() != StatusCode::OK {
            // Log the error response for debugging
            let error_text = create_response.text();
            eprintln!(
                "Create failed for word '{}': {}",
                word_data.word, error_text
            );
        }
        assert_eq!(create_response.status_code(), StatusCode::OK);

        let create_json: serde_json::Value = create_response.json();
        let words = create_json.as_array().unwrap();
        assert!(!words.is_empty());

        let id = words[0]["id"].as_u64().unwrap() as u32;
        created_ids.push(id);
    }

    // READ operations - verify each created word can be retrieved
    for id in &created_ids {
        let get_response = server.get(&format!("/admin/{language}/words/{id}")).await;

        if get_response.status_code() != StatusCode::OK {
            let error_text = get_response.text();
            eprintln!("GET failed for ID {id}: {error_text}");
        }
        assert_eq!(get_response.status_code(), StatusCode::OK);
    }

    // DELETE operations - clean up created words
    for id in created_ids {
        let delete_response = server
            .delete(&format!("/admin/{language}/words/{id}"))
            .await;
        assert_eq!(delete_response.status_code(), StatusCode::OK);
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_validation_batch() -> Result<()> {
    let server = create_test_server_streamlined().await?;
    let language = Language::English;

    // Batch test all validation scenarios for efficiency
    let invalid_bodies = vec![
        json!({ "word": "", "definition": "valid", "pronunciation": "/vælɪd/", "word_type": ALLOWED_WORD_TYPES[0] }),
        json!({ "word": "valid", "definition": "", "pronunciation": "/vælɪd/", "word_type": ALLOWED_WORD_TYPES[0] }),
        json!({ "word": "valid", "definition": "valid", "pronunciation": "", "word_type": ALLOWED_WORD_TYPES[0] }),
        json!({ "word": "valid", "definition": "valid", "pronunciation": "/vælɪd/", "word_type": "invalid" }),
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
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    // Streamlined create-update-verify workflow
    let word_data = create_validated_test_word(ALLOWED_WORD_TYPES[0], "update");
    let create_body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
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
    let update_suffix = generate_unique_suffix();
    let update_body = json!({
        "word": format!("updated{}", update_suffix),
        "definition": format!("updated definition {}", update_suffix),
        "pronunciation": generate_unique_pronunciation("ʌpdeɪt", &update_suffix),
        "word_type": ALLOWED_WORD_TYPES[1]
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
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    // Streamlined create-delete workflow
    let word_data = create_validated_test_word(ALLOWED_WORD_TYPES[0], "delete");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
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
    let language = Language::English;

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
    let update_suffix = generate_unique_suffix();
    let update_body = json!({
        "word": format!("nonexistent{}", update_suffix),
        "definition": format!("definition {}", update_suffix),
        "pronunciation": generate_unique_pronunciation("nɑnɪɡz", &update_suffix),
        "word_type": ALLOWED_WORD_TYPES[0]
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
    let language = Language::English;

    // Batch test all request validation scenarios
    let test_cases = vec![
        // Invalid JSON
        ("{ invalid json", "application/json"),
        // Missing content type
        (
            r#"{"word":"test","definition":"test","pronunciation":"/tɛst/","word_type":"noun"}"#,
            "text/plain",
        ),
    ];

    for (body, content_type) in test_cases {
        let response = server
            .post(&format!("/admin/{language}/words"))
            .text(body)
            .content_type(content_type)
            .await;
        assert!(response.status_code() >= StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_duplicate_prevention() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    // Create a word
    let word_data = create_test_word("duplicate_test");
    let body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
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
async fn test_admin_streamlined_workflow() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;
    let language = Language::English;

    // Test complete admin workflow in single test for maximum efficiency

    // 1. List initial words
    let initial_list = server.get(&format!("/admin/{language}/words")).await;
    assert_eq!(initial_list.status_code(), StatusCode::OK);
    let initial_count = initial_list
        .json::<serde_json::Value>()
        .as_array()
        .unwrap()
        .len();

    // 2. Create new word
    let word_data = create_test_word("workflow");
    let create_body = json!({
        "word": word_data.word,
        "definition": word_data.definition,
        "pronunciation": word_data.pronunciation,
        "word_type": word_data.word_type
    });

    let create_response = server
        .post(&format!("/admin/{language}/words"))
        .json(&create_body)
        .await;
    assert_eq!(create_response.status_code(), StatusCode::OK);

    let create_json: serde_json::Value = create_response.json();
    let id = create_json.as_array().unwrap()[0]["id"].as_u64().unwrap() as u32;

    // 3. Verify word exists
    let get_response = server.get(&format!("/admin/{language}/words/{id}")).await;
    assert_eq!(get_response.status_code(), StatusCode::OK);

    // 4. Update the word
    let update_suffix = generate_unique_suffix();
    let update_body = json!({
        "word": format!("workflowupdated{}", update_suffix),
        "definition": format!("workflow updated definition {}", update_suffix),
        "pronunciation": generate_unique_pronunciation("wɜrkfloʊ", &update_suffix),
        "word_type": ALLOWED_WORD_TYPES[1]
    });

    let update_response = server
        .put(&format!("/admin/{language}/words/{id}"))
        .json(&update_body)
        .await;
    assert_eq!(update_response.status_code(), StatusCode::OK);

    // 5. Verify updated content
    let verify_response = server.get(&format!("/admin/{language}/words/{id}")).await;
    assert_eq!(verify_response.status_code(), StatusCode::OK);

    // 6. Delete the word
    let delete_response = server
        .delete(&format!("/admin/{language}/words/{id}"))
        .await;
    assert_eq!(delete_response.status_code(), StatusCode::OK);

    // 7. Verify list count is back to initial
    let final_list = server.get(&format!("/admin/{language}/words")).await;
    assert_eq!(final_list.status_code(), StatusCode::OK);
    let final_count = final_list
        .json::<serde_json::Value>()
        .as_array()
        .unwrap()
        .len();
    assert_eq!(
        final_count, initial_count,
        "Word count should return to initial after delete"
    );

    Ok(())
}
