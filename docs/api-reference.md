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
- **Interactive Testing**: Test endpoints directly from the documentation with built-in Bearer token authentication
- **Code Examples**: Request/response examples for all endpoints
- **Bearer Authentication**: Properly defined security scheme for all protected endpoints
- **Environment Variables**: Support for manual token configuration in API testing tools
- **Type Definitions**: Full TypeScript-style type definitions for all models

### 🚀 Quick Access
The health endpoint now includes documentation links:
```bash
curl http://localhost:3000/api/v1/health
# Returns documentation URLs in the response
```

### 🗺️ API Endpoint Map

```mermaid
graph TB
    subgraph "🔓 Public Endpoints"
        HEALTH[💓 /api/v1/health/*<br/>Health checks]
        AUTH_PUB[🔐 /api/v1/auth/register<br/>🔐 /api/v1/auth/login]
        TYPES_PUB[🏷️ /api/v1/tasks/types<br/>GET: List types<br/>POST: Register type]
        DOCS[📚 /api-docs/*<br/>OpenAPI documentation]
    end
    
    subgraph "🔒 Protected Endpoints (Bearer Token + RBAC)"
        AUTH_PROT[🚪 /api/v1/auth/logout<br/>🚪 /api/v1/auth/logout-all<br/>🚪 /api/v1/auth/me<br/>🔄 /api/v1/auth/refresh]
        USER_SELF[👤 /api/v1/users/me/*<br/>PUT: Update profile<br/>PUT: Change password<br/>DELETE: Delete account<br/>All users: Own profile only]
        USERS_READ["👥 /api/v1/users<br/>GET: List users<br/>GET: /users/{id}<br/>Moderator+: All users<br/>User: Own profile only"]
        TASKS[⚙️ /api/v1/tasks<br/>POST: Create, GET: List<br/>📊 /api/v1/tasks/stats<br/>💀 /api/v1/tasks/dead-letter<br/>Moderator/Admin: All tasks<br/>User: Own tasks only]
        TASK_OPS["🔧 /api/v1/tasks/{id}<br/>GET, DELETE<br/>🔄 /api/v1/tasks/{id}/retry<br/>🛑 /api/v1/tasks/{id}/cancel<br/>Role-based access applies"]
        MONITORING[📊 /api/v1/monitoring/events<br/>📈 /api/v1/monitoring/metrics<br/>🔧 /api/v1/monitoring/incidents<br/>📉 /api/v1/monitoring/metrics/prometheus<br/>**Tag filtering**: ?tags=key:value,key2:value2<br/>All users: Create/view own data]
    end
    
    subgraph "👨‍💼 Moderator+ Endpoints"
        USER_MOD[👮 /api/v1/users/{id}/status<br/>PUT: Activate/deactivate<br/>POST: /users/{id}/reset-password<br/>Force password reset<br/>Moderator/Admin only]
        MONITORING_MOD[🚨 /api/v1/monitoring/alerts<br/>📊 /api/v1/monitoring/stats<br/>Alert management & system stats<br/>Moderator+ only]
    end
    
    subgraph "👑 Admin Only Endpoints"
        ADMIN[🔧 /api/v1/admin/health<br/>Detailed system status]
        USER_ADMIN[👑 /api/v1/users<br/>POST: Create users<br/>PUT: /users/{id}/role<br/>PUT: /users/{id}/profile<br/>DELETE: /users/{id}<br/>GET: /admin/users/stats<br/>Admin only]
    end
    
    subgraph "🔑 Authentication Flow"
        BEARER["📝 Authorization: Bearer {token}"]
        MIDDLEWARE[🛡️ Auth Middleware<br/>Session validation]
    end
    
    AUTH_PUB --> BEARER
    BEARER --> MIDDLEWARE
    MIDDLEWARE --> AUTH_PROT
    MIDDLEWARE --> USER_SELF
    MIDDLEWARE --> USERS_READ
    MIDDLEWARE --> TASKS
    MIDDLEWARE --> TASK_OPS
    MIDDLEWARE --> MONITORING
    MIDDLEWARE --> USER_MOD
    MIDDLEWARE --> MONITORING_MOD
    MIDDLEWARE --> ADMIN
    MIDDLEWARE --> USER_ADMIN
    
    classDef publicBox fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef protectedBox fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef moderatorBox fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef adminBox fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef authBox fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    
    class HEALTH,AUTH_PUB,TYPES_PUB,DOCS publicBox
    class AUTH_PROT,USER_SELF,USERS_READ,TASKS,TASK_OPS,MONITORING protectedBox
    class USER_MOD,MONITORING_MOD moderatorBox
    class ADMIN,USER_ADMIN adminBox
    class BEARER,MIDDLEWARE authBox
```

