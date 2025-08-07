// API command line interface
use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;
use validator::Validate;

/// Command line interface
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

/// Command line subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a configuration file with default values
    GenConfig {
        /// Configuration file name
        #[arg(default_value = "config.toml")]
        file_name: Option<PathBuf>,
    },
    /// Generate an environment file with default values
    GenEnvFile {
        /// Environment file name
        #[arg(default_value = ".env")]
        file_name: Option<PathBuf>,
    },
}

/// Configuration arguments
#[derive(Args, Debug)]
#[group(id = "cfg", required = false, multiple = false, conflicts_with = "arg")]
pub struct Config {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE_NAME")]
    pub config: Option<PathBuf>,

    /// Environment file path
    #[arg(short, long, value_name = "FILE_NAME")]
    pub env_file: Option<PathBuf>,
}

/// Command line arguments
#[derive(Args, Debug, Validate)]
#[group(id = "arg", multiple = true, conflicts_with = "cfg")]
pub struct Arguments {
    /// API IP address to bind
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// API port number to bind
    #[validate(range(min = 1, max = 65535))]
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// API database connection URL
    #[arg(short, long, default_value = "sqlite:random-words.db")]
    pub database_url: String,

    /// Enable SwaggerUI
    #[arg(long, default_value_t = false)]
    pub with_swagger_ui: bool,

    /// Enable Redoc
    #[arg(long, default_value_t = false)]
    pub with_redoc: bool,

    /// Enable Scalar
    #[arg(long, default_value_t = false)]
    pub with_scalar: bool,

    /// Enable RapiDoc
    #[arg(long, default_value_t = false)]
    pub with_rapidoc: bool,
}
