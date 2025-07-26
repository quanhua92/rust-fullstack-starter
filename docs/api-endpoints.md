# API Endpoints Reference

This document provides a comprehensive reference for all available API endpoints in the Rust Full-Stack Starter.

## Base URL

**Development**: `http://localhost:3000`  
**Production**: Configure via `STARTER__SERVER__HOST` and `STARTER__SERVER__PORT`

## Response Format

All API responses follow a consistent JSON structure:

```json
{
  "success": true,
  "data": {}, // Response data or null
  "message": "Optional message" // Optional additional message
}
```

### Error Responses

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

## Authentication

Protected endpoints require a `Bearer` token in the `Authorization` header:

```
Authorization: Bearer <session_token>
```

## Health Endpoints

### GET /health

Basic health check endpoint.

**Authentication**: None required

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### GET /health/detailed

Detailed health check including database connectivity.

**Authentication**: None required

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "database": {
        "status": "healthy",
        "message": "Connected to PostgreSQL"
      }
    }
  }
}
```

## Authentication Endpoints

### POST /auth/register

Create a new user account.

**Authentication**: None required

**Request Body**:
```json
{
  "username": "newuser",
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "role": "user" // Optional, defaults to "user"
}
```

**Response** (201 Created):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "newuser",
    "email": "user@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": null
  }
}
```

**Validation Rules**:
- Username: 3-50 characters, alphanumeric with `-` and `_`
- Email: Valid email format, max 254 characters
- Password: Minimum 8 characters, max 128 characters

**Error Responses**:
- `400` - Validation error
- `409` - Username or email already exists

### POST /auth/login

Authenticate user and create session.

**Authentication**: None required

**Request Body**:
```json
{
  "username_or_email": "user@example.com",
  "password": "SecurePassword123!",
  "user_agent": "Mozilla/5.0..." // Optional
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "session_token": "ABc123...64-character-token...",
    "expires_at": "2024-01-02T00:00:00Z",
    "user": {
      "id": "uuid-here",
      "username": "newuser",
      "email": "user@example.com",
      "role": "user"
    }
  }
}
```

**Error Responses**:
- `400` - Validation error
- `401` - Invalid credentials or inactive user

### POST /auth/logout

Invalidate current user session.

**Authentication**: Required

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Logged out successfully",
  "message": "Ended 1 session(s)"
}
```

**Error Responses**:
- `401` - Invalid or expired token

### POST /auth/logout-all

Invalidate all sessions for the current user.

**Authentication**: Required

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Logged out from all devices",
  "message": "Ended 3 session(s)"
}
```

**Error Responses**:
- `401` - Invalid or expired token

### GET /auth/me

Get current user profile.

**Authentication**: Required

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "newuser",
    "email": "user@example.com",
    "role": "user"
  }
}
```

**Error Responses**:
- `401` - Invalid or expired token

### POST /auth/refresh

Validate current session (refresh token).

**Authentication**: Required

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Token is still valid",
  "message": "Current session remains active"
}
```

**Error Responses**:
- `401` - Invalid or expired token

## User Management Endpoints

### GET /users/profile

Get current user's detailed profile.

**Authentication**: Required

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "newuser",
    "email": "user@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T12:00:00Z"
  }
}
```

### GET /users/{user_id}

Get another user's profile (public information only).

**Authentication**: Required

**Parameters**:
- `user_id` (path): UUID of the user

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "otheruser",
    "email": "other@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": true,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T10:00:00Z"
  }
}
```

**Error Responses**:
- `401` - Invalid or expired token
- `404` - User not found

## Admin Endpoints

### GET /admin/health

Admin-only detailed health check.

**Authentication**: Required (Admin role)

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "database": {
        "status": "healthy",
        "message": "Connected to PostgreSQL",
        "details": {
          "pool_size": 10,
          "active_connections": 3
        }
      }
    }
  }
}
```

**Error Responses**:
- `401` - Invalid or expired token
- `403` - Insufficient permissions (non-admin user)

## HTTP Status Codes

The API uses standard HTTP status codes:

### Success Codes
- `200 OK` - Request successful
- `201 Created` - Resource created successfully

### Client Error Codes
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Authentication required or invalid
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists

### Server Error Codes
- `500 Internal Server Error` - Unexpected server error
- `503 Service Unavailable` - Service temporarily unavailable

## Rate Limiting

Currently no rate limiting is implemented. Consider adding rate limiting for production use:

- Authentication endpoints: 5 requests per minute per IP
- General API endpoints: 100 requests per minute per user

## CORS Policy

CORS is configured for development:
- Allowed origins: `http://localhost:5173` (configurable via `STARTER__SERVER__CORS_ORIGINS`)
- Allowed methods: All
- Allowed headers: All

## Request/Response Examples

### Complete Authentication Flow

1. **Register a new user**:
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "SecurePass123!"}'
```

2. **Login to get session token**:
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email": "test@example.com", "password": "SecurePass123!"}'
```

3. **Access protected endpoint**:
```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN_HERE"
```

4. **Logout**:
```bash
curl -X POST http://localhost:3000/auth/logout \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN_HERE"
```

## Testing

Use the provided test script to validate all endpoints:

```bash
./scripts/test_auth.sh
```

This script will:
- ✅ Test user registration
- ✅ Test user login
- ✅ Test protected route access
- ✅ Test unauthorized access blocking
- ✅ Test logout functionality
- ✅ Test invalid credentials

## Error Handling

All endpoints return structured error responses with appropriate HTTP status codes. Common error patterns:

### Validation Errors (400)
```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Validation failed for email: Invalid email format"
  }
}
```

### Authentication Errors (401)
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid credentials"
  }
}
```

### Authorization Errors (403)
```json
{
  "error": {
    "code": "FORBIDDEN", 
    "message": "Insufficient permissions"
  }
}
```

### Not Found Errors (404)
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "User not found"
  }
}
```

## API Versioning

Currently, the API does not use versioning. For future versions, consider:
- URL path versioning: `/v1/auth/login`
- Header versioning: `Accept: application/vnd.api+json;version=1`

## Security Considerations

- All passwords are hashed with Argon2
- Session tokens are 64-character cryptographically secure strings
- Sessions expire after 24 hours
- Use HTTPS in production
- Implement rate limiting for production use
- Consider adding request logging and monitoring