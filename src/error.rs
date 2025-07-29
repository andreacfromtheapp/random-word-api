// Helpers for error handling
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Represents an application-level error
#[derive(Debug)]
pub enum Error {
    Sqlx(StatusCode, String),
    NotFound,
}

// These need revision when adding proper Anyhow and pattern
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Error {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound,
            sqlx::Error::Io(err) => Error::Sqlx(StatusCode::SERVICE_UNAVAILABLE, err.to_string()),
            sqlx::Error::Protocol(str) => Error::Sqlx(StatusCode::SERVICE_UNAVAILABLE, str),
            _ => Error::Sqlx(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Sqlx(code, body) => (code, body).into_response(),
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
        }
    }
}
