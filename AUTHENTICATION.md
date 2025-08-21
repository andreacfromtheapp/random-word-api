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

### OpenAPI Documentation (Require Authentication, except redoc)

- `GET /swagger-ui/*` - Swagger UI (protected)
- `GET /scalar/*` - Scalar documentation (protected)
- `GET /rapidoc/*` - RapiDoc documentation (protected)
- `GET /redoc/*` - ReDoc documentation (public)

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
curl -X POST http://localhost:3000/auth/login \
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
curl -X GET http://localhost:3000/admin/en/words \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

```bash
# Create a new word (admin only)
curl -X POST http://localhost:3000/admin/en/words \
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
# Update a word (admin only)
curl -X PUT http://localhost:3000/admin/en/words/1 \
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
# Delete a word (admin only)
curl -X DELETE http://localhost:3000/admin/en/words/1 \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

### 3. Accessing Public Endpoints

No authentication required:

```bash
# Get a random word
curl -X GET http://localhost:3000/en/words/random

# Get a random noun
curl -X GET http://localhost:3000/en/words/random/noun

# Health check
curl -X GET http://localhost:3000/health
```

### 4. Accessing Protected Documentation

```bash
# Swagger UI (requires authentication)
curl -X GET http://localhost:3000/swagger-ui \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

# ReDoc (public access)
curl -X GET http://localhost:3000/redoc
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

## Security Best Practices

### 1. JWT Secret Management

- Use a strong, randomly generated secret (minimum 32 characters)
- Store the secret securely (environment variables, secret management systems)
- Rotate the secret periodically in production

### 2. Password Requirements

- Minimum 6 characters (consider increasing in production)
- Use strong passwords with mixed case, numbers, and symbols
- Hash passwords with Argon2id before storing

### 3. Token Security

- Tokens expire after 24 hours by default
- Store tokens securely on the client side
- Implement proper token refresh mechanisms

### 4. Admin Account Management

- Create admin accounts manually through database access
- Regularly audit admin permissions
- Use principle of least privilege
- Monitor admin activity

## Integration Examples

### JavaScript/Fetch API

```javascript
// Login and store token
async function login(username, password) {
  const response = await fetch("/auth/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ username, password }),
  });

  if (response.ok) {
    const data = await response.json();
    localStorage.setItem("jwt_token", data.token);
    return data;
  } else {
    throw new Error("Login failed");
  }
}

// Use token for admin requests
async function createWord(word, definition, pronunciation, wordType) {
  const token = localStorage.getItem("jwt_token");

  const response = await fetch("/admin/en/words", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({
      word,
      definition,
      pronunciation,
      wordType,
    }),
  });

  if (response.ok) {
    return await response.json();
  } else if (response.status === 401) {
    throw new Error("Authentication required");
  } else if (response.status === 403) {
    throw new Error("Admin privileges required");
  } else {
    throw new Error("Request failed");
  }
}
```

### Python/Requests

```python
import requests

# Login and get token
def login(username, password):
    response = requests.post('/auth/login', json={
        'username': username,
        'password': password
    })

    if response.status_code == 200:
        return response.json()
    else:
        raise Exception('Login failed')

# Use token for admin requests
def create_word(token, word, definition, pronunciation, word_type):
    headers = {'Authorization': f'Bearer {token}'}
    data = {
        'word': word,
        'definition': definition,
        'pronunciation': pronunciation,
        'wordType': word_type
    }

    response = requests.post('/admin/en/words', json=data, headers=headers)

    if response.status_code == 200:
        return response.json()
    elif response.status_code == 401:
        raise Exception('Authentication required')
    elif response.status_code == 403:
        raise Exception('Admin privileges required')
    else:
        raise Exception('Request failed')
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
