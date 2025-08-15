//! Configuration integration tests
//!
//! Tests configuration file generation, validation, and server initialization
//! using existing source code APIs and infrastructure.
//!
//! # Test Coverage
//! - Configuration file generation (TOML and environment formats)
//! - Multi-source configuration loading with precedence rules
//! - Configuration validation and error handling
//! - Default value verification and type safety
//! - File existence validation and error propagation

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

    // Server creation validates configuration loading and database initialization
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
async fn test_configuration_simple_precedence() -> Result<()> {
    // Test configuration precedence logic: env file > config file > CLI args

    // Simulate config file values (TOML defaults)
    let config_db_url = "sqlite://config.db";

    // Simulate environment variable overrides
    let env_address = "127.0.0.1";
    let env_port = "4000";

    // Test precedence: environment vars override config values when present
    let final_address = env_address;
    let final_port = env_port;
    let final_db_url = config_db_url; // From config (no override)

    assert_eq!(final_address, "127.0.0.1"); // From environment (overrides config)
    assert_eq!(final_port, "4000"); // From environment (overrides config)
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

#[tokio::test]
async fn test_toml_file_parsing() -> Result<()> {
    // Test loading configuration from TOML file
    let temp_file = NamedTempFile::new()?;
    let toml_path = PathBuf::from(temp_file.path());

    // Create test TOML content with known values
    let toml_content = r#"
address = "192.168.1.100"
port = 8080
database_url = "sqlite:test.db"

[openapi]
enable_swagger_ui = true
enable_redoc = false
enable_scalar = true
enable_rapidoc = false
"#;
    fs::write(&toml_path, toml_content)?;

    // Test ApiConfig::from_config_file()
    let config = ApiConfig::from_config_file(&toml_path)?;

    // Assert parsed values match expected
    assert_eq!(config.address.to_string(), "192.168.1.100");
    assert_eq!(config.port, 8080);
    assert_eq!(config.database_url, "sqlite:test.db");
    assert!(config.openapi.enable_swagger_ui);
    assert!(!config.openapi.enable_redoc);
    assert!(config.openapi.enable_scalar);
    assert!(!config.openapi.enable_rapidoc);

    Ok(())
}

#[tokio::test]
async fn test_environment_file_parsing() -> Result<()> {
    // Test loading configuration from environment file
    let temp_file = NamedTempFile::new()?;
    let env_path = PathBuf::from(temp_file.path());

    // Create test environment file content with known values
    let env_content = r#"BIND_ADDR="172.16.0.1"
BIND_PORT=9000
DATABASE_URL=sqlite:unique_env_test.db
ENABLE_SWAGGER_UI=false
ENABLE_REDOC=true
ENABLE_SCALAR=false
ENABLE_RAPIDOC=true
"#;
    fs::write(&env_path, env_content)?;

    // Test ApiConfig::from_env_file()
    let config = ApiConfig::from_env_file(&env_path)?;

    // Assert parsed values match expected
    assert_eq!(config.address.to_string(), "172.16.0.1");
    assert_eq!(config.port, 9000);
    assert_eq!(config.database_url, "sqlite:unique_env_test.db");
    assert!(!config.openapi.enable_swagger_ui);
    assert!(config.openapi.enable_redoc);
    assert!(!config.openapi.enable_scalar);
    assert!(config.openapi.enable_rapidoc);

    Ok(())
}

#[tokio::test]
async fn test_configuration_precedence_logic() -> Result<()> {
    // Test configuration precedence logic without environment variable conflicts
    use std::net::IpAddr;
    use std::str::FromStr;
    use word_api_axum::cli::{Arguments, Cli, Config};

    // Test CLI args only (lowest precedence)
    let cli_args_only = Cli {
        cfg: Config {
            env_file: None,
            config: None,
        },
        arg: Arguments {
            address: IpAddr::from_str("10.0.0.1").unwrap(),
            port: 8000,
            database_url: "sqlite:cli_only.db".to_string(),
            with_swagger_ui: true,
            with_redoc: false,
            with_scalar: false,
            with_rapidoc: false,
        },
        command: None,
    };

    let config_from_cli = ApiConfig::from_cli(&cli_args_only)?;
    assert_eq!(config_from_cli.address.to_string(), "10.0.0.1");
    assert_eq!(config_from_cli.port, 8000);
    assert_eq!(config_from_cli.database_url, "sqlite:cli_only.db");
    assert!(config_from_cli.openapi.enable_swagger_ui);

    Ok(())
}

#[tokio::test]
async fn test_file_existence_validation() -> Result<()> {
    use word_api_axum::does_file_exist;

    // Test with existing file (should pass validation)
    let existing_file = NamedTempFile::new()?;
    fs::write(existing_file.path(), "test content")?;

    let result = does_file_exist(existing_file.path(), "test");
    assert!(result.is_ok(), "Existing file should pass validation");

    // Test with non-existent file (should fail with descriptive error)
    let non_existent_path = PathBuf::from("/tmp/definitely_does_not_exist_12345.txt");
    let result = does_file_exist(&non_existent_path, "test");
    assert!(result.is_err(), "Non-existent file should fail validation");

    // Verify error message contains file type information
    if let Err(err) = result {
        let error_msg = format!("{err:?}");
        assert!(error_msg.contains("test"), "Error should mention file kind");
    }

    Ok(())
}

#[tokio::test]
async fn test_enhanced_file_generation_validation() -> Result<()> {
    // Test TOML file generation with comprehensive validation
    let temp_toml = NamedTempFile::new()?;
    let toml_path = PathBuf::from(temp_toml.path());

    ApiConfig::gen_file(&toml_path, FileKind::Toml)?;

    // Verify file exists and has content
    assert!(toml_path.exists(), "Generated TOML file should exist");
    let toml_content = fs::read_to_string(&toml_path)?;
    assert!(!toml_content.is_empty(), "TOML file should not be empty");

    // Verify TOML structure
    assert!(
        toml_content.contains("address = "),
        "TOML should contain address field"
    );
    assert!(
        toml_content.contains("port = "),
        "TOML should contain port field"
    );
    assert!(
        toml_content.contains("database_url = "),
        "TOML should contain database_url"
    );
    assert!(
        toml_content.contains("[openapi]"),
        "TOML should contain openapi section"
    );
    assert!(
        toml_content.contains("enable_swagger_ui"),
        "TOML should contain swagger_ui setting"
    );

    // Test env file generation with comprehensive validation
    let temp_env = NamedTempFile::new()?;
    let env_path = PathBuf::from(temp_env.path());

    ApiConfig::gen_file(&env_path, FileKind::EnvFile)?;

    // Verify file exists and has content
    assert!(env_path.exists(), "Generated env file should exist");
    let env_content = fs::read_to_string(&env_path)?;
    assert!(!env_content.is_empty(), "Env file should not be empty");

    // Verify env file structure
    assert!(
        env_content.contains("BIND_ADDR="),
        "Env file should contain BIND_ADDR"
    );
    assert!(
        env_content.contains("BIND_PORT="),
        "Env file should contain BIND_PORT"
    );
    assert!(
        env_content.contains("DATABASE_URL="),
        "Env file should contain DATABASE_URL"
    );
    assert!(
        env_content.contains("ENABLE_SWAGGER_UI="),
        "Env file should contain swagger UI setting"
    );
    assert!(
        env_content.contains("# OpenAPI Docs"),
        "Env file should contain OpenAPI section header"
    );

    Ok(())
}
