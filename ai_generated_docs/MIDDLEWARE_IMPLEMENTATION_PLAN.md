# Middleware Implementation Plan

## 📋 Requirements Summary

- **Global middleware**: Apply to all routes uniformly
- **Configuration**: Both `.env` and `config.toml` support
- **Rate limiting**: 5 req/sec per IP (configurable), memory-based
- **JWT expiration**: **MINUTES** (default 5, max 1440 = 24 hours)
- **Security headers**: Minimal essential headers only
- **CORS**: Keep current implementation unchanged
- **Compression**: Already added ✅

## Phase 1: Configuration Extensions ✅ COMPLETED

### 1.1 Extend ApiConfig Structure ✅

```rust
pub struct ApiConfig {
    // ... existing fields ...
    pub jwt_secret: String,
    pub jwt_expiration_minutes: u64,      // NEW: Token expiration in minutes
    pub rate_limit_per_second: u64,       // NEW: Rate limiting
    pub security_headers_enabled: bool,   // NEW: Security headers toggle
}
```

### 1.2 Environment Variables (.env) ✅

```bash
JWT_EXPIRATION_MINUTES=5          # Default 5 minutes (short-lived)
RATE_LIMIT_PER_SECOND=5
SECURITY_HEADERS_ENABLED=true
```

### 1.3 Config File (config.toml) ✅

```toml
jwt_expiration_minutes = 5        # Default 5 minutes
rate_limit_per_second = 5
security_headers_enabled = true
```

### 1.4 JWT Configuration Validation ✅

- **Default**: 5 minutes (very short-lived tokens)
- **Minimum**: 1 minute
- **Maximum**: 1440 minutes (24 hours)
- **Use case**: Short API sessions, frequent re-auth

### 1.5 Implementation Results ✅

- **Configuration parsing**: All methods support new fields
- **Validation**: Range checking implemented for JWT and rate limits
- **Test updates**: All 121 tests passing
- **Sample files**: `.env.example` and `config.toml.example` created
- **Backward compatibility**: Maintained with sensible defaults

## Phase 2: Auth System Updates ✅ COMPLETED

### 2.1 JWT Manager Changes ✅

```rust
impl JwtManager {
    // Updated from hardcoded TOKEN_EXPIRATION_MINUTES to dynamic
    pub fn generate_token(
        user: &User,
        secret: &str,
        expiration_minutes: u64
    ) -> Result<String>

    // Return expiration in seconds for API response
    pub fn get_expiration_seconds(expiration_minutes: u64) -> i64
}
```

### 2.2 Auth Handlers Update ✅

- **Login endpoint**: Now uses `ApiConfig.jwt_expiration_minutes`
- **Token response**: Returns actual expiration time from config
- **Backward compatibility**: Removed hardcoded `TOKEN_EXPIRATION_MINUTES = 5`

### 2.3 Implementation Results ✅

- **JWT Manager**: Updated method signatures to accept dynamic expiration
- **Login handler**: Reads expiration from config and passes to JWT generation
- **Test updates**: All JWT tests updated with expiration parameter
- **Integration test**: Added test verifying dynamic expiration works end-to-end
- **API compatibility**: Login response format unchanged

## Phase 3: Middleware Implementation

### 3.1 Rate Limiting Middleware

- **Library**: `tower-governor`
- **Scope**: Global per-IP limiting
- **Configuration**: `ApiConfig.rate_limit_per_second`
- **Storage**: In-memory HashMap

### 3.2 Security Headers Middleware

**Essential headers only:**

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`

### 3.3 Request Size Limits

- **Body size limit**: 1MB default
- **Applied globally**

## Phase 4: Integration & Structure

### 4.1 Middleware Stack Order (Global)

```rust
Router::new()
    .merge(all_routes)
    .layer(SecurityHeadersLayer)      // Security headers first
    .layer(GovernorLayer)             // Rate limiting per-IP
    .layer(RequestBodyLimitLayer)     // Request size limits
    .layer(CompressionLayer)          // Compression (existing)
    .layer(TraceLayer)                // Tracing last (existing)
