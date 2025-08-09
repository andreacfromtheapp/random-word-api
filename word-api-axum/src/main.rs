//! Word API Axum Binary
//!
//! This is the main binary entry point for the Word API server built with Axum.
//! The core application logic is implemented in the library crate for better
//! testability and reusability.
//!
//! ## Running the API
//!
//! Accepted CLI arguments are either config/env files or single arguments for
//! address, port, and database_url.
//!
//! When using --config, if no file is provided, a local ./config.toml is created.
//! When using --env-file, if no file is provided, no default is assumed for security reasons.
//!
//! Word API Axum comes with a comprehensive help menu:
//!
//! ```non_rust
//! word-api-axum -h
//! ```

use clap::Parser;
use word_api_axum::{cli, run_app};

/// Minimal main function - just parse CLI args and delegate to library
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = cli::Cli::parse();
    run_app(cli).await
}
