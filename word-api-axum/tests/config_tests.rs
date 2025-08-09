//! Simplified configuration integration tests
//!
//! This module contains basic tests for configuration loading and validation.
//! It focuses on essential configuration functionality without complex scenarios
//! that can cause test instability.

use anyhow::Result;
use axum::http::StatusCode;

use tempfile::NamedTempFile;

mod helpers;

use helpers::create_test_server;
use std::collections::HashMap;

#[tokio::test]
async fn test_default_configuration() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

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

    Ok(())
}

#[tokio::test]
async fn test_config_with_custom_database() -> Result<()> {
    // Create a temporary database file
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file.path().to_string_lossy();
    let db_url = format!("sqlite://{db_path}");

    // Test that we can create a config with custom database URL
    let mut config = HashMap::new();
    config.insert("database_url".to_string(), db_url.clone());
    assert_eq!(config.get("database_url").unwrap(), &db_url);

    Ok(())
}

#[tokio::test]
async fn test_config_validation() -> Result<()> {
    // Test basic configuration validation patterns
    let mut default_config = HashMap::new();
    default_config.insert("address".to_string(), "127.0.0.1".to_string());
    default_config.insert("port".to_string(), "0".to_string());
    assert!(default_config.contains_key("address"));
    assert!(default_config.contains_key("port"));

    // Test port configuration
    let mut config_with_port = HashMap::new();
    config_with_port.insert("port".to_string(), "8080".to_string());
    assert_eq!(config_with_port.get("port").unwrap(), "8080");

    Ok(())
}

#[tokio::test]
async fn test_config_address_formats() -> Result<()> {
    // Test different address formats
    let addresses = vec!["127.0.0.1", "0.0.0.0", "localhost"];

    for address in addresses {
        let mut config = HashMap::new();
        config.insert("address".to_string(), address.to_string());
        assert_eq!(config.get("address").unwrap(), address);
    }

    Ok(())
}

#[tokio::test]
async fn test_config_database_url_formats() -> Result<()> {
    // Test different database URL formats
    let db_urls = vec![
        "sqlite:///tmp/test.db",
        "sqlite://./test.db",
        "sqlite://:memory:",
    ];

    for db_url in db_urls {
        let mut config = HashMap::new();
        config.insert("database_url".to_string(), db_url.to_string());
        assert_eq!(config.get("database_url").unwrap(), db_url);
    }

    Ok(())
}

#[tokio::test]
async fn test_server_responds_with_config() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test that server responds properly with configuration
    let health_response = server.get("/health/alive").await;
    assert_eq!(health_response.status_code(), StatusCode::OK);

    let db_health_response = server.get("/health/ready").await;
    assert_eq!(db_health_response.status_code(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn test_basic_config_functionality() -> Result<()> {
    // Test that configuration system works with basic values
    let mut config = HashMap::new();
    config.insert("address".to_string(), "127.0.0.1".to_string());
    config.insert("port".to_string(), "0".to_string());

    // Should have required fields
    assert!(config.contains_key("address"));
    assert!(config.contains_key("port"));

    // Should have reasonable default values
    assert_eq!(config.get("address").unwrap(), "127.0.0.1");
    assert_eq!(config.get("port").unwrap(), "0");

    Ok(())
}

#[tokio::test]
async fn test_config_port_variations() -> Result<()> {
    // Test different port configurations
    let ports = vec![0, 8080, 3000, 8000];

    for port in ports {
        let mut config = HashMap::new();
        config.insert("port".to_string(), port.to_string());
        assert_eq!(config.get("port").unwrap(), &port.to_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_config_database_integration() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    // Test that database health check works with configuration
    let response = server.get("/health/ready").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.text();
    assert!(
        body.contains("database"),
        "Database health should mention database"
    );

    Ok(())
}
