//! Configuration integration tests
//!
//! This module tests configuration loading from various sources including
//! CLI arguments, TOML files, and environment files with proper precedence
//! handling and error scenarios.

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

mod common;
use common::test_config;
use random_word_api::cli::{Cli, Commands};
use random_word_api::models::apiconfig::{ApiConfig, FileKind};
use serial_test::serial;

#[test]
fn test_config_file_generation() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("test_config.toml");

    let result = ApiConfig::gen_file(&config_file, FileKind::Toml);
    assert!(result.is_ok());

    // Verify file was created
    assert!(config_file.exists());

    // Verify file content
    let content = std::fs::read_to_string(&config_file).unwrap();
    assert!(content.contains("address"));
    assert!(content.contains("port"));
    assert!(content.contains("database_url"));
    assert!(content.contains("[openapi]"));
    assert!(content.contains("enable_swagger_ui"));
}

#[test]
fn test_env_file_generation() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    let result = ApiConfig::gen_file(&env_file, FileKind::EnvFile);
    assert!(result.is_ok());

    // Verify file was created
    assert!(env_file.exists());

    // Verify file content
    let content = std::fs::read_to_string(&env_file).unwrap();
    assert!(content.contains("BIND_ADDR="));
    assert!(content.contains("BIND_PORT="));
    assert!(content.contains("DATABASE_URL="));
    assert!(content.contains("ENABLE_SWAGGER_UI="));
    assert!(content.contains("# OpenAPI Docs"));
}

#[test]
fn test_config_from_toml_file() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("test_config.toml");

    // Create a test TOML file
    let toml_content = r#"
address = "192.168.1.1"
port = 8080
database_url = "sqlite:test.db"

[openapi]
enable_swagger_ui = true
enable_redoc = false
enable_scalar = true
enable_rapidoc = false
"#;

    let mut file = File::create(&config_file).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    // Test loading the config
    let config = ApiConfig::from_config_file(&config_file).unwrap();

    assert_eq!(config.address.to_string(), "192.168.1.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.database_url, "sqlite:test.db");
    assert!(config.openapi.enable_swagger_ui);
    assert!(!config.openapi.enable_redoc);
    assert!(config.openapi.enable_scalar);
    assert!(!config.openapi.enable_rapidoc);
}

#[test]
fn test_config_from_invalid_toml_file() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("invalid_config.toml");

    // Create an invalid TOML file
    let invalid_toml = r#"
address = "192.168.1.1
port = invalid_port
database_url =
"#;

    let mut file = File::create(&config_file).unwrap();
    file.write_all(invalid_toml.as_bytes()).unwrap();

    // Test loading should fail
    let result = ApiConfig::from_config_file(&config_file);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_config_from_env_file() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    // Create a test environment file
    let env_content = "BIND_ADDR=127.0.0.1
BIND_PORT=9000
DATABASE_URL=sqlite:env_test.db
ENABLE_SWAGGER_UI=true
ENABLE_REDOC=true
ENABLE_SCALAR=false
ENABLE_RAPIDOC=false";

    let mut file = File::create(&env_file).unwrap();
    file.write_all(env_content.as_bytes()).unwrap();

    // Test loading the config
    let config = ApiConfig::from_env_file(&env_file).unwrap();

    assert_eq!(config.address.to_string(), "127.0.0.1");
    assert_eq!(config.port, 9000);
    assert_eq!(config.database_url, "sqlite:env_test.db");
    assert!(config.openapi.enable_swagger_ui);
    assert!(config.openapi.enable_redoc);
    assert!(!config.openapi.enable_scalar);
    assert!(!config.openapi.enable_rapidoc);
}

#[test]
#[serial]
fn test_config_from_missing_env_file() {
    let temp_dir = tempdir().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.env");

    // Test loading should fail
    let result = ApiConfig::from_env_file(&nonexistent_file);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_config_from_invalid_env_file() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("invalid.env");

    // Create an invalid environment file
    let invalid_env = "BIND_ADDR=127.0.0.1
BIND_PORT=not_a_number
DATABASE_URL=sqlite:test.db
ENABLE_SWAGGER_UI=not_a_boolean";

    let mut file = File::create(&env_file).unwrap();
    file.write_all(invalid_env.as_bytes()).unwrap();

    // Test loading should fail
    let result = ApiConfig::from_env_file(&env_file);
    assert!(result.is_err());
}

