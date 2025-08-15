//! Error types and HTTP response conversion
//!
//! Centralizes error handling for the API with automatic conversion to
//! appropriate HTTP status codes and JSON error responses. Provides
//! structured error types for database operations and path validation.

// Error handling helpers
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Main application error wrapper for all error types
#[derive(Debug)]
pub struct AppError(anyhow::Error);

/// Converts `AppError` into HTTP responses with appropriate status codes
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Check if the underlying error is a PathError
        if let Some(path_error) = self.0.downcast_ref::<PathError>() {
            // PathError should return 400 Bad Request
            let message = match path_error {
                PathError::InvalidPath(path) => format!("Invalid language code: {path}"),
                PathError::InvalidWordType(word_type) => {
                    format!("Invalid word type: {word_type}")
                }
            };
            (StatusCode::BAD_REQUEST, message).into_response()
        } else {
            // All other errors return 500 Internal Server Error
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", self.0),
            )
                .into_response()
        }
    }
}

/// Enables automatic conversion from various error types to `AppError`
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Database operation errors with detailed categorization
///
/// Wraps SQLx errors to provide consistent error handling across
/// database operations including connections, queries, and migrations.
#[derive(thiserror::Error, Debug)]
pub enum SqlxError {
    /// Database runtime errors (connections, queries, constraints)
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    /// Database migration errors (schema updates, version conflicts)
    #[error("database migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Path validation errors for API route parameters
///
/// Handles validation failures for URL path parameters including
/// language codes and grammatical word types.
#[derive(thiserror::Error, Debug)]
pub enum PathError {
    /// Invalid language code in URL path (must be supported language like 'en')
    #[error("invalid language code: {0}")]
    InvalidPath(String),
    /// Invalid word type parameter in URL path (must be valid grammatical type)
    #[error("invalid word type: {0}")]
    InvalidWordType(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn test_app_error_into_response() {
        let error = AppError(anyhow::anyhow!("test error"));
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_app_error_from_various_types() {
        // Test conversion from anyhow::Error
        let anyhow_error = anyhow::anyhow!("anyhow error");
        let app_error = AppError::from(anyhow_error);
        assert_eq!(format!("{}", app_error.0), "anyhow error");

        // Test conversion from std::io::Error
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_error = AppError::from(io_error);
        assert!(format!("{}", app_error.0).contains("file not found"));
    }

    #[test]
    fn test_sqlx_error_variants() {
        use sqlx::Error as SqlxDbError;

        // Test database error conversion
        let db_error = SqlxDbError::RowNotFound;
        let sqlx_error = SqlxError::Db(db_error);
        assert!(format!("{sqlx_error}").contains("database error"));

        // Test that SqlxError can be converted to AppError
        let app_error = AppError::from(sqlx_error);
        assert!(format!("{}", app_error.0).contains("database error"));
    }

    #[test]
    fn test_path_error_variants() {
        let path_error = PathError::InvalidPath("invalid_lang".to_string());
        assert_eq!(
            format!("{path_error}"),
            "invalid language code: invalid_lang"
        );

        // Test conversion to AppError
        let app_error = AppError::from(path_error);
        assert!(format!("{}", app_error.0).contains("invalid language code"));
    }

    #[test]
    fn test_error_chain_preservation() {
        // Create a chain of errors
        let root_cause = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let context_error = anyhow::Error::from(root_cause).context("failed to read file");
        let app_error = AppError::from(context_error);

        let error_string = format!("{}", app_error.0);
        // Just check that the context is preserved
        assert!(error_string.contains("failed to read file"));
    }
}
