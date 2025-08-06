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
use sqlx::SqlitePool;

use crate::error::AppError;

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
    path = "/ready",
    operation_id = "api_db_connection_test",
    tag = "healthcheck_endpoints",
    responses(
        (status = 200, description = "OK. The API can establish a connection to the database", body = String),
        (status = 500, description = "Internal server error. Couldn't connect to the database"),
    )
)]
pub async fn ping(State(dbpool): State<SqlitePool>) -> Result<String, AppError> {
    use sqlx::Connection;

    let mut conn = dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "OK. The API can establish a connection to the database".to_string())
        .map_err(Into::into)
}