## 🛡️ Role-Based Access Control (RBAC)

The API implements a three-tier RBAC system that affects endpoint access:

| Role | Level | Task Access | User Access | Admin Access |
|------|-------|-------------|-------------|--------------|
| **User** | 1 | Own tasks only | Own profile only | ❌ |
| **Moderator** | 2 | All user tasks | All user profiles | ❌ |
| **Admin** | 3 | All user tasks | All user profiles | ✅ Full access |

### RBAC Examples

```bash
# User role: Can only see their own tasks
GET /api/v1/tasks → Returns only tasks created by the authenticated user

# Moderator role: Can see all tasks
GET /api/v1/tasks → Returns all tasks from all users

# Admin role: Full system access
GET /api/v1/admin/health → Detailed system status (admin-only endpoint)
```

For detailed authentication and authorization information, see the [Authentication & Authorization Guide](./guides/02-authentication-and-authorization.md).

### 📖 Using the Interactive Docs
1. Start your server: `./scripts/server.sh 3000`
2. Visit: `http://localhost:3000/api-docs`
3. Click "🔧 Swagger UI (External)" for full interactive testing
4. **Bearer Token Authentication**: The OpenAPI spec includes proper Bearer token security definitions
   - Protected endpoints show 🔒 lock icons in Swagger UI
   - Use "Authorize" button to set your Bearer token for testing
   - All protected endpoints automatically include `Authorization: Bearer {token}` header
5. **Environment Variable Support**: API clients can use custom environment variables
   - Import the OpenAPI spec and create your own variables (e.g., `{{token}}`, `{{sessionToken}}`, etc.)
   - The Bearer security scheme automatically applies your chosen variable to protected endpoints
6. Or download the OpenAPI JSON for use with your preferred API client

---

## Base URL

**Development**: `http://localhost:3000/api/v1`  
**Production**: Configure via `STARTER__SERVER__HOST` and `STARTER__SERVER__PORT` + `/api/v1`

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

**Getting a Bearer Token**:
1. Register or login via `/api/v1/auth/login`
2. Extract the `session_token` from the response
3. Use this token in the `Authorization` header for protected endpoints

**Using with API Testing Tools**:
- **Postman/Insomnia**: Import the OpenAPI spec from `/api-docs/openapi.json`
- **Environment Variables**: Create your own variables (e.g., `{{token}}` or `{{sessionToken}}`) in your testing environment
- **Auto-Authorization**: The OpenAPI spec's Bearer security scheme automatically applies to protected endpoints
- **Manual Setup**: Use "Authorization" tab → "Bearer Token" → enter your session token or variable

**Using with React Frontend**:
- **Auto-Generated Types**: Run `cd web && pnpm run generate-api` to generate TypeScript types
- **Centralized Hooks**: Use `useApiQueries.ts` hooks to prevent cache collisions
- **Type Safety**: All API calls are fully type-safe with auto-completion

## Health Endpoints

### GET /api/v1/health

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

### GET /api/v1/health/detailed

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

### GET /api/v1/health/live

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

### GET /api/v1/health/ready

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

