// Error handling helpers
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Represents an application-level error
#[derive(Debug)]
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

/// Enable using `?` on functions that return `Result<_, anyhow::Error>` to turn them into `Result<_, AppError>`
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Custom `thiserror` errors for database operations
#[derive(thiserror::Error, Debug)]
pub enum SqlxError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("database error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}
