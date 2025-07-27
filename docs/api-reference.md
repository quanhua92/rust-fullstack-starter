# API Reference

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
    "version": "0.1.0",
    "uptime": 1234.56
  },
  "message": null
}
```

### GET /health/detailed

Detailed health check including database connectivity.

**Authentication**: Admin role required

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

## Task Management Endpoints

Task endpoints demonstrate background job processing patterns.

### POST /tasks

Create a background task for async processing.

**Authentication**: Required

**Request Body**:
```json
{
  "task_type": "email",
  "payload": {
    "to": "recipient@example.com",
    "subject": "Hello",
    "body": "Example email task"
  },
  "priority": "normal"
}
```

**Available Task Types**:
- `email` - Example email notifications
- `data_processing` - Simple data operations (sum, count)
- `webhook` - HTTP request examples
- `file_cleanup` - File management examples  
- `report_generation` - Report creation examples

**Response** (201 Created):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "task_type": "email",
    "payload": { "to": "recipient@example.com", "subject": "...", "body": "..." },
    "status": "pending",
    "priority": "normal",
    "max_attempts": 3,
    "current_attempt": 0,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "scheduled_at": "2024-01-01T12:00:00Z",
    "created_by": "uuid-here",
    "metadata": { "source": "api" }
  }
}
```

**Error Responses**:
- `400` - Invalid task type or payload
- `401` - Authentication required

### GET /tasks

List your background tasks.

**Authentication**: Required

**Query Parameters**:
- `task_type` (optional): Filter by task type
- `status` (optional): Filter by status  
- `limit` (optional): Number of results (default: 100)

**Response** (200 OK):
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-here",
      "task_type": "email",
      "status": "completed",
      "priority": "normal",
      "current_attempt": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "completed_at": "2024-01-01T00:01:00Z"
    }
  ]
}
```

### GET /tasks/{task_id}

Get details about a specific task.

**Authentication**: Required

**Parameters**:
- `task_id` (path): UUID of the task

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "task_type": "email",
    "payload": { "to": "recipient@example.com", "subject": "...", "body": "..." },
    "status": "completed",
    "priority": "normal",
    "max_attempts": 3,
    "current_attempt": 1,
    "last_error": null,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:01:00Z",
    "started_at": "2024-01-01T00:00:30Z",
    "completed_at": "2024-01-01T00:01:00Z",
    "created_by": "uuid-here",
    "metadata": { "source": "api" }
  }
}
```

**Error Responses**:
- `401` - Authentication required
- `404` - Task not found

### GET /tasks/stats

Get basic task statistics.

**Authentication**: Required

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "total": 150,
    "pending": 5,
    "running": 2,
    "completed": 140,
    "failed": 2,
    "cancelled": 1,
    "retrying": 0
  }
}
```

### POST /tasks/{task_id}/cancel

Cancel a pending or retrying task.

**Authentication**: Required

**Parameters**:
- `task_id` (path): UUID of the task

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Task cancelled successfully",
  "message": "Task uuid-here has been cancelled"
}
```

**Error Responses**:
- `401` - Authentication required
- `404` - Task not found
- `400` - Task cannot be cancelled (already completed/running)

## Task Types Reference

These are example task types to demonstrate different background job patterns:

### Email Task (`email`)
```json
{
  "to": "user@example.com",
  "subject": "Hello",
  "body": "Example message"
}
```

### Data Processing Task (`data_processing`)
```json
{
  "operation": "sum",
  "data": [1, 2, 3, 4, 5]
}
```

### Other Task Types
- `webhook` - HTTP request examples
- `file_cleanup` - File management examples
- `report_generation` - Report creation examples

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

Use the provided test scripts to validate all endpoints:

### Authentication Testing
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

### System Testing
```bash
./scripts/test_tasks_integration.sh
```

This script tests:
- Authentication flow
- Task creation and processing  
- Background worker functionality
- Basic statistics

### Manual Task Testing
```bash
# 1. Start services
./scripts/server.sh 3000
./scripts/worker.sh

# 2. Register and login
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'

TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"testuser","password":"password123"}' \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])")

# 3. Create and monitor task
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {
      "to": "test@example.com",
      "subject": "Test Email",
      "body": "Hello from background worker!"
    },
    "priority": "normal"
  }'

# 4. Check task statistics
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/tasks/stats
```

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

## Testing the API

This starter includes comprehensive integration tests that demonstrate proper API usage patterns.

### Running Integration Tests

```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all API tests (38 integration tests)
cargo nextest run

# Run specific test categories
cargo nextest run auth::
cargo nextest run tasks::
cargo nextest run health::
cargo nextest run api::
```

### Testing Patterns

The integration tests demonstrate:

#### Authentication Flow Testing
```bash
# Test registration
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# Test login and extract token
TOKEN=$(curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email": "testuser", "password": "SecurePass123!"}' \
  | jq -r '.data.session_token')
```

#### Protected Endpoint Testing
```bash
# Test protected endpoint with authentication
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer $TOKEN"

# Test task creation (requires auth)
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "send_email",
    "payload": {"to": "test@example.com", "subject": "Test", "body": "Hello"}
  }'
```

#### Error Response Testing
```bash
# Test validation errors
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "", "email": "invalid", "password": "weak"}'

# Test authentication errors
curl -X GET http://localhost:3000/auth/me
```

### Test Database Isolation

Each test runs in complete isolation:
- **Template Database**: Fast setup using PostgreSQL templates (10x speedup)
- **Per-Test Databases**: Each test gets its own database instance
- **Automatic Cleanup**: Test databases are automatically cleaned up

### API Standards Tested

The integration tests verify:
- **Response Format**: Consistent JSON structure across all endpoints
- **Security Headers**: Proper security headers on all responses
- **CORS Configuration**: Cross-origin request handling
- **Error Handling**: Proper error codes and messages
- **Authentication**: Token-based auth flow
- **Authorization**: Role-based access control

See `starter/tests/README.md` for detailed testing documentation.

## Security Considerations

- All passwords are hashed with Argon2
- Session tokens are 64-character cryptographically secure strings
- Sessions expire after 24 hours
- Use HTTPS in production
- Implement rate limiting for production use
- Security headers included: `X-Content-Type-Options`, `X-Frame-Options`
- CORS configured for development (restrict for production)
- Consider adding request logging and monitoring