### GET /api/v1/health/startup

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

### POST /api/v1/auth/register

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

### POST /api/v1/auth/login

Authenticate user and create session.

**Authentication**: None required

**Request Body**:
```json
{
  "username": "user@example.com",
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

### POST /api/v1/auth/logout

Invalidate current user session.

**Authentication**: Required (Bearer token)

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

### POST /api/v1/auth/logout-all

Invalidate all sessions for the current user (all devices).

**Authentication**: Required (Bearer token)

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

### GET /api/v1/auth/me

Get current user profile.

**Authentication**: Required (Bearer token)

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

### POST /api/v1/auth/refresh

Refresh session token by extending its expiration time.

**Authentication**: Required (Bearer token)

**Request Body**: None

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "expires_at": "2024-01-03T12:00:00Z",
    "refreshed_at": "2024-01-02T12:00:00Z"
  }
}
```

**Response Fields**:
- `expires_at` - New token expiration time (extended by configured hours)
- `refreshed_at` - Timestamp when refresh occurred

**Rate Limiting**: Can only refresh once every 5 minutes (configurable via `STARTER__AUTH__REFRESH_MIN_INTERVAL_MINUTES`)

**Error Responses**:
- `401` - Invalid or expired token
- `409` - Cannot refresh yet (rate limited)

**Rate Limited Response** (409 Conflict):
```json
{
  "error": {
    "code": "CONFLICT",
    "message": "Cannot refresh token yet. Please wait before requesting another refresh."
  }
}
```

## User Management Endpoints

### GET /api/v1/users

List all users in the system (Moderator/Admin only).

**Authentication**: Required (Bearer token) - Moderator or Admin role

**Query Parameters**:
- `limit` (optional): Maximum number of users to return (default: 50, max: 100)
- `offset` (optional): Number of users to skip for pagination (default: 0)

**Response** (200 OK):
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-here",
      "username": "user1",
      "email": "user1@example.com",
      "role": "user",
      "is_active": true,
      "email_verified": true,
      "created_at": "2024-01-01T00:00:00Z",
      "last_login_at": "2024-01-01T10:00:00Z"
    },
    {
      "id": "uuid-here-2",
      "username": "user2",
      "email": "user2@example.com",
      "role": "moderator",
      "is_active": true,
      "email_verified": false,
      "created_at": "2024-01-02T00:00:00Z",
      "last_login_at": null
    }
  ]
}
```

**Error Responses**:
- `401` - Invalid or expired token
- `403` - Insufficient permissions (User role cannot access)

### GET /api/v1/users/{user_id}

Get another user's profile (public information only).

**Authentication**: Required (Bearer token)

**RBAC Access Control**:
- **User**: Can only access own profile
- **Moderator**: Can access any user profile
- **Admin**: Can access any user profile

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
- `404` - User not found (or access denied for Users accessing other profiles)

### POST /api/v1/users

Create a new user account (Admin only).

**Authentication**: Required (Bearer token) - Admin role

**Request Body**:
```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "password": "SecurePassword123!",
  "role": "user"
}
```

**Available Roles**: `user`, `moderator`, `admin`

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "newuser",
    "email": "newuser@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": null
  }
}
```

**Error Responses**:
- `400` - Validation error (invalid username, email, or password)
- `401` - Invalid or expired token
- `403` - Insufficient permissions (Non-admin user)
- `409` - Username or email already exists

### PUT /api/v1/users/me/profile

Update own user profile.

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "username": "updated_username",
  "email": "updated@example.com"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "updated_username",
    "email": "updated@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T10:00:00Z"
  }
}
```

**Error Responses**:
- `400` - Validation error
- `401` - Invalid or expired token
- `409` - Username or email already exists

### PUT /api/v1/users/me/password

Change own password.

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "current_password": "CurrentPassword123!",
  "new_password": "NewPassword123!"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Password updated successfully",
  "message": "Password has been changed. All existing sessions remain active."
}
```

