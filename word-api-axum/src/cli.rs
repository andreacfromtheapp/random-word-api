//! Command-line interface and configuration
//!
//! Supports configuration via CLI arguments, TOML files, or environment variables.
//! Includes subcommands for generating default config files.
//!
//! # Configuration priority (highest to lowest)
//! 1. Environment files (.env)
//! 2. TOML config files
//! 3. CLI arguments

// API command line interface
use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;
use validator::Validate;

/// Main CLI parser combining configuration options and subcommands
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    /// Configuration options
    pub cfg: Config,

    #[command(flatten)]
    /// Argument options
    pub arg: Arguments,

    /// Commands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Utility subcommands for configuration file generation
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates a TOML configuration file with default values
    GenConfig {
        /// Configuration file name
        #[arg(default_value = "config.toml")]
        file_name: Option<PathBuf>,
    },
    /// Generates an environment file with default values
    GenEnvFile {
        /// Environment file name
        #[arg(default_value = ".env")]
        file_name: Option<PathBuf>,
    },
}

/// Configuration file and environment file options (mutually exclusive with Arguments)
#[derive(Args, Debug)]
#[group(id = "cfg", required = false, multiple = false, conflicts_with = "arg")]
pub struct Config {
    /// Path to TOML configuration file
    #[arg(short, long, value_name = "FILE_NAME")]
    pub config: Option<PathBuf>,

    /// Path to environment variable file
    #[arg(short, long, value_name = "FILE_NAME")]
    pub env_file: Option<PathBuf>,
}

/// Direct command-line arguments (mutually exclusive with Config)
#[derive(Args, Debug, Validate)]
#[group(id = "arg", multiple = true, conflicts_with = "cfg")]
pub struct Arguments {
    /// IP address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// Port number to listen on
    #[validate(range(min = 1, max = 65535))]
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Database connection URL
    #[arg(short, long, default_value = "sqlite:random-words.db")]
    pub database_url: String,

    /// Enable SwaggerUI documentation interface
    #[arg(long, default_value_t = false)]
    pub with_swagger_ui: bool,

    /// Enable Redoc documentation interface
    #[arg(long, default_value_t = false)]
    pub with_redoc: bool,

    /// Enable Scalar documentation interface
    #[arg(long, default_value_t = false)]
    pub with_scalar: bool,

    /// Enable RapiDoc documentation interface
    #[arg(long, default_value_t = false)]
    pub with_rapidoc: bool,
}
