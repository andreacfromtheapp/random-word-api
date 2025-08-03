//! This is my first Rust API with Axum. The proverbial itch to learn API design and development with Rust.
//! Its main purpose it to be a simple API to use with my Speak and Spell toy project. WIP. Come back soon.
//!
//! ## Running the API
//!
//! Accepted cli arguments are either config/env files or single arguments for address, port, and database_url. Notes:
//!
//! When using --config, if no file is provided, a - local to where the bin is run - ./config.toml is created.
//!
//! When using --env-file (i.e: DEV.env) if no file is provided no default assumed/created for security reasons.
//!
//! Random Word API comes with a comprehensive help menu:
//!
//! ```non_rust
//! random-word-api -h
//! ```
//!

/// Define default tracing log levels. Uses `RUST_LOG` when unset
const TRACING_LOG_LEVELS: &str = "sqlx=info,tower_http=debug,info";

use anyhow::{bail, Context, Result};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, Commands};
use crate::error::AppError;

/// Cli arguments and interface
mod cli;
/// Helpers for error handling
mod error;
/// API handlers
#[path = "./handlers/mod.rs"]
mod handlers;
/// Model and business logic
#[path = "./model/mod.rs"]
mod model;
use model::config_file::ConfigurationFile;
/// Top-level router
mod routes;
/// Database pool
mod state;

/// Configure tracing and logging (accepts `RUST_LOG` environment variable or uses default const above)
fn init_tracing() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    let rust_log =
        std::env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| TRACING_LOG_LEVELS.to_string());

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse_lossy(rust_log),
        )
        .init();
}

/// Helper to create the default config.toml if non-existent
fn create_default_config_toml(file: &PathBuf) -> Result<(), anyhow::Error> {
    use model::config_file::ConfigurationFile;
    use std::fs::File;
    use std::io::prelude::*;

    // set default config values
    let default_configs = ConfigurationFile::default();
    // read the default values
    let toml = toml::to_string(&default_configs)?;
    // create the default file
    let mut buffer = File::create(file)?;
    // write all lines from the above steps into config.toml
    buffer.write_all(toml.as_bytes())?;

    Ok(())
}

/// Parse Cli arguments to construct `address`, `port`, and `database-url`
/// Accepts `BIND_ADDR`, `BIND_PORT`, and `DATABASE_URL` from an `.env` file.
fn init_arguments(cli: &Cli) -> Result<(IpAddr, u16, String), AppError> {
    let address;
    let port;
    let database_url;

    // if --env-file was used
    if let Some(file) = &cli.cfg.env_file {
        use std::str::FromStr;

        // get all environment variable from the environment file
        dotenvy::from_filename_override(file)?;

        // set the variables as needed
        address = IpAddr::from_str(&dotenvy::var("BIND_ADDR")?)?;
        port = u16::from_str(&dotenvy::var("BIND_PORT")?)?;
        database_url = dotenvy::var("DATABASE_URL")?.to_owned();
    // if --config was used
    } else if let Some(file) = &cli.cfg.config {
        // read the config file line by line and store it in a String
        let file = std::fs::read(file)?
            .iter()
            .map(|c| *c as char)
            .collect::<String>();

        // parse the configuration String and store in model Struct
        let my_configs: ConfigurationFile = toml::from_str(&file)?;

        // set the variables as needed
        address = my_configs.address;
        port = my_configs.port;
        database_url = my_configs.database_url.clone();
    // if positional parameters where used
    } else {
        // set the variables as needed
        address = cli.arg.address;
        port = cli.arg.port;
        database_url = cli.arg.database_url.clone();
    }

    Ok((address, port, database_url))
}

/// Check if provided env-file or config are non-existent and exit gracefully
fn does_file_exist(file_name: &Path, file_kind: &str) -> Result<(), anyhow::Error> {
    std::fs::read(file_name)
        .with_context(|| format!("couldn't read {file_kind} file '{file_name:?}'"))?;

    Ok(())
}

/// Tokio Main. What else?!
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use crate::state::init_dbpool;
    use clap::Parser;
    use routes::create_router;

    // Parse command-line args
    let cli = cli::Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Setup { create_config }) => {
            let file = &create_config.clone().unwrap();
            create_default_config_toml(file)?;
            std::process::exit(0x0100);
        }
        None => {}
    }

    // Get values from either ENV_FILE, CONFIG, or CLI; else exit gracefully
    let Ok((address, port, database_url)) = init_arguments(&cli) else {
        // if --env-file file doesn't exist, inform the user and exit gracefully
        if let Some(file) = &cli.cfg.env_file {
            does_file_exist(file.as_path(), "environment")?;
        }

        // if --config file doesn't exist, inform the user and exit gracefully
        if let Some(file) = &cli.cfg.config {
            does_file_exist(file.as_path(), "configuration")?;
        }

        // this should never be reached. here because let Ok() else requires !
        bail!("something went really wrong... this was not supposed to happen!");
    };

    // Enable tracing using https://tokio.rs/#tk-lib-tracing
    init_tracing();

    // Setup the database connection pool
    let dbpool = init_dbpool(&database_url)
        .await
        .context("couldn't initialize the database connection pool")?;

    // Setup top-level router
    let router = create_router(dbpool).await;

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind((address, port))
        .await
        .context("couldn't bind to address or port")?;

    // Serve the API
    axum::serve(listener, router)
        .await
        .context("couldn't start the API server")?;

    Ok(())
}
