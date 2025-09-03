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
        } else if let Some(auth_error) = self.0.downcast_ref::<AuthError>() {
            // AuthError should return appropriate HTTP status codes
            let (status, message) = match auth_error {
                AuthError::InvalidToken => {
                    (StatusCode::UNAUTHORIZED, "Invalid authentication token")
                }
                AuthError::MissingToken => {
                    (StatusCode::UNAUTHORIZED, "Missing authorization token")
                }
                AuthError::TokenExpired => {
                    (StatusCode::UNAUTHORIZED, "Authentication token expired")
                }
                AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
                AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
                AuthError::ValidationFailed => (StatusCode::BAD_REQUEST, "Validation failed"),
                AuthError::UsernameExists => (StatusCode::BAD_REQUEST, "Username already exists"),
                AuthError::DatabaseError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authentication database error",
                ),
                AuthError::InternalError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal authentication error",
                ),
            };
            (status, message).into_response()
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

/// Authentication and authorization errors for JWT tokens and user operations
///
/// Handles authentication failures including token validation, user credentials,
/// and authorization checks for protected endpoints.
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    /// JWT token is malformed, invalid signature, or corrupted
    #[error("invalid authentication token")]
    InvalidToken,
    /// Authorization header missing from request
    #[error("missing authorization token")]
    MissingToken,
    /// JWT token has expired and needs renewal
    #[error("authentication token expired")]
    TokenExpired,
    /// Username/password combination is incorrect
    #[error("invalid credentials")]
    InvalidCredentials,
    /// User account not found in database
    #[error("user not found")]
    UserNotFound,
    /// Request validation failed (username/password format, etc.)
    #[error("validation failed")]
    ValidationFailed,
    /// Username already exists in the system
    #[error("username already exists")]
    UsernameExists,
    /// Database operation failed during authentication
    #[error("authentication database error")]
    DatabaseError(#[from] sqlx::Error),
    /// Internal server error during authentication
    #[error("internal authentication error")]
    InternalError(#[from] anyhow::Error),
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

    #[test]
    fn test_auth_error_variants() {
        // Test InvalidToken
        let auth_error = AuthError::InvalidToken;
        assert_eq!(format!("{auth_error}"), "invalid authentication token");

        // Test MissingToken
        let auth_error = AuthError::MissingToken;
        assert_eq!(format!("{auth_error}"), "missing authorization token");

        // Test TokenExpired
        let auth_error = AuthError::TokenExpired;
        assert_eq!(format!("{auth_error}"), "authentication token expired");

        // Test InvalidCredentials
        let auth_error = AuthError::InvalidCredentials;
        assert_eq!(format!("{auth_error}"), "invalid credentials");

        // Test UserNotFound
        let auth_error = AuthError::UserNotFound;
        assert_eq!(format!("{auth_error}"), "user not found");

        // Test ValidationFailed
        let auth_error = AuthError::ValidationFailed;
        assert_eq!(format!("{auth_error}"), "validation failed");

        // Test UsernameExists
        let auth_error = AuthError::UsernameExists;
        assert_eq!(format!("{auth_error}"), "username already exists");

        // Test DatabaseError conversion
        let sqlx_error = sqlx::Error::RowNotFound;
        let auth_error = AuthError::DatabaseError(sqlx_error);
        assert!(format!("{auth_error}").contains("authentication database error"));

        // Test InternalError conversion
        let anyhow_error = anyhow::anyhow!("internal auth failure");
        let auth_error = AuthError::InternalError(anyhow_error);
        assert!(format!("{auth_error}").contains("internal authentication error"));
    }

    #[test]
    fn test_auth_error_into_app_error() {
        // Test that AuthError can be converted to AppError
        let auth_error = AuthError::InvalidToken;
        let app_error = AppError::from(auth_error);
        assert!(format!("{}", app_error.0).contains("invalid authentication token"));
    }

    #[test]
    fn test_auth_error_http_responses() {
        // Test InvalidToken response
        let error = AppError(anyhow::Error::from(AuthError::InvalidToken));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test MissingToken response
        let error = AppError(anyhow::Error::from(AuthError::MissingToken));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test TokenExpired response
        let error = AppError(anyhow::Error::from(AuthError::TokenExpired));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test InvalidCredentials response
        let error = AppError(anyhow::Error::from(AuthError::InvalidCredentials));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test UserNotFound response
        let error = AppError(anyhow::Error::from(AuthError::UserNotFound));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Test ValidationFailed response
        let error = AppError(anyhow::Error::from(AuthError::ValidationFailed));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test UsernameExists response
        let error = AppError(anyhow::Error::from(AuthError::UsernameExists));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test DatabaseError response
        let sqlx_error = sqlx::Error::RowNotFound;
        let error = AppError(anyhow::Error::from(AuthError::DatabaseError(sqlx_error)));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Test InternalError response
        let anyhow_error = anyhow::anyhow!("auth internal error");
        let error = AppError(anyhow::Error::from(AuthError::InternalError(anyhow_error)));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_auth_error_from_conversions() {
        // Test conversion from sqlx::Error
        let sqlx_error = sqlx::Error::RowNotFound;
        let auth_error = AuthError::from(sqlx_error);
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));

        // Test conversion from anyhow::Error
        let anyhow_error = anyhow::anyhow!("test error");
        let auth_error = AuthError::from(anyhow_error);
        assert!(matches!(auth_error, AuthError::InternalError(_)));
    }
}
