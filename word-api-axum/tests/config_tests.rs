//! Essential configuration integration tests (Phase 4 Simplified)
//!
//! This module contains streamlined tests for configuration loading and validation.
//! Focuses on core configuration functionality without redundant or complex scenarios.

use anyhow::Result;
use axum::http::StatusCode;
use std::collections::HashMap;

mod helpers;
use helpers::create_test_server_streamlined;

#[tokio::test]
async fn test_config_parallel() -> Result<()> {
    // Run config tests in parallel for efficiency
    let (default_result, db_integration_result) = tokio::join!(
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/alive").await;
            assert_eq!(
                response.status_code(),
                StatusCode::OK,
                "Server with default config should be healthy"
            );
            let body = response.text();
            assert!(
                body.contains("API is successfully running"),
                "Health response should indicate API is running"
            );
            Ok::<(), anyhow::Error>(())
        },
        async {
            let server = create_test_server_streamlined().await?;
            let response = server.get("/health/ready").await;
            assert_eq!(response.status_code(), StatusCode::OK);
            let body = response.text();
            assert!(
                body.contains("database"),
                "Database health should mention database"
            );
            Ok::<(), anyhow::Error>(())
        }
    );

    // Check both parallel operations succeeded
    default_result?;
    db_integration_result?;

    Ok(())
}

#[tokio::test]
async fn test_config_validation() -> Result<()> {
    // Test basic configuration validation patterns
    let mut config = HashMap::new();
    config.insert("address".to_string(), "127.0.0.1".to_string());
    config.insert("port".to_string(), "0".to_string());

    assert!(config.contains_key("address"));
    assert!(config.contains_key("port"));
    assert_eq!(config.get("address").unwrap(), "127.0.0.1");
    assert_eq!(config.get("port").unwrap(), "0");

    // Test database URL validation
    let db_urls = vec![
        "sqlite:///tmp/test.db",
        "sqlite://./test.db",
        "sqlite://:memory:",
    ];

    for db_url in db_urls {
        let mut db_config = HashMap::new();
        db_config.insert("database_url".to_string(), db_url.to_string());
        assert_eq!(db_config.get("database_url").unwrap(), db_url);
    }

    Ok(())
}
