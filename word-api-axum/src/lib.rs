//! Word API Axum Library
//!
//! This library provides the complete functionality for the Word API server built with Axum.
//! It includes models, handlers, routing, configuration management, database initialization,
//! tracing setup, and the main application runner.
//!
//! The library is designed to be used both as a standalone binary and as a library
//! for testing and integration purposes. All core application logic is contained
//! here, following Rust idioms for better testability and code reuse.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use word_api_axum::{cli, run_app};
//! use clap::Parser;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let cli = cli::Cli::parse();
//!     run_app(cli).await
//! }
//! ```

use anyhow::{bail, Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// Define default tracing log levels. Uses `RUST_LOG` when unset
pub const TRACING_LOG_LEVELS: &str = "sqlx=info,tower_http=debug,info";

/// CLI argument parsing and configuration
pub mod cli;

/// Error handling types and conversions
pub mod error;

/// HTTP request handlers
pub mod handlers;

/// Data models and business logic
pub mod models;

/// Route configuration and middleware
pub mod routes;

/// Application state management
pub mod state;

use crate::cli::Commands;
use crate::error::SqlxError;
use crate::models::apiconfig::{ApiConfig, FileKind};

/// Configure tracing and logging (accepts `RUST_LOG` environment variable or uses default const above)
pub fn init_tracing() {
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
pub async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, SqlxError> {
    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(db_url)?.create_if_missing(true))
        .await?;

    sqlx::migrate!("./migrations").run(&dbpool).await?;

    Ok(dbpool)
}

/// Check if provided env-file or config are non-existent and exit gracefully
pub fn does_file_exist(file_name: &Path, file_kind: &str) -> Result<(), anyhow::Error> {
    std::fs::read(file_name)
        .with_context(|| format!("couldn't read {file_kind} file '{file_name:?}'"))?;

    Ok(())
}

/// Main application logic - extracted from main() for better testability and reusability
pub async fn run_app(cli: cli::Cli) -> Result<(), anyhow::Error> {
    use routes::create_router;

    // Handle setup commands first
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
