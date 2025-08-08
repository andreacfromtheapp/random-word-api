//! General handlers module
//!
//! This module contains HTTP handlers for healthcheck utility endpoints that provide
//! system health checks and API status information. These endpoints are publicly
//! accessible and do not require authentication, serving as essential monitoring
//! and debugging tools for the API infrastructure.
//!
//! # Purpose
//!
//! The healthcheck handlers provide fundamental operational endpoints that allow:
//! - Health checks for monitoring systems
//! - Database connectivity verification
//! - API availability confirmation
//! - System readiness validation
//!
//! # Endpoints
//!
//! All endpoints in this module are designed to be lightweight and respond quickly
//! to support automated monitoring and health check systems. They provide essential
//! information about the API's operational status without exposing sensitive data.
//!
//! # Monitoring Integration
//!
//! These endpoints are suitable for integration with:
//! - Load balancer health checks
//! - Container orchestration readiness probes
//! - Application performance monitoring systems
//! - Automated alerting and notification systems
//!
//! # Error Handling
//!
//! Health check endpoints are designed to fail fast and provide clear status
//! information when system components are not functioning properly.
use axum::extract::State;

use crate::error::AppError;
use crate::state::AppState;
/// Simple API liveness check endpoint.
///
/// This endpoint provides a basic liveness probe that confirms the API service
/// is running and responsive. Unlike the readiness probe (`/ready`), this endpoint
/// does not perform any external dependency checks and simply returns a success
/// response to indicate the service is alive.
///
/// # Liveness vs Readiness
///
/// This endpoint differs from the `/ready` endpoint in that it:
/// - Does not test database connectivity
/// - Does not validate external dependencies
/// - Provides immediate response without resource checks
/// - Indicates only that the API process is running
///
/// # Use Cases
///
/// This endpoint is primarily designed for:
/// - Container orchestration liveness probes
/// - Basic service discovery health checks
/// - Load balancer upstream validation
/// - Simple monitoring system integration
/// - Automated restart trigger validation
///
/// # Response Behavior
///
/// The endpoint always returns a success response when the API is running,
/// regardless of the status of external dependencies like databases or
/// third-party services. This makes it suitable for determining whether
/// the service process itself needs to be restarted.
///
/// # Returns
///
/// * `200 OK` - API service is alive and responding to requests
///
/// # Response Format
///
/// Returns a plain text string message confirming the API is running,
/// making it suitable for simple monitoring systems that expect basic
/// text responses rather than JSON payloads.
#[utoipa::path(
    get,
    context_path = "/health",
    path = "/alive",
    operation_id = "api_liveness_check",
    tag = "healthcheck_endpoints",
    responses(
        (status = 200, description = "OK. The API service is alive and running", body = String),
    )
)]
pub async fn alive() -> String {
    "The API is successfully running".to_string()
}

/// Tests database connectivity and returns API readiness status.
///
/// This endpoint performs a live database connection test by acquiring a connection
/// from the pool and executing a ping operation. It serves as a comprehensive health
/// check that validates both the API's ability to connect to the database and the
/// database's responsiveness to queries.
///
/// # Database Testing
///
/// The endpoint performs the following checks:
/// - Acquires a connection from the SQLite connection pool
/// - Executes a ping operation to test database responsiveness
/// - Verifies that the database is accessible and functioning
///
/// # Use Cases
///
/// This endpoint is primarily designed for:
/// - Container orchestration readiness probes
/// - Load balancer health checks
/// - Automated monitoring system integration
/// - Manual API status verification
/// - Deployment validation and testing
///
/// # Response Behavior
///
/// The endpoint provides a simple text response indicating the connection status.
/// Success responses confirm that the API can successfully communicate with the
/// database, while error responses indicate connectivity or database issues.
///
/// # Returns
///
/// * `200 OK` - Database connection successful, API is ready to serve requests
/// * `500 Internal Server Error` - Database connection failed, API is not ready
///
/// # Response Format
///
/// Returns a plain text string message indicating the connection status rather
/// than JSON, making it suitable for simple health check monitoring systems.
#[utoipa::path(
    get,
    context_path = "/health",
    path = "/ready",
    operation_id = "api_db_connection_test",
    tag = "healthcheck_endpoints",
    responses(
        (status = 200, description = "OK. The API can establish a connection to the database", body = String),
        (status = 500, description = "Internal server error. Couldn't connect to the database"),
    )
)]
pub async fn ping(State(state): State<AppState>) -> Result<String, AppError> {
    use sqlx::Connection;

    let mut conn = state.dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "OK. The API can establish a connection to the database".to_string())
        .map_err(Into::into)
}
