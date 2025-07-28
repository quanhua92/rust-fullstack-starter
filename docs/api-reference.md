# API Reference

This document provides a comprehensive reference for all available API endpoints in the Rust Full-Stack Starter.

## 📋 Interactive API Documentation

The starter now includes **comprehensive OpenAPI documentation** with interactive features:

### 🌐 Access Documentation
- **API Documentation Hub**: `http://localhost:3000/api-docs`
- **OpenAPI JSON Schema**: `http://localhost:3000/api-docs/openapi.json`
- **Local OpenAPI File**: [`docs/openapi.json`](openapi.json) (exported specification)
- **📋 [Interactive Swagger UI](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/quanhua92/rust-fullstack-starter/refs/heads/main/docs/openapi.json)**

### ✨ Features
- **Complete API Schema**: All endpoints, request/response models, and validation rules
- **Interactive Testing**: Test endpoints directly from the documentation
- **Code Examples**: Request/response examples for all endpoints
- **Authentication Support**: Built-in authentication testing for protected endpoints
- **Type Definitions**: Full TypeScript-style type definitions for all models

### 🚀 Quick Access
The health endpoint now includes documentation links:
```bash
curl http://localhost:3000/health
# Returns documentation URLs in the response
```

### 🗺️ API Endpoint Map

```mermaid
graph TB
    subgraph "🔓 Public Endpoints"
        HEALTH[💓 /health/*<br/>Health checks]
        AUTH_PUB[🔐 /auth/register<br/>🔐 /auth/login]
        TYPES_PUB[🏷️ /tasks/types<br/>GET: List types<br/>POST: Register type]
        DOCS[📚 /api-docs/*<br/>OpenAPI documentation]
    end
    
    subgraph "🔒 Protected Endpoints (Auth Required)"
        AUTH_PROT[🚪 /auth/logout<br/>🚪 /auth/me<br/>🔄 /auth/refresh]
        USERS["👤 /users/{id}<br/>User profiles"]
        TASKS[⚙️ /tasks<br/>📊 /tasks/stats<br/>💀 /tasks/dead-letter]
        TASK_OPS["🔧 /tasks/{id}<br/>GET, DELETE<br/>🔄 /tasks/{id}/retry<br/>🛑 /tasks/{id}/cancel"]
    end
    
    subgraph "👑 Admin Only"
        ADMIN[🔧 /admin/health<br/>Detailed system status]
    end
    
    subgraph "🔑 Authentication Flow"
        BEARER["📝 Authorization: Bearer {token}"]
        MIDDLEWARE[🛡️ Auth Middleware<br/>Session validation]
    end
    
    AUTH_PUB --> BEARER
    BEARER --> MIDDLEWARE
    MIDDLEWARE --> AUTH_PROT
    MIDDLEWARE --> USERS
    MIDDLEWARE --> TASKS
    MIDDLEWARE --> TASK_OPS
    MIDDLEWARE --> ADMIN
    
    classDef publicBox fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef protectedBox fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef adminBox fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef authBox fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    
    class HEALTH,AUTH_PUB,TYPES_PUB,DOCS publicBox
    class AUTH_PROT,USERS,TASKS,TASK_OPS protectedBox
    class ADMIN adminBox
    class BEARER,MIDDLEWARE authBox
```

### 📖 Using the Interactive Docs
1. Start your server: `./scripts/server.sh 3000`
2. Visit: `http://localhost:3000/api-docs`
3. Click "🔧 Swagger UI (External)" for full interactive testing
4. Or download the OpenAPI JSON for use with your preferred API client

---

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
    "uptime": 1234.56,
    "documentation": {
      "openapi_json": "/api-docs/openapi.json",
      "api_docs": "/api-docs"
    }
  },
  "message": null
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

### GET /health/live

Kubernetes liveness probe endpoint. Checks if the application process is alive and responding.

**Authentication**: None required

