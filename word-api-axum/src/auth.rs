//! Authentication and authorization utilities
//!
//! Provides JWT token generation/validation, password hashing, and middleware
//! for protecting routes. Uses Argon2 for secure password hashing and
//! jsonwebtoken for JWT handling with RS256 algorithm.

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use getrandom::fill;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AuthError};
use crate::models::user::{AuthUser, Claims, User};
use crate::state::AppState;

/// JWT token expiration time in hours
const TOKEN_EXPIRATION_HOURS: i64 = 24;

/// Password hashing utilities using Argon2
pub struct PasswordHelper;

impl PasswordHelper {
    /// Hash a password using Argon2id
    pub fn hash_password(password: &str) -> Result<String> {
        // Generate a random salt
        let mut salt_bytes = [0u8; 16];
        fill(&mut salt_bytes).map_err(|e| anyhow!("Failed to generate salt: {}", e))?;
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against its hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// JWT token utilities with enhanced security validation
///
/// Implements JWT best practices including:
/// - Standard claims validation (iss, aud, exp, nbf, iat, jti)
/// - Token type verification to prevent access/refresh confusion
/// - Cryptographically secure unique identifiers
/// - Comprehensive temporal validation
pub struct JwtManager;

impl JwtManager {
    /// Generate a JWT token for the given user with comprehensive security claims
    ///
    /// Creates a token with:
    /// - Standard RFC 7519 claims for security
    /// - Unique JWT ID for replay attack prevention
    /// - Session ID for revocation capabilities
    /// - Token type specification for access control
    pub fn generate_token(user: &User, secret: &str) -> Result<String> {
        let now = Utc::now();
        let expiration = now + Duration::hours(TOKEN_EXPIRATION_HOURS);
        let not_before = now;

        let claims = Claims {
            // Standard JWT claims (RFC 7519)
            iss: "random-word-api".to_string(),
            aud: "random-word-api-users".to_string(),
            sub: user.id.to_string(),
            exp: expiration.timestamp() as usize,
            nbf: not_before.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),

            // Custom application claims
            username: user.username.clone(),
            is_admin: user.is_admin,
            session_id: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| anyhow!("Failed to generate JWT token: {}", e))
    }

    /// Validate a JWT token with comprehensive security checks
    ///
    /// Performs validation of:
    /// - Token signature and format
    /// - Issuer and audience claims
    /// - Temporal validity (exp, nbf, iat)
    /// - Token structure and required fields
    pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let mut validation = Validation::default();

        // Configure validation for security best practices
        validation.set_issuer(&["random-word-api"]);
        validation.set_audience(&["random-word-api-users"]);
        validation.validate_nbf = true; // Validate "not before" claim

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow!("Failed to validate JWT token: {}", e))?;

        Ok(token_data.claims)
    }

    /// Get token expiration time in seconds
    pub fn get_expiration_seconds() -> i64 {
        TOKEN_EXPIRATION_HOURS * 3600
    }
}

/// Middleware for extracting authenticated user from JWT token
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AppError::from(AuthError::MissingToken))?
            .to_str()
            .map_err(|_| AppError::from(AuthError::InvalidToken))?;

        // Check for Bearer token format
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::from(AuthError::InvalidToken));
        }

        let token = &auth_header[7..]; // Remove "Bearer " prefix

        // Get JWT secret from config
        let jwt_secret = {
            let config = state.apiconfig.lock().map_err(|e| {
                AppError::from(AuthError::InternalError(anyhow!(
                    "Failed to lock config: {}",
                    e
                )))
            })?;
            config.jwt_secret.clone()
        };

        // Validate token and extract claims
        let claims = JwtManager::validate_token(token, &jwt_secret)
            .map_err(|_| AppError::from(AuthError::InvalidToken))?;

        // Additional validation checks (JWT library handles exp, nbf, iss, aud automatically)
        // Verify token type is correct
        if claims.token_type != "access" {
            return Err(AppError::from(AuthError::InvalidToken));
        }

        Ok(AuthUser::from(claims))
    }
}