**Error Responses**:
- `400` - Validation error (password too short/long)
- `401` - Invalid or expired token, or incorrect current password
- `422` - New password same as current password

### DELETE /api/v1/users/me

Delete own user account (soft delete).

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "password": "CurrentPassword123!",
  "confirmation": "DELETE"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Account deleted successfully",
  "message": "Your account has been deactivated. All data will be retained for 30 days."
}
```

**Error Responses**:
- `400` - Missing confirmation or incorrect password
- `401` - Invalid or expired token

### PUT /api/v1/users/{user_id}/profile

Update any user's profile (Admin only).

**Authentication**: Required (Bearer token) - Admin role

**Parameters**:
- `user_id` (path): UUID of the user to update

**Request Body**:
```json
{
  "username": "updated_username",
  "email": "updated@example.com",
  "email_verified": true
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "updated_username",
    "email": "updated@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": true,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T10:00:00Z"
  }
}
```

**Error Responses**:
- `400` - Validation error
- `401` - Invalid or expired token
- `403` - Insufficient permissions (Non-admin user)
- `404` - User not found
- `409` - Username or email already exists

### PUT /api/v1/users/{user_id}/status

Activate or deactivate a user account (Moderator/Admin).

**Authentication**: Required (Bearer token) - Moderator or Admin role

**Parameters**:
- `user_id` (path): UUID of the user

**Request Body**:
```json
{
  "is_active": false,
  "reason": "Account suspended for policy violation"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "username",
    "email": "user@example.com",
    "role": "user",
    "is_active": false,
    "email_verified": true,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T10:00:00Z"
  },
  "message": "User account status updated"
}
```

**Error Responses**:
- `400` - Invalid request (missing is_active field)
- `401` - Invalid or expired token
- `403` - Insufficient permissions (User role cannot access)
- `404` - User not found

### PUT /api/v1/users/{user_id}/role

Change a user's role (Admin only).

**Authentication**: Required (Bearer token) - Admin role

**Parameters**:
- `user_id` (path): UUID of the user

**Request Body**:
```json
{
  "role": "moderator",
  "reason": "Promoted to moderator for community management"
}
```

**Available Roles**: `user`, `moderator`, `admin`

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "username": "username",
    "email": "user@example.com",
    "role": "moderator",
    "is_active": true,
    "email_verified": true,
    "created_at": "2024-01-01T00:00:00Z",
    "last_login_at": "2024-01-01T10:00:00Z"
  },
  "message": "User role updated successfully"
}
```

**Error Responses**:
- `400` - Invalid role value
- `401` - Invalid or expired token
- `403` - Insufficient permissions (Non-admin user)
- `404` - User not found

### POST /api/v1/users/{user_id}/reset-password

Force password reset for a user (Moderator/Admin).

**Authentication**: Required (Bearer token) - Moderator or Admin role

**Parameters**:
- `user_id` (path): UUID of the user

**Request Body**:
```json
{
  "new_password": "TemporaryPassword123!",
  "require_change": true,
  "reason": "Password reset requested by user"
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": "Password reset successfully",
  "message": "User's password has been updated. All existing sessions have been invalidated."
}
```

**Error Responses**:
- `400` - Invalid password (too short/long)
- `401` - Invalid or expired token
- `403` - Insufficient permissions (User role cannot access)
- `404` - User not found

### DELETE /api/v1/users/{user_id}

Delete a user account (Admin only).

**Authentication**: Required (Bearer token) - Admin role

**Parameters**:
- `user_id` (path): UUID of the user

**Request Body**:
```json
{
  "reason": "Account deletion requested by user",
  "hard_delete": false
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": "User account deleted successfully",
  "message": "User account has been deactivated. Data retained for 30 days for recovery."
}
```

**Error Responses**:
- `400` - Cannot delete own account via this endpoint
- `401` - Invalid or expired token
- `403` - Insufficient permissions (Non-admin user)
- `404` - User not found