```

### 4.2 File Structure

```text
src/
├── middleware/
│   ├── mod.rs              # Middleware configuration & setup
│   ├── rate_limit.rs       # Rate limiting with governor
│   ├── security.rs         # Security headers layer
│   └── limits.rs           # Request size limits
├── auth.rs                 # Remove TOKEN_EXPIRATION_MINUTES constant
├── config.rs               # Add jwt_expiration_minutes field
└── routes/mod.rs           # Apply global middleware stack
```

## Phase 5: Configuration Integration

### 5.1 Config Defaults & Validation

```rust
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            jwt_expiration_minutes: 5,          // Very short-lived tokens
            rate_limit_per_second: 5,
            security_headers_enabled: true,
        }
    }
}
```

### 5.2 Validation Rules

- **JWT expiration**: 1 ≤ minutes ≤ 1440
- **Rate limits**: 1 ≤ requests/second ≤ 1000
- **Security headers**: Boolean toggle

## 🔧 Implementation Summary

### Key Changes from Previous Plan

- ✅ **JWT expiration**: Changed from hours to **minutes**
- ✅ **Default expiration**: **5 minutes** (short-lived sessions)
- ✅ **Maximum expiration**: **1440 minutes** (24 hours)
- ✅ **Auth system**: Remove hardcoded constant, use dynamic config

### Dependencies to Add

```toml
tower-governor = "0.6.2"
tower = { version = "0.5.1", features = ["limit"] }
```

### Configuration Fields to Add

- `jwt_expiration_minutes: u64` (default: 5, max: 1440)
- `rate_limit_per_second: u64` (default: 5)
- `security_headers_enabled: bool` (default: true)

### Auth System Impact

- **Short sessions**: 5-minute default requires more frequent login
- **Security benefit**: Reduced token exposure window
- **API compatibility**: Token expiration still returned in response
- **Configurable**: Can extend to 24 hours for different use cases

## Implementation Order

1. **Phase 1**: ✅ **COMPLETED** - Extend configuration structure and parsing
2. **Phase 2**: ✅ **COMPLETED** - Update JWT system to use dynamic expiration
3. **Phase 3**: 🔄 **NEXT** - Implement middleware components
4. **Phase 4**: ⏳ **PENDING** - Integrate middleware stack in routes
5. **Phase 5**: ⏳ **PENDING** - Test and validate configuration loading

## Testing Strategy

- **Unit tests**: Each middleware component
- **Integration tests**: Middleware stack behavior
- **Configuration tests**: ✅ **COMPLETED** - Validation and defaults (31 tests
  passing)
- **Auth tests**: ✅ **COMPLETED** - Dynamic JWT expiration (8 tests passing)
- **Rate limiting tests**: Per-IP limiting behavior

## Phase 1 Completion Summary

1. **Extended ApiConfig Structure**: Added `jwt_expiration_minutes`,
   `rate_limit_per_second`, `security_headers_enabled`
2. **Configuration Parsing**: Updated all parsing methods (.env, TOML, CLI) to
   support new fields
3. **Validation Logic**: Implemented range checking with proper error messages
4. **Default Values**: Set secure defaults (5min JWT, 5 req/sec, security
   headers enabled)
5. **Test Updates**: Fixed all test code and added comprehensive validation
   tests
6. **Sample Configuration**: Created documented example files for both formats
7. **Backward Compatibility**: Maintained with `#[serde(default)]` attributes

- **Total Tests**: 122 passing (75 unit + 47 integration)
- **Configuration Tests**: 31 passing (including new validation tests)
- **Zero Regressions**: All existing functionality preserved

## Phase 2 Completion Summary

1. **JWT Manager Updates**: Updated `generate_token()` and
   `get_expiration_seconds()` to accept dynamic expiration
2. **Auth Handler Integration**: Login endpoint now reads
   `jwt_expiration_minutes` from `ApiConfig`
3. **Removed Hardcoded Values**: Eliminated `TOKEN_EXPIRATION_MINUTES` constant
4. **Test Updates**: Updated all JWT unit tests and added dynamic expiration
   integration test
5. **API Compatibility**: Maintained existing login response format
6. **Configuration Integration**: JWT system now fully configurable via
   environment/TOML

- **Total Tests**: 122 passing (75 unit + 47 integration)
- **Auth Tests**: 8 passing (including new dynamic expiration test)
- **JWT Integration**: Verified 10-minute config produces 600-second response
- **Zero Regressions**: All existing functionality preserved

### 🎯 Ready for Phase 3

JWT system now uses configurable expiration from `ApiConfig`. Next phase can
safely implement rate limiting, security headers, and request size limit
middleware.

---

**✅ This plan provides production-ready middleware with configurable
short-lived JWT tokens and global rate limiting.**
