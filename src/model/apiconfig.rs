// Application configuration
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr};

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
    // pub fn new(address: IpAddr, port: u16, database_url: String, openapi: OpenApiDocs) -> Self {
    //     Self {
    //         address,
    //         port,
    //         database_url,
    //         openapi,
    //     }
    // }

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
    enable_swagger_ui: bool,
    enable_redoc: bool,
    enable_scalar: bool,
    enable_rapidoc: bool,
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
