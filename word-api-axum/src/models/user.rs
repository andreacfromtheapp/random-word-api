//! User model and authentication data structures
//!
//! Defines the core user types for authentication and authorization,
//! including database representations and request/response DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;
use utoipa::ToSchema;
use validator::Validate;

use crate::error::AuthError;

/// Database representation of a user
#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Database operations for user authentication
impl User {
    /// Find user by username
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<Self>, AuthError> {
        sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(AuthError::DatabaseError)
    }

    /// Create a new user
    pub async fn create_user(
        pool: &SqlitePool,
        username: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> Result<User, AuthError> {
        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, is_admin) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(username)
        .bind(password_hash)
        .bind(is_admin)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        let user_id = result.last_insert_rowid();

        // Fetch the created user
        sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(AuthError::DatabaseError)
    }
}

/// User login request data
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

/// User registration request data
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub is_admin: Option<bool>,
}

/// Authentication response containing JWT token
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: i64,
}

/// JWT token claims following RFC 7519 standards and security best practices
///
/// This structure implements comprehensive JWT security measures:
///
/// ## Standard Claims (RFC 7519)
/// - **iss (issuer)**: Prevents token confusion attacks by identifying our API
/// - **aud (audience)**: Prevents tokens from being used with unintended services
/// - **exp (expiration)**: Limits token lifetime to reduce exposure window
/// - **nbf (not before)**: Prevents premature token usage
/// - **iat (issued at)**: Provides token creation timestamp for auditing
/// - **jti (JWT ID)**: Enables token revocation and prevents replay attacks
///
/// ## Security Benefits
/// - **Token Confusion Prevention**: `iss` and `aud` prevent cross-service token abuse
/// - **Replay Attack Mitigation**: Unique `jti` for each token prevents reuse
/// - **Granular Time Control**: `nbf` adds additional temporal security layer
/// - **Session Management**: `session_id` enables proper logout and token invalidation
/// - **Token Type Safety**: `token_type` prevents access/refresh token confusion
///
/// ## Implementation Notes
/// - All timestamps use Unix epoch format for consistency
/// - UUIDs provide cryptographically strong unique identifiers
/// - Validation occurs automatically during token verification
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // Standard JWT claims (RFC 7519)
    /// Issuer - identifies the service that issued this token
    /// Prevents token confusion attacks across different services
    pub iss: String,

    /// Audience - specifies the intended recipients of this token
    /// Prevents tokens from being used with unintended services
    pub aud: String,

    /// Subject - the user ID this token represents
    pub sub: String,

    /// Expiration time - Unix timestamp when token becomes invalid
    /// Limits exposure window if token is compromised
    pub exp: usize,

    /// Not before - Unix timestamp before which token is invalid
    /// Prevents premature use of tokens
    pub nbf: usize,

    /// Issued at - Unix timestamp when token was created
    /// Enables token age validation and audit trails
    pub iat: usize,

    /// JWT ID - unique identifier for this specific token
    /// Enables token revocation and prevents replay attacks
    pub jti: String,

    // Custom application claims
    /// Username for display and logging purposes
    pub username: String,

    /// Administrative privileges flag
    pub is_admin: bool,

    /// Session identifier for token revocation capabilities
    /// Allows proper logout functionality and session management
    pub session_id: String,

    /// Token type ("access" or "refresh") to prevent confusion
    /// Ensures tokens are used for their intended purpose
    pub token_type: String,
}

/// Authenticated user context extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub.parse().unwrap_or(0),
            username: claims.username,
            is_admin: claims.is_admin,
        }
    }
}
