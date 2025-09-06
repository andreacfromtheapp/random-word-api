//! Define default tracing log levels. Uses `RUST_LOG` when unset.
pub const TRACING_LOG_LEVELS: &str = "sqlx=info,tower_http=debug,info";

/// Configure  tracing and logging using Tokio lib-tracing
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