**Use Case**: Kubernetes liveness probes to detect if the container needs to be restarted.

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "alive",
    "probe": "liveness",
    "timestamp": "2024-01-01T00:00:00Z"
  },
  "message": null
}
```

### GET /health/ready

Kubernetes readiness probe endpoint. Checks if the application is ready to serve traffic by validating all critical dependencies.

**Authentication**: None required

**Use Case**: Kubernetes readiness probes to determine if traffic should be routed to this pod.

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "ready",
    "probe": "readiness",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "database": {
        "status": "healthy",
        "message": "Database connection successful",
        "details": null
      },
      "application": {
        "status": "healthy",
        "message": "Application configuration is valid",
        "details": {
          "config_loaded": true,
          "auth_configured": true
        }
      }
    }
  },
  "message": null
}
```

**Error Response** (503 Service Unavailable if not ready):
```json
{
  "success": true,
  "data": {
    "status": "not_ready",
    "probe": "readiness",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "database": {
        "status": "unhealthy",
        "message": "Database connection failed",
        "details": {
          "error": "Connection refused"
        }
      }
    }
  },
  "message": null
}
```

### GET /health/startup

Kubernetes startup probe endpoint. Checks if the application has completed initialization, including database schema validation.

**Authentication**: None required

**Use Case**: Kubernetes startup probes to determine when the application has finished starting up.

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "started",
    "probe": "startup",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "database": {
        "status": "healthy",
        "message": "Database connection successful",
        "details": null
      },
      "schema": {
        "status": "healthy",
        "message": "Database schema is initialized",
        "details": {
          "tables_exist": true,
          "migrations_applied": true
        }
      }
    }
  },
  "message": null
}
```

**Error Response** (503 Service Unavailable if not started):
```json
{
  "success": true,
  "data": {
    "status": "starting",
    "probe": "startup",
    "timestamp": "2024-01-01T00:00:00Z",
    "checks": {
      "schema": {
        "status": "unhealthy",
        "message": "Database schema not initialized",
        "details": {
          "error": "relation \"users\" does not exist",
          "suggestion": "Ensure database migrations have been applied"
        }
      }
    }
  },
  "message": null
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

Invalidate all sessions for the current user (all devices).

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

**Use Case**: Security feature to log out from all devices if account is compromised.

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

### Task Type Management

Before creating tasks, you must register task types with the API server. This is typically done automatically by workers, but you can also manage task types manually.

### POST /tasks/types

Register a new task type that workers can handle.

**Authentication**: None required (public endpoint for worker registration)

**Request Body**:
```json
{
  "task_type": "email",
  "description": "Email notification tasks"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "task_type": "email",
    "description": "Email notification tasks",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

**Use Case**: Workers automatically call this endpoint on startup to register their capabilities.

**Error Responses**:
- `400` - Invalid request data

### GET /tasks/types

List all registered task types available for task creation.

**Authentication**: None required (public endpoint)

**Response** (200 OK):
```json
{
  "success": true,
  "data": [
    {
      "task_type": "email",
      "description": "Email notification tasks",
      "is_active": true,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    },
    {
      "task_type": "webhook",
      "description": "Webhook notification tasks",
      "is_active": true,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

**Use Case**: Check which task types are available before creating tasks.

## Task Management Endpoints

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

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "task_type": "email",
    "payload": { "to": "recipient@example.com", "subject": "...", "body": "..." },
    "status": "Pending",
    "priority": "Normal",
    "retry_strategy": {
      "Exponential": {
        "base_delay": { "nanos": 0, "secs": 1 },
        "max_attempts": 5,
        "max_delay": { "nanos": 0, "secs": 300 },
        "multiplier": 2.0
      }
    },
    "max_attempts": 5,
    "current_attempt": 0,
    "last_error": null,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "scheduled_at": "2024-01-01T12:00:00Z",
    "started_at": null,
    "completed_at": null,
    "created_by": null,
    "metadata": { "api_created": true }
  },
  "message": null
}
```

**Error Responses**:
- `400` - Invalid payload format
- `401` - Authentication required

**Note**: The API accepts any task type string. Unknown task types will be accepted but will fail during processing if no handler is registered for that type.

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
      "status": "Completed",
      "priority": "Normal",
      "current_attempt": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "completed_at": "2024-01-01T00:01:00Z"
    }
  ],
  "message": null
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
    "status": "Completed",
    "priority": "Normal",
    "retry_strategy": {
      "Exponential": {
        "base_delay": { "nanos": 0, "secs": 1 },
        "max_attempts": 5,
        "max_delay": { "nanos": 0, "secs": 300 },
        "multiplier": 2.0
      }
    },
    "max_attempts": 5,
    "current_attempt": 1,
    "last_error": null,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:01:00Z",
    "scheduled_at": "2024-01-01T12:00:00Z",
    "started_at": "2024-01-01T00:00:30Z",
    "completed_at": "2024-01-01T00:01:00Z",
    "created_by": null,
    "metadata": { "api_created": true }
  },
  "message": null
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

### GET /tasks/dead-letter

Get all failed tasks in the dead letter queue for debugging and manual recovery.

**Authentication**: Required

**Query Parameters**:
- `limit` (optional): Maximum number of tasks to return (default: 100)
- `offset` (optional): Number of tasks to skip for pagination

**Response** (200 OK):
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-here",
      "task_type": "email",
      "status": "Failed",
      "priority": "Normal",
      "current_attempt": 5,
      "max_attempts": 5,
      "last_error": "SMTP connection failed",
      "created_at": "2024-01-01T00:00:00Z",
      "failed_at": "2024-01-01T00:05:00Z"
    }
  ]
}
```

**Use Case**: Monitor failed tasks for debugging and decide which ones to retry manually.

**Error Responses**:
- `401` - Authentication required

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

### POST /tasks/{task_id}/retry

Retry a failed task by resetting it to pending status.

**Authentication**: Required

**Parameters**:
- `task_id` (path): UUID of the task

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Task retried successfully",
  "message": "Task uuid-here has been reset to pending status"
}
```

**Use Case**: Manually retry tasks that failed due to temporary issues (network errors, service downtime).

**Error Responses**:
- `401` - Authentication required
- `404` - Task not found or not in failed status
- `400` - Task is not in failed status

### DELETE /tasks/{task_id}

Permanently delete a completed, failed, or cancelled task.

**Authentication**: Required

**Parameters**:
- `task_id` (path): UUID of the task

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Task deleted successfully",
  "message": "Task uuid-here has been permanently deleted"
}
```

**Use Case**: Clean up old completed/failed tasks to reduce database size and improve performance.

**Error Responses**:
- `401` - Authentication required
- `404` - Task not found
- `400` - Task is not in a deletable status (running tasks cannot be deleted)

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
- `401` - Invalid or expired token, or insufficient permissions (non-admin user)

## HTTP Status Codes

The API uses standard HTTP status codes:

### Success Codes
- `200 OK` - Request successful (includes resource creation)

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

### Comprehensive Testing
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all 40 integration tests (~10 seconds)
cargo nextest run

# Run specific test categories
cargo nextest run auth::     # Authentication tests (6 tests)
cargo nextest run tasks::    # Task system tests (11 tests)
cargo nextest run api::      # API standards tests (13 tests)
cargo nextest run health::   # Health check tests (8 tests)
```

The integration test suite covers:
- ✅ User registration and authentication (6 tests)
- ✅ Task creation and processing (11 tests)
- ✅ Background worker functionality
- ✅ API standards and security headers
- ✅ Health monitoring endpoints
- ✅ Error handling and edge cases

### API Endpoint Testing
```bash
# Test all documented endpoints with curl (29 tests)
./scripts/test-with-curl.sh

# Test custom server configuration
./scripts/test-with-curl.sh localhost 8080
./scripts/test-with-curl.sh api.example.com 443  # HTTPS auto-detected

# Full validation workflow
cargo nextest run && ./scripts/test-with-curl.sh
```

The curl test script validates:
- ✅ All 18 documented API endpoints
- ✅ Input/output formats match documentation exactly
- ✅ Authentication flows and error handling
- ✅ Custom server configurations
- ✅ HTTPS support and protocol detection

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

# Run all API tests (40 integration tests)
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
    "task_type": "email",
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