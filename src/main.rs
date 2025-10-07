//! Random Word API Server
//!
//! Main entry point for the word API server. Use `word-api-axum -h` for
//! command-line options and configuration help.

use clap::Parser;
use random_word_api::{cli, error::AppError, run_app};

/// Minimal main function - just parse CLI args and delegate to library
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = cli::Cli::parse();
    run_app(cli).await
}
