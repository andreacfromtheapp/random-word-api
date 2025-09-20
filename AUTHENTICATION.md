# JWT Authentication Guide

This document explains how to use the JWT authentication system implemented for
the Random Word API.

## Overview

The API implements secure JWT (JSON Web Token) authentication for admin
endpoints and OpenAPI documentation. This follows JWT best practices including:

- Argon2 password hashing for secure credential storage
- HS256 JWT tokens with configurable expiration (24 hours default)
- Role-based access control (admin vs regular users)
- Secure random salt generation using getrandom
- **Manual user management** (no public registration for security)

## Protected Endpoints

### Admin Endpoints (Require Admin Authentication)

- `GET /admin/{lang}/words` - List all words
- `POST /admin/{lang}/words` - Create new word
- `GET /admin/{lang}/words/{id}` - Get word by ID
- `PUT /admin/{lang}/words/{id}` - Update word by ID
- `DELETE /admin/{lang}/words/{id}` - Delete word by ID

### OpenAPI Documentation (Nginx Authorization Required)

- `GET /swagger-ui/*` - Swagger UI
- `GET /scalar/*` - Scalar documentation
- `GET /rapidoc/*` - RapiDoc documentation
- `GET /redoc/*` - ReDoc documentation

### Public Endpoints (No Authentication Required)

- `POST /auth/login` - User login
- `GET /health` - Health check
- `GET /{lang}/words/random` - Get random word
- `GET /{lang}/words/random/{type}` - Get random word by type

## Configuration

### JWT Secret

Set the JWT secret via environment variable or configuration file:

```bash
# Environment variable
export JWT_SECRET="your-secret-key-change-in-production"

# Or in config.toml
jwt_secret = "your-secret-key-change-in-production"
```

**Important**: Use a strong, randomly generated secret in production!

## User Management

### Creating Users (Database Administration)

Since public registration is disabled for security, users must be created
directly in the database:

```sql
-- Create an admin user
INSERT INTO users (username, password_hash, is_admin)
VALUES (
    'admin_user',
    '$argon2id$v=19$m=19456,t=2,p=1$SALT$HASH',  -- Use proper Argon2 hash
    true
);

-- Create a regular user
INSERT INTO users (username, password_hash, is_admin)
VALUES (
    'regular_user',
    '$argon2id$v=19$m=19456,t=2,p=1$SALT$HASH',  -- Use proper Argon2 hash
    false
);
```

### Generating Password Hashes

Use a tool to generate Argon2 hashes for passwords:

```bash
# Example using argon2 CLI tool
echo -n "your_password" | argon2 some_salt -id -t 2 -m 19 -p 1

# Or use online Argon2 generators (for development only)
# For production, use secure server-side generation
```

### Database Access

Connect to your SQLite database:

```bash
# If using SQLite file
sqlite3 word_api.db

# Check existing users
SELECT username, is_admin, created_at FROM users;
```

## Usage Examples

### 1. User Login

```bash
curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin_user",
    "password": "unsafe_password"
  }'
```

Response:

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "username": "admin_user",
    "isAdmin": true
  }
}
```

### 2. Accessing Protected Admin Endpoints

Use the token from login in the Authorization header:

```bash
# List all words (admin only)
curl -s -X GET http://localhost:3000/admin/en/words \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

```bash
# Create a new word (admin only)
curl -s -X POST http://localhost:3000/admin/en/words \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "word": "example",
    "definition": "an instance or illustration",
    "pronunciation": "/ɪɡˈzæmpəl/",
    "wordType": "noun"
  }'
```

```bash
# Get a word by ID (admin only)
curl -s -X GET http://localhost:3000/admin/en/words/1 \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

```bash
# Update a word by ID (admin only)
curl -s -X PUT http://localhost:3000/admin/en/words/1 \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "word": "updated_example",
    "definition": "an updated instance or illustration",
    "pronunciation": "/ʌpˈdeɪtɪd ɪɡˈzæmpəl/",
    "wordType": "noun"
  }'
```

```bash
# Delete a word by ID (admin only)
curl -s -X DELETE http://localhost:3000/admin/en/words/1 \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

### 3. Accessing Public Endpoints

No authentication required:

```bash
# Get a random word
curl -s -X GET http://localhost:3000/en/random

# Get a random noun
curl -s -X GET http://localhost:3000/en/noun

# Health check
curl -s -X GET http://localhost:3000/health
```

## Error Responses

### Authentication Errors

```json
// Missing token
{
  "error": "Unauthorized"
}

// Invalid token
{
  "error": "Invalid token"
}

// Expired token
{
  "error": "Token expired"
}

// Insufficient permissions (non-admin accessing admin endpoint)
{
  "error": "Forbidden"
}
```

### Login Errors

```json
// Invalid credentials
{
  "error": "Invalid credentials"
}

// Missing fields
{
  "error": "Username and password required"
}
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized on admin endpoints**: Ensure you're including the JWT
   token in the Authorization header
2. **403 Forbidden on admin endpoints**: Check that the user account has
   `is_admin: true`
3. **Token expired**: Login again to get a new token
4. **Invalid token format**: Ensure the token is prefixed with "Bearer "
5. **No users exist**: Create users manually in the database

### Debug Steps

1. Verify the JWT secret is correctly configured
2. Check that the Authorization header format is:
   `Authorization: Bearer <token>`
3. Ensure the token hasn't expired (24 hour default)
4. Verify the user exists and has admin privileges for admin endpoints
5. Check database connection and user table structure

### User Creation Troubleshooting

```sql
-- Check if users table exists
.schema users

-- Check existing users
SELECT * FROM users;

-- Verify password hash format
SELECT username, password_hash FROM users WHERE username = 'your_username';
```

For additional support, check the server logs for detailed error messages.
