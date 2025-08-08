//! Error handling module
//!
//! This module provides comprehensive error handling for the random word API,
//! including custom error types, HTTP response conversion, and database error
//! handling. It centralizes error management to ensure consistent error responses
//! and proper status code mapping throughout the application.
//!
//! # Error Architecture
//!
//! The module implements a layered error handling approach:
//! - `AppError`: Main application error wrapper for all error types
//! - `SqlxError`: Specialized database error handling with detailed error types
//! - Automatic conversion from various error types using `From` trait implementations
//! - HTTP response integration through Axum's `IntoResponse` trait
//!
//! # HTTP Integration
//!
//! All errors are automatically converted to appropriate HTTP responses with
//! proper status codes and error messages. The error handling is integrated
//! with Axum's response system to provide consistent API error responses.
//!
//! # Database Error Handling
//!
//! Special handling is provided for database operations with specific error
//! types for connection issues, query failures, and migration problems.

// Error handling helpers
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Main application error wrapper for all error types.
///
/// This struct serves as the primary error type for the entire application,
/// wrapping all other error types using `anyhow::Error` for flexible error
/// handling. It provides a single error type that can represent any error
/// that occurs within the application while maintaining the original error
/// context and chain.
///
/// # Design Philosophy
///
/// The wrapper pattern allows for:
/// - Unified error handling across all application layers
/// - Automatic conversion from any error type that implements `Into<anyhow::Error>`
/// - Consistent HTTP response formatting through Axum integration
/// - Preservation of error context and stack traces for debugging
///
/// # HTTP Response Conversion
///
/// When converted to an HTTP response, all `AppError` instances result in:
/// - HTTP status code 500 (Internal Server Error)
/// - Error message formatted as "Something went wrong: {error_details}"
/// - Plain text response body suitable for client consumption
///
/// # Usage Pattern
///
/// This error type is designed to be used with Rust's `?` operator for
/// convenient error propagation throughout the application stack.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

/// Converts `AppError` into an HTTP response for client consumption.
///
/// This implementation defines how application errors are presented to API clients
/// by converting internal errors into appropriate HTTP responses. All errors are
/// currently mapped to 500 Internal Server Error status codes with descriptive
/// error messages.
///
/// # Response Format
///
/// The response includes:
/// - HTTP status code: 500 (Internal Server Error)
/// - Content type: Plain text
/// - Body: Formatted error message with error details
///
/// # Error Message Format
///
/// Error messages follow the pattern "Something went wrong: {error_details}"
/// where error_details contains the underlying error information from the
/// wrapped `anyhow::Error`.
///
/// # Security Considerations
///
/// The current implementation exposes internal error details to clients.
/// In production environments, consider filtering sensitive information
/// from error messages to prevent information disclosure.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

/// Enables automatic conversion from various error types to `AppError`.
///
/// This generic implementation allows any error type that can be converted
/// into `anyhow::Error` to be automatically converted into `AppError` using
/// Rust's `From` trait. This enables the use of the `?` operator for error
/// propagation throughout the application.
///
/// # Automatic Conversions
///
/// This implementation provides automatic conversion for common error types:
/// - Standard library errors (`std::io::Error`, etc.)
/// - Database errors (`sqlx::Error`)
/// - Serialization errors (`serde_json::Error`)
/// - Custom application errors
/// - Any error type implementing `Into<anyhow::Error>`
///
/// # Error Chain Preservation
///
/// The conversion preserves the complete error chain, including:
/// - Original error message and details
/// - Error source chain for root cause analysis
/// - Stack trace information where available
/// - Context added by intermediate error handlers
///
/// # Usage with `?` Operator
///
/// This implementation enables seamless error propagation using the `?` operator
/// in functions that return `Result<T, AppError>`, eliminating the need for
/// explicit error conversion calls throughout the codebase.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Specialized error types for database operations and migrations.
///
/// This enum provides specific error handling for database-related operations
/// using the `thiserror` crate for structured error definitions. It categorizes
/// database errors into distinct types for better error handling and debugging.
///
/// # Error Categories
///
/// The enum handles two primary categories of database errors:
/// - Runtime database errors during query execution and connection management
/// - Migration errors during schema updates and database initialization
///
/// # Error Propagation
///
/// Each variant uses the `#[from]` attribute to enable automatic conversion
/// from the underlying error types, allowing seamless error propagation from
/// the database layer to the application layer.
///
/// # Integration with AppError
///
/// These errors are automatically converted to `AppError` through the generic
/// `From` implementation, ensuring consistent error handling throughout the
/// application while preserving database-specific error information.
///
/// # Debugging Support
///
/// The `Debug` trait implementation provides detailed error information
/// including the original database error context, making it easier to
/// diagnose database-related issues during development and production.
#[derive(thiserror::Error, Debug)]
pub enum SqlxError {
    /// Database runtime errors including connection failures and query errors.
    ///
    /// This variant encompasses all runtime database errors that can occur during
    /// normal database operations, including:
    /// - Connection pool exhaustion or timeout
    /// - SQL query syntax or execution errors
    /// - Database constraint violations
    /// - Transaction rollback failures
    /// - Database connection drops or network issues
    ///
    /// The `#[from]` attribute enables automatic conversion from `sqlx::Error`
    /// to this variant, simplifying error handling in database operations.
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    /// Database migration errors during schema updates and initialization.
    ///
    /// This variant handles errors that occur during database migration operations,
    /// including:
    /// - Migration file parsing errors
    /// - Schema update conflicts or failures
    /// - Migration version inconsistencies
    /// - Database initialization problems
    /// - Migration rollback issues
    ///
    /// The `#[from]` attribute enables automatic conversion from `sqlx::migrate::MigrateError`
    /// to this variant, providing specialized handling for migration-specific errors.
    #[error("database error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Path validation errors for API route parameters.
///
/// This enum provides specific error handling for invalid path parameters
/// in API routes, particularly for language codes and other path-based
/// identifiers that must conform to specific formats or allowed values.
///
/// # Error Categories
///
/// The enum handles path-related validation errors that occur when:
/// - Invalid language codes are provided in URL paths
/// - Unsupported path parameters are used in requests
/// - Path segments don't match expected format patterns
///
/// # Integration with AppError
///
/// These errors are automatically converted to `AppError` through the generic
/// `From` implementation, ensuring consistent error handling throughout the
/// application while preserving path-specific error information.
///
/// # HTTP Response Mapping
///
/// Path errors typically result in:
/// - 400 Bad Request status codes for invalid path parameters
/// - 404 Not Found status codes for unsupported language paths
/// - Descriptive error messages indicating the invalid path component
#[derive(thiserror::Error, Debug)]
pub enum PathError {
    /// Invalid API path parameter error.
    ///
    /// This variant handles errors when API path parameters don't match
    /// expected values or formats, such as:
    /// - Unsupported language codes in `/{lang}/` routes
    /// - Invalid resource identifiers in path segments
    /// - Path parameters that don't conform to validation rules
    /// - Malformed path components that can't be processed
    ///
    /// The error includes the invalid path value for debugging and
    /// client error message generation.
    #[error("invalid API Path: {0}")]
    InvalidPath(String),
}
