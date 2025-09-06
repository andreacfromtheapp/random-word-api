//! Authentication endpoint handlers
//!
//! Provides login endpoints for user authentication.
//! Handles JWT token generation and user credential validation.

use axum::{extract::State, response::IntoResponse, Json};
use validator::Validate;

use crate::auth::{JwtManager, PasswordHelper};
use crate::error::{AppError, AuthError};
use crate::models::user::{AuthResponse, LoginRequest, User};
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
    let user = User::find_by_username(&state.dbpool, &request.username)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Verify password
    let password_valid = PasswordHelper::verify_password(&request.password, &user.password_hash)
        .map_err(AuthError::InternalError)?;

    if !password_valid {
        return Err(AuthError::InvalidCredentials.into());
    }

    // Get JWT secret and expiration from config
    let (jwt_secret, expiration_minutes) = {
        let config = state
            .apiconfig
            .lock()
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Config lock failed: {}", e)))?;
        (
            config.jwt_settings.secret.clone(),
            config.jwt_settings.token_expiration_minutes,
        )
    };

    // Generate JWT token with dynamic expiration
    let token = JwtManager::generate_token(&user, &jwt_secret, expiration_minutes.into())
        .map_err(AuthError::InternalError)?;

    let response = AuthResponse {
        token,
        expires_in: JwtManager::get_expiration_seconds(expiration_minutes.into()),
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::models::user::User;
    use axum_test::TestServer;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;

    async fn create_test_app() -> TestServer {
        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());

        let dbpool = crate::state::init_dbpool(&db_url).await.unwrap();

        // Run migrations to create the users table
        sqlx::migrate!("./migrations").run(&dbpool).await.unwrap();

        let config = ApiConfig {
            server_settings: crate::config::ApiSettings::new(
                "127.0.0.1".parse().unwrap(),
                3000,
                db_url,
                vec!["localhost".to_string()],
            ),
            compression: crate::config::ApiCompression::default(),
            jwt_settings: crate::config::JwtSettings::new(5, "test_secret_key".to_string()),
            api_limits: crate::config::ApiLimits::new(5, 10, 30, 1024),
            openapi: crate::config::OpenApiDocs::default(),
        };

        let state = AppState {
            apiconfig: Arc::new(Mutex::new(config)),
            dbpool,
        };

        let app = axum::Router::new()
            .route("/auth/login", axum::routing::post(login))
            .with_state(state);

        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_login_success() {
        // Create a test user using User to avoid SQL compilation issues
        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());
        let dbpool = crate::state::init_dbpool(&db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&dbpool).await.unwrap();

        let password_hash = PasswordHelper::hash_password("loginpassword123").unwrap();
        let _user = User::create_user(&dbpool, "logintest", &password_hash, false)
            .await
            .unwrap();

        let config = ApiConfig {
            server_settings: crate::config::ApiSettings::new(
                "127.0.0.1".parse().unwrap(),
                3000,
                db_url,
                vec!["localhost".to_string()],
            ),
            compression: crate::config::ApiCompression::default(),
            jwt_settings: crate::config::JwtSettings::new(5, "test_secret_key".to_string()),
            api_limits: crate::config::ApiLimits::new(5, 10, 30, 1024),
            openapi: crate::config::OpenApiDocs::default(),
        };

        let state = AppState {
            apiconfig: Arc::new(Mutex::new(config)),
            dbpool,
        };

        let app = axum::Router::new()
            .route("/auth/login", axum::routing::post(login))
            .with_state(state);

        let server = TestServer::new(app).unwrap();

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
        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());
        let dbpool = crate::state::init_dbpool(&db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&dbpool).await.unwrap();

        let password_hash = PasswordHelper::hash_password("correctpassword123").unwrap();
        let _user = User::create_user(&dbpool, "wrongpwtest", &password_hash, false)
            .await
            .unwrap();

        let config = ApiConfig {
            server_settings: crate::config::ApiSettings::new(
                "127.0.0.1".parse().unwrap(),
                3000,
                db_url,
                vec!["localhost".to_string()],
            ),
            compression: crate::config::ApiCompression::default(),
            jwt_settings: crate::config::JwtSettings::new(5, "test_secret_key".to_string()),
            api_limits: crate::config::ApiLimits::new(5, 10, 30, 1024),
            openapi: crate::config::OpenApiDocs::default(),
        };

        let state = AppState {
            apiconfig: Arc::new(Mutex::new(config)),
            dbpool,
        };

        let app = axum::Router::new()
            .route("/auth/login", axum::routing::post(login))
            .with_state(state);

        let server = TestServer::new(app).unwrap();

        let login_body = json!({
            "username": "wrongpwtest",
            "password": "wrongpassword123"
        });

        let response = server.post("/auth/login").json(&login_body).await;

        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_uses_dynamic_jwt_expiration() {
        use tempfile::NamedTempFile;

        let temp_db = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());

        let dbpool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&dbpool).await.unwrap();

        // Create test user with properly hashed password
        let password_hash =
            crate::auth::PasswordHelper::hash_password("secure_password_123").unwrap();
        sqlx::query(
            "INSERT INTO users (username, password_hash, is_admin, created_at, updated_at)
             VALUES (?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind("dynamictest")
        .bind(password_hash)
        .bind(true)
        .execute(&dbpool)
        .await
        .unwrap();

        // Create config with custom JWT expiration (10 minutes)
        let config = ApiConfig {
            server_settings: crate::config::ApiSettings::new(
                "127.0.0.1".parse().unwrap(),
                3000,
                db_url,
                vec!["localhost".to_string()],
            ),
            compression: crate::config::ApiCompression::default(),
            jwt_settings: crate::config::JwtSettings::new(10, "test_secret_key".to_string()), // Custom expiration
            api_limits: crate::config::ApiLimits::new(5, 10, 30, 1024),
            openapi: crate::config::OpenApiDocs::default(),
        };

        let state = AppState {
            apiconfig: Arc::new(Mutex::new(config)),
            dbpool,
        };

        let app = axum::Router::new()
            .route("/auth/login", axum::routing::post(login))
            .with_state(state);

        let server = TestServer::new(app).unwrap();

        let login_body = json!({
            "username": "dynamictest",
            "password": "secure_password_123"
        });

        let response = server.post("/auth/login").json(&login_body).await;

        response.assert_status(axum::http::StatusCode::OK);

        let response_json: serde_json::Value = response.json();

        // Verify the expires_in matches our custom 10 minutes (600 seconds)
        assert_eq!(response_json["expires_in"], 600);
        assert!(response_json["token"].is_string());
    }
}
