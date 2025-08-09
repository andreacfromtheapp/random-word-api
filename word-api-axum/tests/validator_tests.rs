//! Validator integration tests
//!
//! This module provides comprehensive testing of the validation functions
//! from word.rs, ensuring they work correctly in isolation and integration
//! with database operations.

mod common;
use common::{
    create_test_db, invalid_definition_field, invalid_pronunciation_field, invalid_word,
    invalid_word_field, invalid_word_type_field, sample_word, sample_word_with_type,
    validate_test_word,
};
use word_api_axum::models::word::{
    is_valid_definition, is_valid_lemma, is_valid_pronunciation, UpsertWord, Word,
};

#[tokio::test]
async fn test_lemma_validator_comprehensive() {
    // Valid cases (alphanumeric, hyphen, apostrophe, period, accented chars)
    assert!(is_valid_lemma("test"));
    assert!(is_valid_lemma("hello"));
    assert!(is_valid_lemma("world"));
    assert!(is_valid_lemma("test123"));
    assert!(is_valid_lemma("hello-world"));
    assert!(is_valid_lemma("multi-part-word"));
    assert!(is_valid_lemma("123abc"));
    assert!(is_valid_lemma("a"));
    assert!(is_valid_lemma("AB"));
    assert!(is_valid_lemma("test.com")); // Period is allowed
    assert!(is_valid_lemma("don't")); // Apostrophe is allowed
    assert!(is_valid_lemma("café")); // Accented characters are allowed

    // Invalid cases
    assert!(!is_valid_lemma(""));
    assert!(!is_valid_lemma("invalid word")); // Contains space
    assert!(!is_valid_lemma("test@invalid")); // Contains @
    assert!(!is_valid_lemma("hello world")); // Contains space
    assert!(!is_valid_lemma("user@domain")); // Contains @
    assert!(!is_valid_lemma("hello world!")); // Contains space and !
    assert!(!is_valid_lemma("test/slash")); // Contains slash
    assert!(!is_valid_lemma("test\\backslash")); // Contains backslash
    assert!(!is_valid_lemma("test|pipe")); // Contains pipe
    assert!(!is_valid_lemma("test*asterisk")); // Contains asterisk
}

#[tokio::test]
async fn test_definition_validator_comprehensive() {
    // Valid cases (letters, numbers, spaces, common punctuation)
    assert!(is_valid_definition("a simple definition"));
    assert!(is_valid_definition("Test with Capital letters"));
    assert!(is_valid_definition("Definition with punctuation!"));
    assert!(is_valid_definition("Numbers 123 are allowed"));
    assert!(is_valid_definition("Apostrophe's are valid"));
    assert!(is_valid_definition("Hyphen-ated words work"));
    assert!(is_valid_definition("Question marks? Yes."));
    assert!(is_valid_definition("Commas, semicolons; work too"));
    assert!(is_valid_definition("Parentheses (like this) are fine"));
    assert!(is_valid_definition("Single quotes work fine"));
    assert!(is_valid_definition("Accented characters like café"));
    assert!(is_valid_definition("Colons: work too"));

    // Invalid cases (symbols not in the allowed character class)
    assert!(!is_valid_definition(""));
    assert!(!is_valid_definition("test@invalid.com")); // Contains @
    assert!(!is_valid_definition("definition with @ symbol"));
    assert!(!is_valid_definition("definition with # hash"));
    assert!(!is_valid_definition("definition with $ dollar"));
    assert!(!is_valid_definition("definition with % percent"));
    assert!(!is_valid_definition("definition with ^ caret"));
    assert!(!is_valid_definition("definition with & ampersand"));
    assert!(!is_valid_definition("definition with * asterisk"));
    assert!(!is_valid_definition("definition with + plus"));
    assert!(!is_valid_definition("definition with = equals"));
    assert!(!is_valid_definition("definition with \" quotes")); // Double quotes not allowed
}

