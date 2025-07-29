//! This is my first Rust API with Axum. The proverbial itch to learn API design and development
//! with Rust.
//!
//! Its main purpose it to be a simple API to use with my Speak and Spell toy project. WIP. Come back soon.
//!
//! Run with
//!
//! ```not_rust
//! cargo run
//! ```

use clap::Parser;
use router::create_router;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

/// Helpers for error handling
mod error;
/// API handlers
mod handlers;
/// Top-level router
mod router;
/// Model and business logic
mod word;

use word::Environment;

/// Arguments for clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The port number to bind the API to
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Environment: Development | Test | Production
    #[arg(short, long, default_value_t = Environment::Development, value_enum)]
    env: Environment,

    /// The database connection URL
    #[arg(short, long, default_value = "", env = "DATABASE_URL")]
    database_url: String,
}

/// Configure tracing and logging
fn init_tracing() {
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

/// Configure the database Pool
async fn init_dbpool(db_url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    let db_connection_str =
        std::env::var(db_url).unwrap_or_else(|_| "sqlite:db.sqlite".to_string());

    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(&db_connection_str)?.create_if_missing(true))
        .await
        .expect("can't connect to database");

    sqlx::migrate!("./migrations")
        .run(&dbpool)
        .await
        .expect("database migration failed");

    Ok(dbpool)
}

/// Main. What else?!
#[tokio::main]
async fn main() {
    // Parse command-line args
    let cli = Args::parse();

    // Setup DB connection pool
    let dbpool = init_dbpool(&cli.database_url)
        .await
        .expect("couldn't initialize DB pool");

    // Enable tracing using Tokio's https://tokio.rs/#tk-lib-tracing
    init_tracing();

    // Setup top-level router
    let router = create_router(dbpool).await;

    // Setup socket binding IP address and port
    let bind_addr = SocketAddr::from(([0, 0, 0, 0], cli.port));

    // Instantiate a listener on the socket address and port
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("couldn't bind to address");

    // Serve the word API
    axum::serve(listener, router).await.expect("server failed");
}
