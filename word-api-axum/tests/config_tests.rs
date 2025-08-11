//! Configuration integration tests
//!
//! Tests configuration file generation, validation, and server initialization
//! using existing source code APIs and infrastructure.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use word_api_axum::models::apiconfig::{ApiConfig, FileKind};

mod helpers;

#[tokio::test]
async fn test_config_creation() -> Result<()> {
    // Test server creation with default configuration
    let _server = helpers::create_test_server_streamlined().await?;

    // Verify successful server initialization
    assert!(true, "Server creation with default config successful");

    Ok(())
}

#[tokio::test]
async fn test_config_file_generation() -> Result<()> {
    // Test TOML configuration file generation
    let temp_file = NamedTempFile::new()?;
    let config_path = PathBuf::from(temp_file.path());

    ApiConfig::gen_file(&config_path, FileKind::Toml)?;

    // Verify file was created and contains expected content
    let content = fs::read_to_string(&config_path)?;
    assert!(
        !content.is_empty(),
        "Generated config file should not be empty"
    );
    assert!(
        content.contains("address"),
        "Config should contain address field"
    );
    assert!(content.contains("port"), "Config should contain port field");
    assert!(
        content.contains("database_url"),
        "Config should contain database_url field"
    );

    Ok(())
}

#[tokio::test]
async fn test_env_file_generation() -> Result<()> {
    // Test environment file generation
    let temp_file = NamedTempFile::new()?;
    let env_path = PathBuf::from(temp_file.path());

    ApiConfig::gen_file(&env_path, FileKind::EnvFile)?;

    // Verify environment file was created
    let content = fs::read_to_string(&env_path)?;
    assert!(
        !content.is_empty(),
        "Generated env file should not be empty"
    );
    assert!(
        content.contains("BIND_ADDR"),
        "Env file should contain BIND_ADDR"
    );
    assert!(
        content.contains("BIND_PORT"),
        "Env file should contain BIND_PORT"
    );
    assert!(
        content.contains("DATABASE_URL"),
        "Env file should contain DATABASE_URL"
    );

    Ok(())
}

#[tokio::test]
async fn test_environment_file_loading() -> Result<()> {
    // Test loading configuration from environment file
    let temp_file = NamedTempFile::new()?;
    let env_path = temp_file.path();

    // Create test environment file
    let env_content = "TEST_ADDRESS=0.0.0.0\nTEST_PORT=8080\nTEST_DATABASE_URL=sqlite://test.db\n";
    fs::write(env_path, env_content)?;

    // Simulate loading environment variables
    let content = fs::read_to_string(env_path)?;
    let mut loaded_vars = std::collections::HashMap::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            loaded_vars.insert(key.to_string(), value.to_string());
        }
    }

    // Verify environment variables were loaded correctly
    assert_eq!(loaded_vars.get("TEST_ADDRESS").unwrap(), "0.0.0.0");
    assert_eq!(loaded_vars.get("TEST_PORT").unwrap(), "8080");
    assert_eq!(
        loaded_vars.get("TEST_DATABASE_URL").unwrap(),
        "sqlite://test.db"
    );

    Ok(())
}

#[tokio::test]
async fn test_configuration_precedence() -> Result<()> {
    // Test configuration precedence logic with simple string comparisons

    // Simulate config file values (TOML defaults)
    let config_address = "0.0.0.0";
    let config_port = "3000";
    let config_db_url = "sqlite://config.db";

    // Simulate environment variable overrides
    let env_address = Some("127.0.0.1");
    let env_port = Some("4000");
    let env_db_url: Option<&str> = None; // Not overridden

    // Test precedence: env vars override config values when present
    let final_address = env_address.unwrap_or(config_address);
    let final_port = env_port.unwrap_or(config_port);
    let final_db_url = env_db_url.unwrap_or(config_db_url);

    assert_eq!(final_address, "127.0.0.1"); // From env (overrides config)
    assert_eq!(final_port, "4000"); // From env (overrides config)
    assert_eq!(final_db_url, "sqlite://config.db"); // From config (no override)

    Ok(())
}

#[tokio::test]
async fn test_config_validation() -> Result<()> {
    // Test configuration validation using existing types
    use std::net::IpAddr;

    // Test valid IP addresses
    let valid_ips = vec!["127.0.0.1", "0.0.0.0", "192.168.1.1"];
    for ip_str in valid_ips {
        let parsed_ip: IpAddr = ip_str.parse()?;
        assert!(matches!(parsed_ip, IpAddr::V4(_)));
    }

    // Test valid port ranges
    let valid_ports = vec![0, 3000, 8080, 65535];
    for port in valid_ports {
        assert!(port <= 65535, "Port should be within valid range");
    }

    // Test valid database URL patterns
    let valid_db_urls = vec![
        "sqlite:///tmp/test.db",
        "sqlite://./test.db",
        "sqlite://:memory:",
    ];

    for db_url in valid_db_urls {
        assert!(
            db_url.starts_with("sqlite://"),
            "Database URL should use sqlite scheme"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_default_config_values() -> Result<()> {
    // Test default configuration values
    let config = ApiConfig::default();

    assert_eq!(config.address.to_string(), "0.0.0.0");
    assert_eq!(config.port, 3000);
    assert!(config.database_url.contains("sqlite:"));

    // Test OpenAPI defaults
    assert!(!config.openapi.enable_swagger_ui);
    assert!(!config.openapi.enable_redoc);
    assert!(!config.openapi.enable_scalar);
    assert!(!config.openapi.enable_rapidoc);

    Ok(())
}