#[tokio::test]
async fn test_pronunciation_validator_comprehensive() {
    // Valid cases - proper IPA format
    assert!(is_valid_pronunciation("/tɛst/"));
    assert!(is_valid_pronunciation("/həˈloʊ/"));
    assert!(is_valid_pronunciation("/ˈwɜːrd/"));
    assert!(is_valid_pronunciation("/ˌɪntərˈnæʃənəl/"));
    assert!(is_valid_pronunciation("/ˈθɪŋkɪŋ/"));
    assert!(is_valid_pronunciation("/ˈbeɪsɪk/"));
    assert!(is_valid_pronunciation("/kəmˈplɛks/"));
    assert!(is_valid_pronunciation("/ˈsɪmpəl/"));

    // Invalid cases
    assert!(!is_valid_pronunciation(""));
    assert!(!is_valid_pronunciation("invalid")); // Missing slashes
    assert!(!is_valid_pronunciation("/invalid@/")); // Contains @
    assert!(!is_valid_pronunciation("/")); // Only slashes, no content
    assert!(!is_valid_pronunciation("//")); // Empty between slashes
    assert!(!is_valid_pronunciation("test")); // No slashes
    assert!(!is_valid_pronunciation("/test")); // Missing closing slash
    assert!(!is_valid_pronunciation("test/")); // Missing opening slash
    assert!(!is_valid_pronunciation("/test123/")); // Contains numbers
    assert!(!is_valid_pronunciation("/test with space/")); // Contains space
    assert!(!is_valid_pronunciation("/test@invalid/")); // Contains @
}

#[tokio::test]
async fn test_word_type_validation() {
    // Valid word types
    let valid_types = ["noun", "verb", "adjective", "adverb"];

    for word_type in valid_types {
        let word_data = sample_word_with_type(word_type);
        assert!(
            validate_test_word(&word_data),
            "Word type '{word_type}' should be valid"
        );
    }

    // Invalid word types - these would be caught by database constraints
    // but we can test the validation logic
    let invalid_types = [
        "preposition",
        "article",
        "conjunction",
        "invalid",
        "",
        "NOUN",
    ];

    for word_type in invalid_types {
        let mut word_data = sample_word();
        word_data.word_type = word_type.to_string();

        assert!(
            !validate_test_word(&word_data),
            "Word type '{word_type}' should be invalid"
        );
    }
}

#[tokio::test]
async fn test_validator_consistency_with_database() {
    let pool = create_test_db().await;

    // Test that validators are consistent with database operations

    // Valid data should succeed in both validators and database
    let valid_word = sample_word();
    assert!(
        validate_test_word(&valid_word),
        "Validators should accept valid word"
    );

    let db_result = Word::create(pool.clone(), "en", valid_word).await;
    assert!(
        db_result.is_ok(),
        "Database should accept validator-approved word"
    );

    // Invalid data should fail in both validators and database
    let invalid_word_data = invalid_word();
    assert!(
        !validate_test_word(&invalid_word_data),
        "Validators should reject invalid word"
    );

    let db_result = Word::create(pool.clone(), "en", invalid_word_data).await;
    assert!(
        db_result.is_err(),
        "Database should reject validator-rejected word"
    );
}

#[tokio::test]
async fn test_field_specific_validation_consistency() {
    let pool = create_test_db().await;

    // Test each field's validation consistency between validators and database

    // Invalid word field
    let invalid_word_data = invalid_word_field();
    assert!(
        !is_valid_lemma(&invalid_word_data.word),
        "Word validator should reject invalid word"
    );
    let result = Word::create(pool.clone(), "en", invalid_word_data).await;
    assert!(result.is_err(), "Database should reject invalid word field");

    // Invalid definition field
    let invalid_def_data = invalid_definition_field();
    assert!(
        !is_valid_definition(&invalid_def_data.definition),
        "Definition validator should reject invalid definition"
    );
    let result = Word::create(pool.clone(), "en", invalid_def_data).await;
    assert!(
        result.is_err(),
        "Database should reject invalid definition field"
    );

    // Invalid pronunciation field
    let invalid_pron_data = invalid_pronunciation_field();
    assert!(
        !is_valid_pronunciation(&invalid_pron_data.pronunciation),
        "Pronunciation validator should reject invalid pronunciation"
    );
    let result = Word::create(pool.clone(), "en", invalid_pron_data).await;
    assert!(
        result.is_err(),
        "Database should reject invalid pronunciation field"
    );

    // Invalid word type field
    let invalid_type_data = invalid_word_type_field();
    assert!(
        !["noun", "verb", "adjective", "adverb"].contains(&invalid_type_data.word_type.as_str()),
        "Word type should be invalid"
    );
    let result = Word::create(pool.clone(), "en", invalid_type_data).await;
    assert!(
        result.is_err(),
        "Database should reject invalid word type field"
    );
}

