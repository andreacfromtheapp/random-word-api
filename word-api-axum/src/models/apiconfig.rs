//! Server configuration management with multi-source support
//!
//! Supports configuration via CLI arguments, TOML files, and environment variables.
//! Includes utilities for generating default configuration files.

// Application configuration
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, path::PathBuf};

use crate::cli::Cli;

/// Main API configuration structure containing all runtime settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub address: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub openapi: OpenApiDocs,
}

/// File format types for configuration file generation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileKind {
    /// TOML configuration file format
    Toml,
    /// Environment variable file format
    EnvFile,
}

impl ApiConfig {
    /// Creates a new ApiConfig instance with specified values
    pub fn new(address: IpAddr, port: u16, database_url: String, openapi: OpenApiDocs) -> Self {
        Self {
            address,
            port,
            database_url,
            openapi,
        }
    }

    /// Generates a configuration file with default values
    pub fn gen_file(file: &PathBuf, kind: FileKind) -> Result<(), anyhow::Error> {
        use std::fs::File;
        use std::io::prelude::*;

        // set default config values
        let default_configs = Self::default();

        let what_file = match kind {
            FileKind::Toml => toml::to_string(&default_configs)?,
            FileKind::EnvFile => Self::to_string(&default_configs),
        };

        // create the default file
        let mut buffer = File::create(file)?;
        // write all lines from the above steps
        buffer.write_all(what_file.as_bytes())?;

        println!("configuration file '{file:?}' created successfully");

        Ok(())
    }

    /// Creates ApiConfig from CLI arguments with source precedence handling
    ///
    /// Configuration source priority (highest to lowest):
    /// 1. Environment file (--env-file flag)
    /// 2. TOML configuration file (--config flag)
    /// 3. Direct command-line arguments (default)
    pub fn from_cli(cli: &Cli) -> Result<Self, anyhow::Error> {
        // if --env-file was used
        if let Some(file) = &cli.cfg.env_file {
            Self::from_env_file(file)
        // if --config was used
        } else if let Some(file) = &cli.cfg.config {
            Self::from_config_file(file)
        // if positional parameters were used
        } else {
            Self::from_cli_args(cli)
        }
    }

    /// Creates ApiConfig from environment variable file
    pub fn from_env_file(file: &PathBuf) -> Result<Self, anyhow::Error> {
        use std::str::FromStr;

        // get all environment variable from the environment file
        dotenvy::from_filename_override(file)?;

        // set the variables as needed
        Ok(Self::new(
            IpAddr::from_str(dotenvy::var("BIND_ADDR")?.trim())?,
            u16::from_str(dotenvy::var("BIND_PORT")?.trim())?,
            dotenvy::var("DATABASE_URL")?.trim().to_owned(),
            OpenApiDocs::new(
                bool::from_str(dotenvy::var("ENABLE_SWAGGER_UI")?.trim())?,
                bool::from_str(dotenvy::var("ENABLE_REDOC")?.trim())?,
                bool::from_str(dotenvy::var("ENABLE_SCALAR")?.trim())?,
                bool::from_str(dotenvy::var("ENABLE_RAPIDOC")?.trim())?,
            ),
        ))
    }

    /// Creates ApiConfig from TOML configuration file
    pub fn from_config_file(file: &PathBuf) -> Result<Self, anyhow::Error> {
        // read the config file line by line and store it in a String
        let file_content = std::fs::read(file)?
            .iter()
            .map(|c| *c as char)
            .collect::<String>();

        // parse the configuration String and store in model Struct
        let my_configs: Self = toml::from_str(&file_content)?;

        // set the variables as needed
        Ok(Self::new(
            my_configs.address,
            my_configs.port,
            my_configs.database_url.clone(),
            OpenApiDocs::new(
                my_configs.openapi.enable_swagger_ui,
                my_configs.openapi.enable_redoc,
                my_configs.openapi.enable_scalar,
                my_configs.openapi.enable_rapidoc,
            ),
        ))
    }

    /// Creates ApiConfig directly from command-line arguments
    pub fn from_cli_args(cli: &Cli) -> Result<Self, anyhow::Error> {
        // set the variables as needed
        Ok(Self::new(
            cli.arg.address,
            cli.arg.port,
            cli.arg.database_url.clone(),
            OpenApiDocs::new(
                cli.arg.with_swagger_ui,
                cli.arg.with_redoc,
                cli.arg.with_scalar,
                cli.arg.with_rapidoc,
            ),
        ))
    }
}

/// Formats ApiConfig as environment variable file content
impl fmt::Display for ApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BIND_ADDR=\"{}\"\nBIND_PORT={}\nDATABASE_URL={}\n\n{}",
            self.address, self.port, self.database_url, self.openapi
        )
    }
}

