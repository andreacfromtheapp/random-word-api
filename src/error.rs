// Helpers for error handling
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Represents an application-level error
#[derive(Debug)]
pub enum Error {
    NotFound,
    BadArgument,
    Sqlx(StatusCode, String),
    DotEnvy,
    Io,
    Path,
}

// These need revision when adding proper Anyhow and pattern
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Error {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound,
            sqlx::Error::InvalidArgument(_str) => Error::BadArgument,
            sqlx::Error::Io(err) => Error::Sqlx(StatusCode::SERVICE_UNAVAILABLE, err.to_string()),
            sqlx::Error::Protocol(str) => Error::Sqlx(StatusCode::SERVICE_UNAVAILABLE, str),
            _ => Error::Sqlx(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
    }
}

impl From<dotenvy::Error> for Error {
    fn from(_err: dotenvy::Error) -> Error {
        Error::DotEnvy
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(_err: std::net::AddrParseError) -> Error {
        Error::DotEnvy
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_err: std::num::ParseIntError) -> Error {
        Error::DotEnvy
    }
}

impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Self {
        Error::Io
    }
}

impl From<toml::ser::Error> for Error {
    fn from(_err: toml::ser::Error) -> Self {
        Error::Path
    }
}

impl From<toml::de::Error> for Error {
    fn from(_err: toml::de::Error) -> Self {
        Error::Path
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
            Error::BadArgument => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
            Error::Sqlx(code, body) => (code, body).into_response(),
            Error::DotEnvy => StatusCode::NOT_ACCEPTABLE.into_response(),
            Error::Io => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
            Error::Path => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        }
    }
}