#[tokio::test]
async fn test_boundary_conditions() {
    // Test edge cases and boundary conditions for validators

    // Test minimum valid inputs
    assert!(is_valid_lemma("a"));
    assert!(is_valid_definition("a"));
    assert!(is_valid_pronunciation("/a/"));

    // Test very long but valid inputs
    let long_word = "a".repeat(100);
    let long_definition = "a valid definition ".repeat(50);
    let long_pronunciation = format!("/{}/", "a".repeat(50));

    assert!(is_valid_lemma(&long_word));
    assert!(is_valid_definition(&long_definition));
    assert!(is_valid_pronunciation(&long_pronunciation));

    // Test Unicode and special characters in definitions (should be valid for accented chars)
    assert!(is_valid_definition("définition avec accents"));
    assert!(is_valid_definition("määritelmä with Finnish"));
    // Note: Chinese characters are not in the supported Unicode ranges

    // Test IPA-specific characters in pronunciations (should be valid)
    assert!(is_valid_pronunciation("/θɪŋk/")); // theta
    assert!(is_valid_pronunciation("/ðɪs/")); // eth
    assert!(is_valid_pronunciation("/ʃʊər/")); // esh
    assert!(is_valid_pronunciation("/ʒənˈrɑl/")); // ezh
    assert!(is_valid_pronunciation("/ʧɪp/")); // chi
    assert!(is_valid_pronunciation("/ʤʌmp/")); // jot
    assert!(is_valid_pronunciation("/ŋɪŋ/")); // eng
}

#[tokio::test]
async fn test_validator_performance() {
    // Test that validators perform well with repeated calls
    use std::time::Instant;

    let start = Instant::now();

    // Run validators many times
    for i in 0..1000 {
        let word = format!("test{i}");
        let definition = format!("test definition {i}");
        let pronunciation = "/tɛst/";

        assert!(is_valid_lemma(&word));
        assert!(is_valid_definition(&definition));
        assert!(is_valid_pronunciation(pronunciation));
    }

    let duration = start.elapsed();

    // Should complete quickly (arbitrary threshold)
    assert!(
        duration.as_millis() < 100,
        "Validators should be fast: took {duration:?}"
    );
}

#[tokio::test]
async fn test_create_all_valid_combinations() {
    let pool = create_test_db().await;

    // Test creating words with all combinations of valid word types
    let word_types = ["noun", "verb", "adjective", "adverb"];
    let mut created_count = 0;

    for word_type in word_types {
        // Create multiple words of each type
        for i in 0..3 {
            let word_data = UpsertWord {
                word: format!("valid{word_type}{i}"),
                definition: format!("a valid {word_type} definition for testing {i}"),
                pronunciation: match (word_type, i) {
                    ("noun", 0) => "/ˈvalɪdnaʊn/".to_string(),
                    ("noun", 1) => "/ˈvalɪdnaʊnə/".to_string(),
                    ("noun", 2) => "/ˈvalɪdnaʊnɪ/".to_string(),
                    ("verb", 0) => "/ˈvalɪdvɜːb/".to_string(),
                    ("verb", 1) => "/ˈvalɪdvɜːbə/".to_string(),
                    ("verb", 2) => "/ˈvalɪdvɜːbɪ/".to_string(),
                    ("adjective", 0) => "/ˈvalɪdæʤəktɪv/".to_string(),
                    ("adjective", 1) => "/ˈvalɪdæʤəktɪvə/".to_string(),
                    ("adjective", 2) => "/ˈvalɪdæʤəktɪvɪ/".to_string(),
                    ("adverb", 0) => "/ˈvalɪdædvɜːb/".to_string(),
                    ("adverb", 1) => "/ˈvalɪdædvɜːbə/".to_string(),
                    ("adverb", 2) => "/ˈvalɪdædvɜːbɪ/".to_string(),
                    _ => "/ˈvalɪd/".to_string(),
                },
                word_type: word_type.to_string(),
            };

            // Validate test data using our validators
            assert!(
                validate_test_word(&word_data),
                "Generated word data should be valid according to validators: word='{}', definition='{}', pronunciation='{}', type='{}'",
                word_data.word, word_data.definition, word_data.pronunciation, word_data.word_type
            );

            let result = Word::create(pool.clone(), "en", word_data).await;

            assert!(
                result.is_ok(),
                "Database should accept validator-approved word of type {word_type}"
            );

            created_count += 1;
        }
    }

    // Verify all words were created
    let all_words = Word::list(pool, "en").await.unwrap();
    assert_eq!(all_words.len(), created_count);
}