#[test]
fn test_cli_config_precedence() {
    // Test that CLI arguments work when no files are specified
    use clap::Parser;

    // Simulate CLI args
    let args = vec![
        "program",
        "--address",
        "10.0.0.1",
        "--port",
        "4000",
        "--database-url",
        "sqlite:cli_test.db",
        "--with-swagger-ui",
    ];

    let cli = Cli::try_parse_from(args).unwrap();
    let config = ApiConfig::from_cli(&cli).unwrap();

    assert_eq!(config.address.to_string(), "10.0.0.1");
    assert_eq!(config.port, 4000);
    assert_eq!(config.database_url, "sqlite:cli_test.db");
    assert!(config.openapi.enable_swagger_ui);
    assert!(!config.openapi.enable_redoc);
}

#[test]
fn test_cli_validation_invalid_port() {
    use clap::Parser;

    // Test invalid port (too high)
    let args = vec!["program", "--port", "99999"];

    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_validation_invalid_address() {
    use clap::Parser;

    // Test invalid IP address
    let args = vec!["program", "--address", "300.300.300.300"];

    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_gen_config_command() {
    use clap::Parser;

    let args = vec!["program", "gen-config", "custom_config.toml"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::GenConfig { file_name }) => {
            assert_eq!(file_name.unwrap().to_string_lossy(), "custom_config.toml");
        }
        _ => panic!("Expected GenConfig command"),
    }
}

#[test]
fn test_cli_gen_env_file_command() {
    use clap::Parser;

    let args = vec!["program", "gen-env-file", "custom.env"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Some(Commands::GenEnvFile { file_name }) => {
            assert_eq!(file_name.unwrap().to_string_lossy(), "custom.env");
        }
        _ => panic!("Expected GenEnvFile command"),
    }
}

#[test]
fn test_config_mutual_exclusion() {
    use clap::Parser;

    // Test that config file and direct args are mutually exclusive
    let args = vec!["program", "--config", "config.toml", "--port", "8080"];

    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // Should fail due to mutual exclusion
}

#[test]
fn test_config_serialization() {
    let config = test_config();

    // Test TOML serialization
    let toml_result = toml::to_string(&config);
    assert!(toml_result.is_ok());

    let toml_string = toml_result.unwrap();
    assert!(toml_string.contains("address"));
    assert!(toml_string.contains("port"));
    assert!(toml_string.contains("[openapi]"));
}

#[test]
fn test_config_display_formatting() {
    let config = test_config();
    let display_output = format!("{config}");

    // Should format as environment variables
    assert!(display_output.contains("BIND_ADDR="));
    assert!(display_output.contains("BIND_PORT="));
    assert!(display_output.contains("DATABASE_URL="));
    assert!(display_output.contains("# OpenAPI Docs"));
    assert!(display_output.contains("ENABLE_SWAGGER_UI="));
}

#[test]
fn test_config_clone() {
    let config1 = test_config();
    let config2 = config1.clone();

    assert_eq!(config1.address, config2.address);
    assert_eq!(config1.port, config2.port);
    assert_eq!(config1.database_url, config2.database_url);
    assert_eq!(
        config1.openapi.enable_swagger_ui,
        config2.openapi.enable_swagger_ui
    );
}

#[test]
fn test_missing_config_file() {
    let temp_dir = tempdir().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.toml");

    let result = ApiConfig::from_config_file(&nonexistent_file);
    assert!(result.is_err());
}

#[test]
fn test_config_with_comments() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("commented_config.toml");

    // Create a TOML file with comments
    let toml_content = r#"
# Server configuration
address = "0.0.0.0"  # Bind to all interfaces
port = 3000          # Default port

# Database settings
database_url = "sqlite:words.db"

[openapi]
# Documentation settings
enable_swagger_ui = true   # Enable interactive docs
enable_redoc = false       # Disable redoc
enable_scalar = false      # Disable scalar
enable_rapidoc = false     # Disable rapidoc
"#;

    let mut file = File::create(&config_file).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    // Should parse successfully despite comments
    let config = ApiConfig::from_config_file(&config_file).unwrap();
    assert_eq!(config.port, 3000);
    assert!(config.openapi.enable_swagger_ui);
}
