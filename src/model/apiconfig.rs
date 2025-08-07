// Application configuration
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr};

use crate::cli::Cli;

/// Define config.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub address: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub openapi: OpenApiDocs,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileKind {
    Toml,
    EnvFile,
}

impl ApiConfig {
    pub fn new(address: IpAddr, port: u16, database_url: String, openapi: OpenApiDocs) -> Self {
        Self {
            address,
            port,
            database_url,
            openapi,
        }
    }

    /// Helper to create a file with default values
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

    /// Parse Cli arguments to construct `address`, `port`, and `database-url`
    /// Accepts `BIND_ADDR`, `BIND_PORT`, and `DATABASE_URL` from an `.env` file.
    pub fn from_cli(cli: &Cli) -> Result<Self, anyhow::Error> {
        let apiconfig: Self;

        // if --env-file was used
        if let Some(file) = &cli.cfg.env_file {
            use std::str::FromStr;

            // get all environment variable from the environment file
            dotenvy::from_filename_override(file)?;

            // set the variables as needed
            apiconfig = Self::new(
                IpAddr::from_str(&dotenvy::var("BIND_ADDR")?)?,
                u16::from_str(&dotenvy::var("BIND_PORT")?)?,
                dotenvy::var("DATABASE_URL")?.to_owned(),
                OpenApiDocs::new(
                    bool::from_str(&dotenvy::var("ENABLE_SWAGGER_UI")?)?,
                    bool::from_str(&dotenvy::var("ENABLE_REDOC")?)?,
                    bool::from_str(&dotenvy::var("ENABLE_SCALAR")?)?,
                    bool::from_str(&dotenvy::var("ENABLE_RAPIDOC")?)?,
                ),
            );
            // if --config was used
        } else if let Some(file) = &cli.cfg.config {
            // read the config file line by line and store it in a String
            let file = std::fs::read(file)?
                .iter()
                .map(|c| *c as char)
                .collect::<String>();

            // parse the configuration String and store in model Struct
            let my_configs: Self = toml::from_str(&file)?;

            // set the variables as needed
            apiconfig = Self::new(
                my_configs.address,
                my_configs.port,
                my_configs.database_url.clone(),
                OpenApiDocs::new(
                    my_configs.openapi.enable_swagger_ui,
                    my_configs.openapi.enable_redoc,
                    my_configs.openapi.enable_scalar,
                    my_configs.openapi.enable_rapidoc,
                ),
            );
        // if positional parameters where used
        } else {
            // set the variables as needed
            apiconfig = Self::new(
                cli.arg.address,
                cli.arg.port,
                cli.arg.database_url.clone(),
                OpenApiDocs::new(
                    cli.arg.with_swagger_ui,
                    cli.arg.with_redoc,
                    cli.arg.with_scalar,
                    cli.arg.with_rapidoc,
                ),
            );
        }

        Ok(apiconfig)
    }
}

impl fmt::Display for ApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BIND_ADDR=\"{}\"\nBIND_PORT={}\nDATABASE_URL={}\n\n{}",
            self.address, self.port, self.database_url, self.openapi
        )
    }
}

/// Default trait implementation
impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            address: IpAddr::from_str("0.0.0.0").unwrap(),
            port: 3000,
            database_url: "sqlite:random-words.db".to_string(),
            openapi: OpenApiDocs::default(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct OpenApiDocs {
    pub enable_swagger_ui: bool,
    pub enable_redoc: bool,
    pub enable_scalar: bool,
    pub enable_rapidoc: bool,
}

impl OpenApiDocs {
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

impl fmt::Display for OpenApiDocs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# OpenAPI Docs\nENABLE_SWAGGER_UI={}\nENABLE_REDOC={}\nENABLE_SCALAR={}\nENABLE_RAPIDOC={}\n",
            self.enable_swagger_ui, self.enable_redoc, self.enable_scalar, self.enable_rapidoc
        )
    }
}