#[tokio::test]
async fn test_validation_error_messages() {
    let pool = create_test_db().await;

    // Test that validation failures provide meaningful context
    // by using our validator-generated invalid data

    let test_cases = [
        ("invalid word field", invalid_word_field()),
        ("invalid definition field", invalid_definition_field()),
        ("invalid pronunciation field", invalid_pronunciation_field()),
        ("invalid word type field", invalid_word_type_field()),
    ];

    for (test_name, invalid_data) in test_cases {
        // Verify the data is actually invalid according to our validators
        assert!(
            !validate_test_word(&invalid_data),
            "Test case '{test_name}' should generate invalid data"
        );

        // Verify database rejects it
        let result = Word::create(pool.clone(), "en", invalid_data).await;
        assert!(
            result.is_err(),
            "Database should reject invalid data for test case '{test_name}'"
        );
    }
}

#[tokio::test]
async fn test_validator_edge_cases() {
    // Test specific edge cases that might cause issues

    // Lemma edge cases
    assert!(is_valid_lemma("a")); // Single character
    assert!(is_valid_lemma("123")); // Only numbers
    assert!(is_valid_lemma("a-b-c-d")); // Multiple hyphens
    assert!(is_valid_lemma("-test")); // Starting with hyphen is actually allowed
    assert!(is_valid_lemma("test-")); // Ending with hyphen is actually allowed
    assert!(is_valid_lemma("--test")); // Double hyphen is allowed

    // Definition edge cases
    assert!(is_valid_definition("a")); // Single character
    assert!(is_valid_definition("123")); // Only numbers
    assert!(!is_valid_definition("test@email.com")); // Contains @
    assert!(!is_valid_definition("@start")); // Starting with @
    assert!(!is_valid_definition("end@")); // Ending with @

    // Pronunciation edge cases
    assert!(is_valid_pronunciation("/a/")); // Single IPA character
    assert!(is_valid_pronunciation("/ˈa/")); // With stress mark
    assert!(is_valid_pronunciation("/aː/")); // With length mark
    assert!(!is_valid_pronunciation("/123/")); // Numbers not allowed
    assert!(!is_valid_pronunciation("/a b/")); // Spaces not allowed
    assert!(!is_valid_pronunciation("/a@b/")); // @ not allowed
}

#[tokio::test]
async fn test_unicode_handling() {
    // Test how validators handle various Unicode characters

    // Lemma allows accented characters (based on regex pattern)
    assert!(is_valid_lemma("café")); // Accented letters are allowed in lemmas
    assert!(is_valid_lemma("naïve")); // Diacritics are allowed in lemmas
    assert!(is_valid_lemma("über")); // Umlaut is allowed in lemmas

    // Definition should be permissive for international content
    assert!(is_valid_definition("café definition"));
    assert!(is_valid_definition("naïve description"));
    assert!(is_valid_definition("über description"));
    // Note: Chinese and Arabic characters are not in the supported Unicode ranges
    assert!(!is_valid_definition("中文 definition"));
    assert!(!is_valid_definition("العربية definition"));

    // Pronunciation should allow IPA characters
    assert!(is_valid_pronunciation("/kæˈfeɪ/")); // IPA for "café"
    assert!(is_valid_pronunciation("/naɪˈiːv/")); // IPA for "naïve"
    assert!(is_valid_pronunciation("/ˈuːbər/")); // IPA for "über"
}

#[tokio::test]
async fn test_regression_cases() {
    // Test specific cases that might have caused issues in the past

    // Test words that might look like email addresses
    assert!(!is_valid_lemma("user@domain"));
    assert!(!is_valid_definition("contact user@domain.com for info"));

    // Test definitions that might look like code
    assert!(!is_valid_definition("function test() { return @value; }")); // Contains @
    assert!(is_valid_definition("function test returns a value")); // Only allowed chars

    // Test pronunciations that might have invalid characters
    assert!(!is_valid_pronunciation("/test@invalid/"));
    assert!(!is_valid_pronunciation("/test123/"));
    assert!(!is_valid_pronunciation("/test invalid/"));

    // Test empty and whitespace-only inputs
    assert!(!is_valid_lemma(""));
    assert!(!is_valid_lemma(" ")); // Space not allowed in lemmas
    assert!(!is_valid_definition(""));
    assert!(is_valid_definition(" ")); // Single space should be valid for definitions
    assert!(!is_valid_pronunciation(""));
    assert!(!is_valid_pronunciation(" ")); // Space not valid for pronunciation
    assert!(!is_valid_pronunciation("/ /")); // Space between slashes not valid
}

