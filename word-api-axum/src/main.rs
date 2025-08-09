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
//! word-api-axum -h
//! ```
//!

/// Define default tracing log levels. Uses `RUST_LOG` when unset
const TRACING_LOG_LEVELS: &str = "sqlx=info,tower_http=debug,info";

use anyhow::{bail, Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// Cli arguments and interface
mod cli;
/// Helpers for error handling
mod error;
/// API handlers
#[path = "./handlers/mod.rs"]
mod handlers;
/// Model and business logic
#[path = "./models/mod.rs"]
mod models;
/// Routes module
#[path = "./routes/mod.rs"]
mod routes;
/// Database pool
mod state;

use crate::cli::Commands;
use crate::error::SqlxError;
use crate::models::apiconfig::{ApiConfig, FileKind};

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

/// Configure the database pool
async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, SqlxError> {
    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(db_url)?.create_if_missing(true))
        .await?;

    sqlx::migrate!("./migrations").run(&dbpool).await?;

    Ok(dbpool)
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
    use clap::Parser;
    use routes::create_router;

    // Parse command-line args
    let cli = cli::Cli::parse();

    // if setup --create-config was issued, create the file and exit
    match &cli.command {
        Some(Commands::GenConfig { file_name }) => {
            let file = &file_name.clone().unwrap();
            ApiConfig::gen_file(file, FileKind::Toml)?;
            std::process::exit(0x0100);
        }
        Some(Commands::GenEnvFile { file_name }) => {
            let file = &file_name.clone().unwrap();
            ApiConfig::gen_file(file, FileKind::EnvFile)?;
            std::process::exit(0x0100);
        }
        None => {}
    }

    // Get values from either ENV_FILE, CONFIG, or CLI; else exit gracefully
    let Ok(apiconfig) = ApiConfig::from_cli(&cli) else {
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
    let dbpool = init_dbpool(&apiconfig.database_url)
        .await
        .context("couldn't initialize the database connection pool")?;

    let state = state::AppState {
        config: Arc::new(Mutex::new(apiconfig.clone())),
        dbpool: dbpool.clone(),
    };

    // Setup top-level router (includes SwaggerUI)
    let router = create_router(state).await;

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind((apiconfig.address, apiconfig.port))
        .await
        .context("couldn't bind to address or port")?;

    // Serve the API
    axum::serve(listener, router)
        .await
        .context("couldn't start the API server")?;

    Ok(())
}