### GET /api/v1/admin/users/stats

Get user statistics (Admin only).

**Authentication**: Required (Bearer token) - Admin role

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "total_users": 1250,
    "active_users": 1180,
    "inactive_users": 70,
    "email_verified": 950,
    "email_unverified": 300,
    "by_role": {
      "user": 1200,
      "moderator": 45,
      "admin": 5
    },
    "recent_registrations": {
      "last_24h": 12,
      "last_7d": 85,
      "last_30d": 320
    },
    "last_updated": "2024-01-01T12:00:00Z"
  }
}
```

**Error Responses**:
- `401` - Invalid or expired token
- `403` - Insufficient permissions (Non-admin user)

## Task Management Endpoints

Task endpoints demonstrate background job processing patterns.

### Task Type Management

Before creating tasks, you must register task types with the API server. This is typically done automatically by workers, but you can also manage task types manually.

### POST /api/v1/tasks/types

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

### GET /api/v1/tasks/types

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

### POST /api/v1/tasks

Create a background task for async processing.

**Authentication**: Required (Bearer token)

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

### GET /api/v1/tasks

List your background tasks.

**Authentication**: Required (Bearer token)

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

### GET /api/v1/tasks/{task_id}

Get details about a specific task.

**Authentication**: Required (Bearer token)

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

### GET /api/v1/tasks/stats

Get basic task statistics.

**Authentication**: Required (Bearer token)

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

### GET /api/v1/tasks/dead-letter

Get all failed tasks in the dead letter queue for debugging and manual recovery.

**Authentication**: Required (Bearer token)

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

### POST /api/v1/tasks/{task_id}/cancel

Cancel a pending or retrying task.

**Authentication**: Required (Bearer token)

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

### POST /api/v1/tasks/{task_id}/retry

Retry a failed task by resetting it to pending status.

**Authentication**: Required (Bearer token)

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

### DELETE /api/v1/tasks/{task_id}

Permanently delete a completed, failed, or cancelled task.

**Authentication**: Required (Bearer token)

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

## Monitoring and Observability Endpoints

The system includes comprehensive monitoring capabilities for tracking application health, collecting metrics, managing alerts, and performing incident management.

### Event Management

#### POST /api/v1/monitoring/events

Create a monitoring event (log, metric, trace, or alert).

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "event_type": "log",
  "source": "payment-service",
  "message": "Payment processed successfully",
  "level": "info",
  "tags": {
    "request_id": "req-123",
    "user_id": "user-456",
    "amount": "100.00"
  },
  "payload": {
    "payment_method": "credit_card",
    "gateway": "stripe"
  }
}
```

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "evt-789",
    "event_type": "log",
    "source": "payment-service",
    "message": "Payment processed successfully",
    "level": "info",
    "tags": { "request_id": "req-123", "user_id": "user-456" },
    "payload": { "payment_method": "credit_card" },
    "timestamp": "2024-01-15T10:30:00Z",
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

#### GET /api/v1/monitoring/events

Query events with optional filters.

**Authentication**: Required (Bearer token)

**Query Parameters**:
- `event_type`: Filter by event type (`log`, `metric`, `trace`, `alert`)
- `source`: Filter by service name
- `level`: Filter by level (`error`, `warn`, `info`, `debug`)
- `tags`: **Filter by tags using key:value pairs** - `user_id:123,environment:production`
- `start_time`, `end_time`: ISO 8601 timestamps
- `limit`: Max results (default: 100)
- `offset`: Skip results (default: 0)

**Examples**:
```bash
# Basic filtering
GET /api/v1/monitoring/events?event_type=log&source=payment-service&limit=50

# Tag filtering with single tag
GET /api/v1/monitoring/events?tags=user_id:123

# Tag filtering with multiple tags (AND logic)
GET /api/v1/monitoring/events?tags=user_id:123,environment:production

# Combined filtering
GET /api/v1/monitoring/events?event_type=log&level=error&tags=service:payment,severity:high
```

