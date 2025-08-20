//! Authentication endpoint handlers
//!
//! Provides login and registration endpoints for user authentication.
//! Handles JWT token generation and user credential validation.

use axum::{extract::State, response::IntoResponse, Json};
use validator::Validate;

use crate::auth::{JwtManager, PasswordHelper, UserRepository};
use crate::error::{AppError, AuthError};
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest};
use crate::state::AppState;

/// User login endpoint
///
/// Validates user credentials and returns a JWT token on success.
/// The token should be included in the Authorization header for protected routes.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful - JWT token returned", body = AuthResponse),
        (status = 400, description = "Bad request - validation failed or malformed JSON"),
        (status = 401, description = "Unauthorized - invalid username or password"),
        (status = 500, description = "Internal server error - database or authentication system failure")
    ),
    tag = "auth_endpoints"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|_| AuthError::ValidationFailed)?;

    // Find user by username
    let user = UserRepository::find_by_username(&state.dbpool, &request.username)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Verify password
    let password_valid = PasswordHelper::verify_password(&request.password, &user.password_hash)
        .map_err(AuthError::InternalError)?;

    if !password_valid {
        return Err(AuthError::InvalidCredentials.into());
    }

    // Get JWT secret from config
    let jwt_secret = {
        let config = state
            .apiconfig
            .lock()
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Config lock failed: {}", e)))?;
        config.jwt_secret.clone()
    };

    // Generate JWT token
    let token = JwtManager::generate_token(&user, &jwt_secret).map_err(AuthError::InternalError)?;

    let response = AuthResponse {
        token,
        expires_in: JwtManager::get_expiration_seconds(),
    };

    Ok(Json(response))
}

/// User registration endpoint
///
/// Creates a new user account with the provided credentials.
/// Only admin users can create other admin accounts.
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created successfully - JWT token returned", body = AuthResponse),
        (status = 400, description = "Bad request - validation failed, malformed JSON, or username already exists"),
        (status = 422, description = "Validation failed - username too short (min 3 chars) or password too short (min 8 chars)"),
        (status = 500, description = "Internal server error - database or authentication system failure")
    ),
    tag = "auth_endpoints"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|_| AuthError::ValidationFailed)?;

    // Check if username already exists
    if UserRepository::find_by_username(&state.dbpool, &request.username)
        .await?
        .is_some()
    {
        return Err(AuthError::UsernameExists.into());
    }

    // Hash the password
    let password_hash =
        PasswordHelper::hash_password(&request.password).map_err(AuthError::InternalError)?;

    // Create the user (non-admin by default)
    let is_admin = request.is_admin.unwrap_or(false);
    let user =
        UserRepository::create_user(&state.dbpool, &request.username, &password_hash, is_admin)
            .await?;

    // Get JWT secret from config
    let jwt_secret = {
        let config = state
            .apiconfig
            .lock()
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Config lock failed: {}", e)))?;
        config.jwt_secret.clone()
    };

    // Generate JWT token
    let token = JwtManager::generate_token(&user, &jwt_secret).map_err(AuthError::InternalError)?;

    let response = AuthResponse {
        token,
        expires_in: JwtManager::get_expiration_seconds(),
    };

    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use axum_test::TestServer;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;

    async fn create_test_app() -> TestServer {
        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());

        let dbpool = crate::init_dbpool(&db_url).await.unwrap();

        // Run migrations to create the users table
        sqlx::migrate!("./migrations").run(&dbpool).await.unwrap();

        let config = ApiConfig {
            address: "127.0.0.1".parse().unwrap(),
            port: 3000,
            database_url: db_url,
            openapi: crate::config::OpenApiDocs::default(),
            jwt_secret: "test_secret_key".to_string(),
        };

        let state = AppState {
            apiconfig: Arc::new(Mutex::new(config)),
            dbpool,
        };

        let app = axum::Router::new()
            .route("/auth/login", axum::routing::post(login))
            .route("/auth/register", axum::routing::post(register))
            .with_state(state);

        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_register_success() {
        let server = create_test_app().await;

        let request_body = json!({
            "username": "testuser",
            "password": "testpassword123"
        });

        let response = server.post("/auth/register").json(&request_body).await;

        response.assert_status(axum::http::StatusCode::CREATED);

        let body: serde_json::Value = response.json();
        assert!(body.get("token").is_some());
        assert!(body.get("expires_in").is_some());
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let server = create_test_app().await;

        let request_body = json!({
            "username": "testuser",
            "password": "testpassword123"
        });

        // First registration should succeed
        let response = server.post("/auth/register").json(&request_body).await;
        response.assert_status(axum::http::StatusCode::CREATED);

        // Second registration with same username should fail
        let response = server.post("/auth/register").json(&request_body).await;
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_invalid_data() {
        let server = create_test_app().await;

        let request_body = json!({
            "username": "ab", // Too short
            "password": "123" // Too short
        });

        let response = server.post("/auth/register").json(&request_body).await;

        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_success() {
        let server = create_test_app().await;

        // First register a user
        let register_body = json!({
            "username": "logintest",
            "password": "loginpassword123"
        });

        let response = server.post("/auth/register").json(&register_body).await;
        response.assert_status(axum::http::StatusCode::CREATED);

        // Then try to login
        let login_body = json!({
            "username": "logintest",
            "password": "loginpassword123"
        });

        let response = server.post("/auth/login").json(&login_body).await;

        response.assert_status(axum::http::StatusCode::OK);

        let body: serde_json::Value = response.json();
        assert!(body.get("token").is_some());
        assert!(body.get("expires_in").is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let server = create_test_app().await;

        let login_body = json!({
            "username": "nonexistent",
            "password": "wrongpassword"
        });

        let response = server.post("/auth/login").json(&login_body).await;

        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let server = create_test_app().await;

        // Register a user
        let register_body = json!({
            "username": "wrongpwtest",
            "password": "correctpassword123"
        });

        let response = server.post("/auth/register").json(&register_body).await;
        response.assert_status(axum::http::StatusCode::CREATED);

        // Try to login with wrong password
        let login_body = json!({
            "username": "wrongpwtest",
            "password": "wrongpassword123"
        });

        let response = server.post("/auth/login").json(&login_body).await;

        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }
}
