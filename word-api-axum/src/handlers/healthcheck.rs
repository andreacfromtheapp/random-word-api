//! Health check and system status endpoints
//!
//! Public endpoints for monitoring system health and database connectivity.
//! Designed for use with load balancers and monitoring systems.
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
/// # Returns
///
/// * `200 OK` - API service is alive and responding to requests
///
#[utoipa::path(
    get,
    context_path = "/health",
    path = "/alive",
    operation_id = "api_liveness_check",
    tag = "healthcheck_endpoints",

    responses(
        (status = 200, description = "API service is alive and responding to requests", body = String),
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
/// # Returns
///
/// * `200 OK` - Database connection successful, API is ready to serve requests
/// * `500 Internal Server Error` - Database connection failed, API is not ready
///
#[utoipa::path(
    get,
    context_path = "/health",
    path = "/ready",
    operation_id = "api_db_connection_test",
    tag = "healthcheck_endpoints",

    responses(
        (status = 200, description = "API is ready and database connection is successful", body = String),
        (status = 500, description = "API is not ready - database connection failed or other critical error"),
    )
)]
pub async fn ready(State(state): State<AppState>) -> Result<String, AppError> {
    use sqlx::Connection;

    let mut conn = state.dbpool.acquire().await?;
    conn.ping()
        .await
        .map(|_| "The API can establish a connection to the database".to_string())
        .map_err(Into::into)
}
