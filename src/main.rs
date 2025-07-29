//! This is my first Rust API with Axum. The proverbial itch to learn API design and development
//! with Rust.
//!
//! Its main purpose it to be a simple API to use with my Speak and Spell toy project. WIP. Come back soon.
//!
//! Example run with custom port and SQLite db.file
//!
//! ```not_rust
//! cargo run -p 5555 -d random-words.db
//! ```

/// Cli arguments and interface
mod cli;
/// Helpers for error handling
mod error;
/// API handlers
mod handlers;
/// Top-level router
mod router;
/// Model and business logic
mod word;

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

/// Configure the database pool
async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(db_url)?.create_if_missing(true))
        .await
        .expect("can't connect to database");

    sqlx::migrate!("./migrations")
        .run(&dbpool)
        .await
        .expect("database migration failed");

    Ok(dbpool)
}

/// Tokio Main. What else?!
#[tokio::main]
async fn main() {
    use clap::Parser;
    use router::create_router;

    // Parse command-line args
    let cli = cli::Cli::parse();

    // Setup DB connection pool
    let dbpool = init_dbpool(&cli.database_url)
        .await
        .expect("couldn't initialize DB pool");

    // Enable tracing using Tokio's https://tokio.rs/#tk-lib-tracing
    init_tracing();

    // Setup top-level router
    let router = create_router(dbpool).await;

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind((cli.address, cli.port))
        .await
        .expect("couldn't bind to address");

    // Serve the word API
    axum::serve(listener, router).await.expect("server failed");
}
