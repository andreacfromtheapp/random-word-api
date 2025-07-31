//! This is my first Rust API with Axum. The proverbial itch to learn API design and development with Rust.
//! Its main purpose it to be a simple API to use with my Speak and Spell toy project. WIP. Come back soon.
//!
//! Example run with custom port and SQLite db.file:
//!
//! ```not_rust
//! random-word-api -p 3000 -d random-words.db
//! ```
//!
//! Random Word API comes with a comprehensive help menu:
//!
//! ```non_rust
//! random-word-api -h
//! ```
//!

use crate::error::AppError;
use anyhow::{bail, Context, Result};
use cli::Cli;
use std::{net::IpAddr, path::PathBuf};

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
/// Top-level router
mod routes;
/// Database pool
mod state;

/// Configure tracing and logging
fn init_tracing() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    let rust_log = std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| "sqlx=info,tower_http=debug,info".to_string());

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
fn create_config_toml(file: &PathBuf) -> Result<(), AppError> {
    use model::config_file::ConfigurationFile;
    use std::fs::File;
    use std::io::prelude::*;

    let default_configs = ConfigurationFile::default();
    let mut buffer = File::create(file)?;
    let toml = toml::to_string(&default_configs)?;
    buffer.write_all(toml.as_bytes())?;

    Ok(())
}

/// Parse Cli arguments or ENV variables to construct needed address, port, and database-url
fn init_arguments(cli: &Cli) -> Result<(IpAddr, u16, String), AppError> {
    use model::config_file::ConfigurationFile;
    use std::fs::{self};
    use std::path::Path;
    use std::str::FromStr;

    let address;
    let port;
    let database_url;

    // if --env-file was used, get all values from the `.env` file
    if let Some(file) = &cli.cfg.env_file {
        // get all environment variable from the environment file
        dotenvy::from_filename_override(file)?;
        address = IpAddr::from_str(&dotenvy::var("BIND_ADDR")?)?;
        port = u16::from_str(&dotenvy::var("BIND_PORT")?)?;
        database_url = dotenvy::var("DATABASE_URL")?.to_owned();
    // if --config was used, get all values from config.toml
    } else if cli.cfg.config.is_some() {
        // FIXME TODO change this to default config_dir/config.toml?
        // unless this is pointless....
        let config_file = &cli.cfg.config.clone().unwrap();

        // Create config.toml with default values, if non-existent
        if !Path::new(&config_file).exists() {
            create_config_toml(config_file)?;
        };

        // need to read the TOML file before we can do anything with it
        let file = fs::read(config_file)?
            .iter()
            .map(|c| *c as char)
            .collect::<String>();

        // parsing the configuration file
        let my_configs: ConfigurationFile = toml::from_str(&file)?;

        address = my_configs.address;
        port = my_configs.port;
        database_url = my_configs.database_url.clone();
    // otherwise, if positional parameters where used, set those
    } else {
        address = cli.args.address;
        port = cli.args.port;
        database_url = cli.args.database_url.clone();
    }

    Ok((address, port, database_url))
}

/// Tokio Main. What else?!
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use crate::state::init_dbpool;
    use clap::Parser;
    use routes::create_router;

    // Parse command-line args
    let cli = cli::Cli::parse();
    // Either CLI or (with precedence order): ENV_FILE, CONFIG
    let Ok((address, port, database_url)) = init_arguments(&cli) else {
        let env_file_path = cli.cfg.env_file.clone().unwrap();
        let _wrong_env = std::fs::read(env_file_path.clone()).with_context(|| {
            format!(
                "failed to read environment from {:?}",
                env_file_path.into_os_string().into_string().unwrap()
            )
        })?;
        bail!("something went really wrong... this was not supposed to happen!");
    };

    // Setup DB connection pool
    let dbpool = init_dbpool(&database_url)
        .await
        .context("couldn't initialize the database pool")?;

    // Enable tracing using Tokio's https://tokio.rs/#tk-lib-tracing
    init_tracing();

    // Setup top-level router
    let router = create_router(dbpool).await;

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind((address, port))
        .await
        .context("couldn't bind to address")?;

    // Serve the word API
    axum::serve(listener, router)
        .await
        .context("couldn't start the Axum server")?;

    Ok(())
}