#[tokio::test]
async fn test_validation_integration_with_crud() {
    let pool = create_test_db().await;

    // Test that CRUD operations work correctly with validator-verified data

    // CREATE with valid data
    let valid_word = sample_word();
    assert!(validate_test_word(&valid_word));
    let created = Word::create(pool.clone(), "en", valid_word).await.unwrap();
    let word_id = created[0].id();

    // READ the created word
    let read_words = Word::read(pool.clone(), "en", word_id).await.unwrap();
    assert_eq!(read_words.len(), 1);

    // Verify the read word still passes validation
    let read_word = &read_words[0];
    assert!(is_valid_lemma(read_word.word()));
    assert!(is_valid_definition(read_word.definition()));
    assert!(is_valid_pronunciation(read_word.pronunciation()));

    // UPDATE with valid data
    let update_data = sample_word_with_type("verb");
    assert!(validate_test_word(&update_data));
    let updated = Word::update(pool.clone(), "en", word_id, update_data)
        .await
        .unwrap();
    assert_eq!(updated.len(), 1);

    // Verify updated word still passes validation
    let updated_word = &updated[0];
    assert!(is_valid_lemma(updated_word.word()));
    assert!(is_valid_definition(updated_word.definition()));
    assert!(is_valid_pronunciation(updated_word.pronunciation()));

    // DELETE
    let delete_result = Word::delete(pool.clone(), "en", word_id).await;
    assert!(delete_result.is_ok());
}

#[tokio::test]
async fn test_stress_validation() {
    // Stress test validators with many different inputs

    let test_words = [
        "test",
        "hello",
        "world",
        "example",
        "sample",
        "demo",
        "check",
        "validate",
        "process",
        "function",
        "method",
        "system",
        "application",
        "database",
        "server",
        "client",
        "request",
        "response",
        "data",
    ];

    let test_definitions = [
        "a test definition",
        "example description for testing",
        "sample definition used in validation",
        "demonstration of definition validation",
        "testing the definition validation system",
    ];

    let test_pronunciations = [
        "/tɛst/",
        "/həˈloʊ/",
        "/wɜːrld/",
        "/ˈsæmpəl/",
        "/ˈdiːmoʊ/",
        "/ʧɛk/",
        "/ˈproʊsɛs/",
        "/ˈfʌŋkʃən/",
    ];

    // Test all combinations
    for word in test_words {
        assert!(is_valid_lemma(word), "Word '{word}' should be valid");
    }

    for definition in test_definitions {
        assert!(
            is_valid_definition(definition),
            "Definition '{definition}' should be valid"
        );
    }

    for pronunciation in test_pronunciations {
        assert!(
            is_valid_pronunciation(pronunciation),
            "Pronunciation '{pronunciation}' should be valid"
        );
    }
}

#[tokio::test]
async fn test_validator_thread_safety() {
    // Test that validators work correctly in concurrent scenarios

    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    // Spawn multiple tasks that use validators concurrently
    for i in 0..10 {
        join_set.spawn(async move {
            let word = format!("concurrent{i}");
            let definition = format!("concurrent definition {i}");
            let pronunciation = "/kənˈkʌrənt/";

            // All should be valid
            assert!(is_valid_lemma(&word));
            assert!(is_valid_definition(&definition));
            assert!(is_valid_pronunciation(pronunciation));

            // Test invalid cases too
            assert!(!is_valid_lemma(&format!("invalid word {i}")));
            assert!(!is_valid_definition(&format!("invalid@definition{i}.com")));
            assert!(!is_valid_pronunciation(&format!("invalid{i}")));
        });
    }

    // Wait for all tasks to complete
    while let Some(result) = join_set.join_next().await {
        result.unwrap(); // Panic if any task failed
    }
}