#### GET /api/v1/monitoring/events/{id}

Get a specific event by ID.

**Authentication**: Required (Bearer token)

### Metric Management

#### POST /api/v1/monitoring/metrics

Submit a performance or business metric.

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "name": "payment_processing_duration_ms",
  "metric_type": "histogram",
  "value": 245.5,
  "labels": {
    "payment_method": "credit_card",
    "gateway": "stripe",
    "currency": "USD"
  }
}
```

**Metric Types**: `counter`, `gauge`, `histogram`, `summary`

#### GET /api/v1/monitoring/metrics

Query metrics with filters.

**Authentication**: Required (Bearer token)

**Query Parameters**:
- `name`: Filter by metric name
- `metric_type`: Filter by metric type
- `start_time`, `end_time`: Time range filters
- `limit`, `offset`: Pagination

#### GET /api/v1/monitoring/metrics/prometheus

Export metrics in Prometheus exposition format.

**Authentication**: Required (Bearer token)

**Response**: Plain text Prometheus format
```
# HELP monitoring_total_events Total number of events in the system
# TYPE monitoring_total_events counter
monitoring_total_events 15420

# HELP monitoring_active_alerts Number of currently active alerts
# TYPE monitoring_active_alerts gauge
monitoring_active_alerts 3
```

### Alert Management

#### POST /api/v1/monitoring/alerts

Create a new alert rule (Moderator+ required).

**Authentication**: Required (Bearer token) - Moderator or Admin role

**Request Body**:
```json
{
  "name": "High Error Rate",
  "description": "Alert when error rate exceeds threshold",
  "query": "error_rate > 0.05",
  "threshold_value": 0.05
}
```

#### GET /api/v1/monitoring/alerts

List all alert rules.

**Authentication**: Required (Bearer token)

### Incident Management

#### POST /api/v1/monitoring/incidents

Create a new incident for tracking outages or issues.

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "title": "Payment Gateway Degradation",
  "description": "Stripe API response time increased to 5+ seconds",
  "severity": "high",
  "assigned_to": "engineer-uuid-here"
}
```

**Severity Levels**: `low`, `medium`, `high`, `critical`

**Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "inc-123",
    "title": "Payment Gateway Degradation",
    "description": "Stripe API response time increased to 5+ seconds",
    "severity": "high",
    "status": "open",
    "started_at": "2024-01-15T10:00:00Z",
    "created_by": "user-uuid",
    "assigned_to": "engineer-uuid-here",
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### GET /api/v1/monitoring/incidents

List incidents with pagination.

**Authentication**: Required (Bearer token)

**Query Parameters**:
- `limit`: Max results (default: 100)
- `offset`: Skip results (default: 0)

#### GET /api/v1/monitoring/incidents/{id}

Get details about a specific incident.

**Authentication**: Required (Bearer token)

#### PUT /api/v1/monitoring/incidents/{id}

Update an incident (Moderator+ or incident creator).

**Authentication**: Required (Bearer token)

**Request Body**:
```json
{
  "status": "resolved",
  "root_cause": "Third-party payment provider experienced regional outage"
}
```

**Status Values**: `open`, `investigating`, `resolved`, `closed`

#### GET /api/v1/monitoring/incidents/{id}/timeline

Get incident timeline with correlated events.

**Authentication**: Required (Bearer token)

**Query Parameters**:
- `limit`: Max timeline entries (default: 100)
- `offset`: Skip entries (default: 0)

**Response**:
```json
{
  "success": true,
  "data": {
    "incident_id": "inc-123",
    "start_time": "2024-01-15T09:00:00Z",
    "end_time": "2024-01-15T10:30:00Z",
    "entries": [
      {
        "id": "evt-456",
        "timestamp": "2024-01-15T09:15:00Z",
        "event_type": "log",
        "source": "payment-service",
        "message": "Gateway response time increased",
        "level": "warn",
        "tags": { "gateway": "stripe", "duration_ms": "3000" }
      }
    ],
    "total_count": 45
  }
}
```

