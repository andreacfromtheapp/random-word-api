//! Random Word API Axum Binary
//!
//! This is the main binary entry point for the Word API server built with Axum.
//! The core application logic is implemented in the library crate for better
//! testability and reusability.
//!
//! ## Usage
//!
//! Word API Axum comes with a comprehensive help menu:
//!
//! ```non_rust
//! word-api-axum -h
//! ```

use clap::Parser;
use word_api_axum::{cli, error::AppError, run_app};

/// Minimal main function - just parse CLI args and delegate to library
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = cli::Cli::parse();
    run_app(cli).await
}