/// Provides default configuration values for development and testing
impl Default for ApiConfig {
    fn default() -> Self {
        use std::str::FromStr;

        ApiConfig {
            address: IpAddr::from_str("0.0.0.0").unwrap(),
            port: u16::from_str("3000").unwrap(),
            database_url: "sqlite:random-words.db".to_string(),
            openapi: OpenApiDocs::default(),
        }
    }
}

/// OpenAPI documentation interface configuration
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct OpenApiDocs {
    pub enable_swagger_ui: bool,
    pub enable_redoc: bool,
    pub enable_scalar: bool,
    pub enable_rapidoc: bool,
}

impl OpenApiDocs {
    /// Creates a new OpenApiDocs configuration with specified interface settings
    pub fn new(
        enable_swagger_ui: bool,
        enable_redoc: bool,
        enable_scalar: bool,
        enable_rapidoc: bool,
    ) -> Self {
        Self {
            enable_swagger_ui,
            enable_redoc,
            enable_scalar,
            enable_rapidoc,
        }
    }
}

/// Formats OpenApiDocs as environment variable section
impl fmt::Display for OpenApiDocs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# OpenAPI Docs\nENABLE_SWAGGER_UI={}\nENABLE_REDOC={}\nENABLE_SCALAR={}\nENABLE_RAPIDOC={}\n",
            self.enable_swagger_ui, self.enable_redoc, self.enable_scalar, self.enable_rapidoc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_api_config_new() {
        let address = IpAddr::from_str("127.0.0.1").unwrap();
        let openapi = OpenApiDocs::new(true, false, true, false);
        let config = ApiConfig::new(address, 8080, "sqlite:test.db".to_string(), openapi);

        assert_eq!(config.address, address);
        assert_eq!(config.port, 8080);
        assert_eq!(config.database_url, "sqlite:test.db");
        assert!(config.openapi.enable_swagger_ui);
        assert!(!config.openapi.enable_redoc);
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();

        assert_eq!(config.address, IpAddr::from_str("0.0.0.0").unwrap());
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "sqlite:random-words.db");
        assert!(!config.openapi.enable_swagger_ui);
        assert!(!config.openapi.enable_redoc);
        assert!(!config.openapi.enable_scalar);
        assert!(!config.openapi.enable_rapidoc);
    }

    #[test]
    fn test_openapi_docs_new() {
        let docs = OpenApiDocs::new(true, false, true, false);

        assert!(docs.enable_swagger_ui);
        assert!(!docs.enable_redoc);
        assert!(docs.enable_scalar);
        assert!(!docs.enable_rapidoc);
    }

    #[test]
    fn test_openapi_docs_default() {
        let docs = OpenApiDocs::default();

        assert!(!docs.enable_swagger_ui);
        assert!(!docs.enable_redoc);
        assert!(!docs.enable_scalar);
        assert!(!docs.enable_rapidoc);
    }

    #[test]
    fn test_api_config_display() {
        let address = IpAddr::from_str("192.168.1.1").unwrap();
        let openapi = OpenApiDocs::new(true, false, true, false);
        let config = ApiConfig::new(address, 9000, "sqlite:display_test.db".to_string(), openapi);

        let output = format!("{config}");
        assert!(output.contains("BIND_ADDR=\"192.168.1.1\""));
        assert!(output.contains("BIND_PORT=9000"));
        assert!(output.contains("DATABASE_URL=sqlite:display_test.db"));
        assert!(output.contains("ENABLE_SWAGGER_UI=true"));
        assert!(output.contains("ENABLE_REDOC=false"));
        assert!(output.contains("ENABLE_SCALAR=true"));
        assert!(output.contains("ENABLE_RAPIDOC=false"));
    }

    #[test]
    fn test_openapi_docs_display() {
        let docs = OpenApiDocs::new(false, true, false, true);
        let output = format!("{docs}");

        assert!(output.contains("# OpenAPI Docs"));
        assert!(output.contains("ENABLE_SWAGGER_UI=false"));
        assert!(output.contains("ENABLE_REDOC=true"));
        assert!(output.contains("ENABLE_SCALAR=false"));
        assert!(output.contains("ENABLE_RAPIDOC=true"));
    }

    #[test]
    fn test_file_kind_variants() {
        // Test that FileKind variants exist and can be created
        let _toml = FileKind::Toml;
        let _env = FileKind::EnvFile;

        // Test Debug implementation
        assert_eq!(format!("{:?}", FileKind::Toml), "Toml");
        assert_eq!(format!("{:?}", FileKind::EnvFile), "EnvFile");
    }
}