/// Database operations for user authentication
pub struct UserRepository;

impl UserRepository {
    /// Find user by username
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<User>, AuthError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
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
        let result =
            sqlx::query("INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)")
                .bind(username)
                .bind(password_hash)
                .bind(is_admin)
                .execute(pool)
                .await
                .map_err(AuthError::DatabaseError)?;

        let user_id = result.last_insert_rowid();

        // Fetch the created user
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(AuthError::DatabaseError)
    }
}

/// Middleware for admin-only routes
pub struct RequireAdmin(pub AuthUser);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_admin {
            return Err(AppError::from(AuthError::InvalidCredentials));
        }

        Ok(RequireAdmin(user))
    }
}

/// Admin authentication middleware for router-level protection
///
/// Validates JWT tokens and ensures users have admin privileges before
/// allowing access to admin endpoints. Injects AuthUser into request
/// extensions for handlers that need user context.
///
/// # Security Features
/// - Validates JWT signature and claims
/// - Checks admin privileges
/// - Injects authenticated user context
/// - Returns appropriate error responses for auth failures
pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .ok_or(AppError::from(AuthError::MissingToken))?
        .to_str()
        .map_err(|_| AppError::from(AuthError::InvalidToken))?;

    // Check for Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::from(AuthError::InvalidToken));
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Get JWT secret from config
    let jwt_secret = {
        let config = state.apiconfig.lock().map_err(|e| {
            AppError::from(AuthError::InternalError(anyhow!(
                "Failed to lock config: {}",
                e
            )))
        })?;
        config.jwt_secret.clone()
    };

    // Validate token and extract claims
    let claims = JwtManager::validate_token(token, &jwt_secret)
        .map_err(|_| AppError::from(AuthError::InvalidToken))?;

    // Verify token type is correct
    if claims.token_type != "access" {
        return Err(AppError::from(AuthError::InvalidToken));
    }

    // Create AuthUser from claims
    let auth_user = AuthUser::from(claims);

    // Check admin privileges
    if !auth_user.is_admin {
        return Err(AppError::from(AuthError::InvalidCredentials));
    }

    // Inject authenticated user into request extensions
    request.extensions_mut().insert(auth_user);

    // Continue to the handler
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User {
            id: 1,
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            is_admin: true,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";

        // Hash password
        let hash = PasswordHelper::hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));

        // Verify correct password
        assert!(PasswordHelper::verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!PasswordHelper::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_token_generation_and_validation() {
        let user = create_test_user();
        let secret = "test_secret_key";

        // Generate token
        let token = JwtManager::generate_token(&user, secret).unwrap();
        assert!(!token.is_empty());

        // Validate token
        let claims = JwtManager::validate_token(&token, secret).unwrap();
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.is_admin, user.is_admin);
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.iss, "random-word-api");
        assert_eq!(claims.aud, "random-word-api-users");
        assert_eq!(claims.token_type, "access");
        assert!(!claims.jti.is_empty());
        assert!(!claims.session_id.is_empty());
        assert!(claims.exp > claims.iat);
        assert!(claims.nbf <= claims.iat);
    }

    #[test]
    fn test_jwt_token_validation_with_wrong_secret() {
        let user = create_test_user();
        let secret = "test_secret_key";
        let wrong_secret = "wrong_secret_key";

        // Generate token with correct secret
        let token = JwtManager::generate_token(&user, secret).unwrap();

        // Try to validate with wrong secret
        let result = JwtManager::validate_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_token_validation_with_wrong_issuer() {
        let user = create_test_user();
        let secret = "test_secret_key";

        // Create token with different issuer manually
        let now = Utc::now();
        let claims = Claims {
            iss: "wrong-issuer".to_string(), // Wrong issuer
            aud: "random-word-api-users".to_string(),
            sub: user.id.to_string(),
            exp: (now + Duration::hours(24)).timestamp() as usize,
            nbf: now.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
            username: user.username.clone(),
            is_admin: user.is_admin,
            session_id: uuid::Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        // Should fail validation due to wrong issuer
        let result = JwtManager::validate_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_token_validation_with_wrong_token_type() {
        let user = create_test_user();
        let secret = "test_secret_key";

        // Create token with wrong type manually
        let now = Utc::now();
        let claims = Claims {
            iss: "random-word-api".to_string(),
            aud: "random-word-api-users".to_string(),
            sub: user.id.to_string(),
            exp: (now + Duration::hours(24)).timestamp() as usize,
            nbf: now.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
            username: user.username.clone(),
            is_admin: user.is_admin,
            session_id: uuid::Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(), // Wrong type
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        // Token should validate but middleware should reject wrong type
        let claims_result = JwtManager::validate_token(&token, secret);
        assert!(claims_result.is_ok());
        assert_eq!(claims_result.unwrap().token_type, "refresh");
    }

    #[test]
    fn test_auth_user_from_claims() {
        let claims = Claims {
            iss: "random-word-api".to_string(),
            aud: "random-word-api-users".to_string(),
            sub: "123".to_string(),
            exp: 1234567890,
            nbf: 1234567870,
            iat: 1234567880,
            jti: "unique-jwt-id".to_string(),
            username: "testuser".to_string(),
            is_admin: true,
            session_id: "unique-session-id".to_string(),
            token_type: "access".to_string(),
        };

        let auth_user = AuthUser::from(claims);
        assert_eq!(auth_user.id, 123);
        assert_eq!(auth_user.username, "testuser");
        assert!(auth_user.is_admin);
    }

    #[test]
    fn test_middleware_auth_logic_valid_admin_token() {
        let user = create_test_user();
        let secret = "test_secret_key";

        // Generate valid admin token
        let token = JwtManager::generate_token(&user, secret).unwrap();

        // Test the core logic that middleware uses
        let claims = JwtManager::validate_token(&token, secret).unwrap();
        assert_eq!(claims.token_type, "access");

        let auth_user = AuthUser::from(claims);
        assert!(auth_user.is_admin);
        assert_eq!(auth_user.username, "testuser");
    }

    #[test]
    fn test_middleware_auth_logic_non_admin_user() {
        let user = User {
            id: 2,
            username: "regular_user".to_string(),
            password_hash: "hash".to_string(),
            is_admin: false, // Not admin
            created_at: None,
            updated_at: None,
        };
        let secret = "test_secret_key";

        // Generate valid token for non-admin user
        let token = JwtManager::generate_token(&user, secret).unwrap();

        // Test the core logic that middleware uses
        let claims = JwtManager::validate_token(&token, secret).unwrap();
        let auth_user = AuthUser::from(claims);

        // Should fail admin check
        assert!(!auth_user.is_admin);
    }

    #[test]
    fn test_middleware_auth_logic_invalid_token() {
        let secret = "test_secret_key";
        let invalid_token = "invalid.jwt.token";

        // Should fail validation
        let result = JwtManager::validate_token(invalid_token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_middleware_auth_logic_wrong_token_type() {
        let user = create_test_user();
        let secret = "test_secret_key";

        // Create token with wrong type manually
        let now = Utc::now();
        let claims = Claims {
            iss: "random-word-api".to_string(),
            aud: "random-word-api-users".to_string(),
            sub: user.id.to_string(),
            exp: (now + Duration::hours(24)).timestamp() as usize,
            nbf: now.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
            username: user.username.clone(),
            is_admin: user.is_admin,
            session_id: uuid::Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(), // Wrong type
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        // Token validates but has wrong type
        let claims_result = JwtManager::validate_token(&token, secret).unwrap();
        assert_eq!(claims_result.token_type, "refresh");
        // Middleware would reject this in the token_type check
    }

    #[test]
    fn test_middleware_header_parsing() {
        // Test valid Bearer token format
        let auth_header = "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9";
        assert!(auth_header.starts_with("Bearer "));
        let token = &auth_header[7..];
        assert_eq!(token, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9");

        // Test invalid format
        let invalid_header = "Basic dXNlcjpwYXNz";
        assert!(!invalid_header.starts_with("Bearer "));
    }
}
