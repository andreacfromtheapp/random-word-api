# JWT Authentication Guide

This document explains how to use the JWT authentication system implemented for
the Random Word API.

## Overview

The API now implements secure JWT (JSON Web Token) authentication for admin and
OpenAPI documentation endpoints. This follows JWT best practices including:

- Argon2 password hashing for secure credential storage
- HS256 JWT tokens with configurable expiration (24 hours default)
- Role-based access control (admin vs regular users)
- Secure random salt generation using getrandom

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
- `POST /auth/register` - User registration
- `GET /health/alive` - Health check
- `GET /health/ready` - Readiness check
- `GET /{lang}/random` - Get random word
- `GET /{lang}/{type}` - Get random word by type

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

## Usage Examples

### 1. User Registration

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin_user",
    "password": "secure_password_123",
    "is_admin": true
  }'
```

Response:

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": 86400
}
```

### 2. User Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin_user",
    "password": "secure_password_123"
  }'
```

Response:

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": 86400
}
```

### 3. Accessing Protected Admin Endpoints

Use the token from login/registration in the Authorization header:

```bash
# List all words (admin only)
curl -X GET http://localhost:3000/admin/en/words \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

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

# Delete a word (admin only)
curl -X DELETE http://localhost:3000/admin/en/words/1 \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJHUzI1NiJ9..."
```

### 4. Accessing Public Endpoints

No authentication required:

```bash
# Get a random word
curl -X GET http://localhost:3000/en/random

# Get a random noun
curl -X GET http://localhost:3000/en/noun

# Health check
curl -X GET http://localhost:3000/health/alive
```

### 5. Accessing Protected Documentation

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
  "error": "Missing authorization token"
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
  "error": "Invalid credentials"
}
```

### Validation Errors

```json
// Invalid registration data
{
  "error": "Validation failed",
  "details": {
    "username": ["Username must be at least 3 characters long"],
    "password": ["Password must be at least 6 characters long"]
  }
}

// Username already exists
{
  "error": "Username already exists"
}

// Invalid login credentials
{
  "error": "Invalid credentials"
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
- Consider implementing password complexity requirements

### 3. Token Security

- Tokens expire after 24 hours by default
- Store tokens securely on the client side (secure storage, not localStorage)
- Implement token refresh mechanisms for long-running applications

### 4. Admin Account Management

- Create admin accounts carefully
- Regularly audit admin permissions
- Use principle of least privilege

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
    return data.token;
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
        return response.json()['token']
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
    else:
        raise Exception('Request failed')
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized on admin endpoints**: Ensure you're including the JWT
   token in the Authorization header
2. **Token expired**: Login again to get a new token
3. **Invalid token format**: Ensure the token is prefixed with "Bearer "
4. **Non-admin user accessing admin endpoints**: Check that the user account has
   `is_admin: true`

### Debug Steps

1. Verify the JWT secret is correctly configured
2. Check that the Authorization header format is:
   `Authorization: Bearer <token>`
3. Ensure the token hasn't expired (24 hour default)
4. Verify the user has admin privileges for admin endpoints

For additional support, check the server logs for detailed error messages.