### System Statistics

#### GET /api/v1/monitoring/stats

Get monitoring system statistics (Moderator+ required).

**Authentication**: Required (Bearer token) - Moderator or Admin role

**Response**:
```json
{
  "success": true,
  "data": {
    "total_events": 15420,
    "total_metrics": 8934,
    "active_alerts": 3,
    "open_incidents": 1,
    "events_last_hour": 245,
    "metrics_last_hour": 189
  }
}
```

### RBAC Access Control

| Role | Events | Metrics | Alerts | Incidents | Stats |
|------|--------|---------|--------|-----------|-------|
| **User** | ✅ Create/View | ✅ Create/View | ❌ View only | ✅ Create/View own | ❌ |
| **Moderator** | ✅ Full access | ✅ Full access | ✅ Create/Manage | ✅ Full access | ✅ View |
| **Admin** | ✅ Full access | ✅ Full access | ✅ Full access | ✅ Full access | ✅ View |

For detailed implementation examples and patterns, see the [Monitoring and Observability Guide](guides/15-monitoring-and-observability.md).

## Admin Endpoints

### GET /api/v1/admin/health

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

#### Backend (API) Testing

1. **Register a new user**:
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "SecurePass123!"}'
```

2. **Login to get session token**:
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "test@example.com", "password": "SecurePass123!"}'
```

3. **Access protected endpoint**:
```bash
curl -X GET http://localhost:3000/api/v1/auth/me \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN_HERE"
```

4. **Logout**:
```bash
curl -X POST http://localhost:3000/api/v1/auth/logout \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN_HERE"
```

#### Frontend (React) Integration

```typescript
// web/src/lib/auth/context.tsx - Authentication flow
import { useCurrentUser } from '@/hooks/useApiQueries';
import { apiClient } from '@/lib/api/client';

function LoginForm() {
  const { login } = useAuth();
  const [credentials, setCredentials] = useState({
    username: '',
    password: ''
  });

  const loginMutation = useMutation({
    mutationFn: login,
    onSuccess: () => {
      navigate({ to: '/admin' });
    },
    onError: (error: ApiError) => {
      toast.error(error.message);
    }
  });

  // Type-safe form submission
  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    loginMutation.mutate(credentials);
  };

  // Auto-generated types ensure API compatibility
  return (
    <form onSubmit={handleSubmit}>
      {/* Form inputs */}
    </form>
  );
}

// Centralized user data with caching
function UserProfile() {
  const { data: user, isLoading } = useCurrentUser(30000); // 30s refresh
  
  if (isLoading) return <LoadingSpinner />;
  
  return <div>Welcome, {user?.username}!</div>;
}
```

## Testing

Use the provided test scripts to validate all endpoints:

### Comprehensive Testing
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all 123 tests (~17 seconds)
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

# Optional: Multiple concurrent workers
# ./scripts/worker.sh --id 1
# ./scripts/worker.sh --id 2

# 2. Register and login
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'

TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}' \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])")

# 3. Create and monitor task
curl -X POST http://localhost:3000/api/v1/tasks \
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
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/tasks/stats
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

# Run all API tests (123 tests)
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
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# Test login and extract token
TOKEN=$(curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "SecurePass123!"}' \
  | jq -r '.data.session_token')
```

#### Protected Endpoint Testing
```bash
# Test protected endpoint with authentication
curl -X GET http://localhost:3000/api/v1/auth/me \
  -H "Authorization: Bearer $TOKEN"

# Test task creation (requires auth)
curl -X POST http://localhost:3000/api/v1/tasks \
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
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "", "email": "invalid", "password": "weak"}'

# Test authentication errors
curl -X GET http://localhost:3000/api/v1/auth/me
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