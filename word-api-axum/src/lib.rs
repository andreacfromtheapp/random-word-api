//! Random Word API - A simple dictionary word service
//!
//! Provides HTTP endpoints for retrieving random dictionary words with definitions
//! and pronunciations. Supports filtering by grammatical type (noun, verb, etc.)
//! and includes administrative endpoints for word management.
//!
//! Built with Axum for high-performance async HTTP handling and SQLite for
//! lightweight data storage.
//!
//! # Security
//! - Public endpoints: Word retrieval, health checks, API documentation
//! - Protected endpoints: Administrative word management (requires JWT authentication)
//!

use anyhow::{Context, Result};
use std::path::Path;
use tokio::signal;

pub mod auth;
pub mod cli;
pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod state;

use crate::error::AppError;

/// Graceful shutdown implemented to stop accepting new requests, wait for the current
///  requests to finish, and then shut down the server.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Validates file existence for configuration files
///
/// Checks if the specified configuration file exists and is readable.
/// Used for validating --config and --env-file arguments before processing.
pub fn does_file_exist(file_name: &Path, file_kind: &str) -> Result<(), AppError> {
    std::fs::read(file_name)
        .with_context(|| format!("couldn't read {file_kind} file '{file_name:?}'"))?;

    Ok(())
}

/// Main application logic with configuration handling and server startup
///
/// Processes CLI arguments, initializes configuration from multiple sources,
/// sets up database connections, and starts the HTTP server with all routes.
/// Extracted from main() for better testability and reusability.
pub async fn run_app(cli: cli::Cli) -> Result<(), AppError> {
    use crate::cli::Commands;
    use crate::config::{ApiConfig, FileKind};
    use crate::routes::create_router;
    use crate::state::init_dbpool;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};

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
        // I want a panic here, so it's explicit.....
        panic!("something went really wrong... this was not supposed to happen!");
    };

    // Setup the database connection pool
    let dbpool = init_dbpool(&apiconfig.server_settings.database_url)
        .await
        .context("couldn't initialize the database connection pool")?;

    // Setup the shared mutable state
    let shared_state = state::AppState {
        apiconfig: Arc::new(Mutex::new(apiconfig.clone())),
        dbpool: dbpool.clone(),
    };

    // Setup top-level router (includes SwaggerUI)
    let router = create_router(shared_state).await;

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind((
        apiconfig.server_settings.address,
        apiconfig.server_settings.port,
    ))
    .await
    .context("couldn't bind TCP listener")?;

    // Serve the API
    axum::serve(
        listener,
        router?.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("couldn't start the API server")?;

    Ok(())
}
