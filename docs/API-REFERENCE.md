# API Reference

*Essential endpoints and examples for the REST API. For complete interactive documentation, visit `/api-docs` when running the server.*

## 🌐 Base Information

- **Base URL**: `http://localhost:3000/api/v1`
- **Authentication**: Bearer token in Authorization header
- **Content Type**: `application/json`
- **Total Endpoints**: 48 (37 unique paths)

## 🔐 Authentication Endpoints

### Register User
```http
POST /auth/register
Content-Type: application/json

{
  "username": "newuser",
  "email": "user@example.com", 
  "password": "SecurePass123!"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "username": "newuser",
      "email": "user@example.com",
      "role": "user",
      "created_at": "2024-01-15T10:30:00Z"
    }
  }
}
```

### Login
```http
POST /auth/login
Content-Type: application/json

{
  "username": "newuser",
  "password": "SecurePass123!"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "session_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "expires_at": "2024-01-16T10:30:00Z",
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "username": "newuser",
      "role": "user"
    }
  }
}
```

### Current User
```http
GET /auth/me
Authorization: Bearer <token>
```

### Logout
```http
POST /auth/logout
Authorization: Bearer <token>
```

### Logout from All Devices
```http
POST /auth/logout-all
Authorization: Bearer <token>
```

**Description**: Logout current user from all devices and end all sessions

### Refresh Token
```http
POST /auth/refresh
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "expires_at": "2024-01-16T10:30:00Z",
    "refreshed_at": "2024-01-15T10:30:00Z"
  }
}
```

## 👥 User Management

### Get Own Profile
```http
GET /users/me/profile
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "newuser",
    "email": "user@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": true,
    "created_at": "2024-01-15T10:30:00Z",
    "last_login_at": "2024-01-15T09:30:00Z"
  }
}
```

### Update Own Profile
```http
PUT /users/me/profile
Authorization: Bearer <token>
Content-Type: application/json

{
  "email": "updated@example.com"
}
```

### Change Password
```http
PUT /users/me/password
Authorization: Bearer <token>
Content-Type: application/json

{
  "current_password": "OldPass123!",
  "new_password": "NewSecurePass456!"
}
```

### List Users (Moderator+)
```http
GET /users?limit=20&offset=0
Authorization: Bearer <moderator_token>
```

### Create User (Admin)
```http
POST /users
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "username": "newadmin",
  "email": "admin@example.com",
  "password": "AdminPass123!",
  "role": "moderator"
}
```

### Get User by ID
```http
GET /users/{user_id}
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "456e7890-e89b-12d3-a456-426614174000",
    "username": "otheruser",
    "email": "other@example.com",
    "role": "user",
    "is_active": true,
    "email_verified": true,
    "created_at": "2024-01-10T08:15:00Z"
  }
}
```

### Update User Profile (Admin)
```http
PUT /users/{user_id}/profile
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "username": "updated_username",
  "email": "updated@example.com",
  "email_verified": true
}
```

### Update User Role (Admin)
```http
PUT /users/{user_id}/role
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "role": "moderator",
  "reason": "Promoted for community management"
}
```

### Update User Status (Moderator+)
```http
PUT /users/{user_id}/status
Authorization: Bearer <moderator_token>
Content-Type: application/json

{
  "is_active": false,
  "reason": "Account suspended for policy violation"
}
```

### Reset User Password (Moderator+)
```http
POST /users/{user_id}/reset-password
Authorization: Bearer <moderator_token>
Content-Type: application/json

{
  "new_password": "TempPassword123!",
  "require_change": true,
  "reason": "Password reset requested by user"
}
```

### Delete User Account (Admin)
```http
DELETE /users/{user_id}
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "hard_delete": false,
  "reason": "Account deletion requested"
}
```

### Delete Own Account
```http
DELETE /users/me
Authorization: Bearer <token>
Content-Type: application/json

{
  "password": "CurrentPassword123!",
  "confirmation": "DELETE"
}
```

## ⚙️ Background Tasks

### Create Task
```http
POST /tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "task_type": "email",
  "payload": {
    "to": "recipient@example.com",
    "subject": "Important Message",
    "body": "This is the email content"
  },
  "priority": "normal"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "456e7890-e89b-12d3-a456-426614174000",
    "task_type": "email",
    "status": "pending",
    "priority": "normal",
    "created_at": "2024-01-15T10:30:00Z",
    "created_by": "123e4567-e89b-12d3-a456-426614174000"
  }
}
```

### List Tasks
```http
GET /tasks?status=pending&limit=50
Authorization: Bearer <token>
```

**Query Parameters**:
- `status`: `pending`, `running`, `completed`, `failed`, `cancelled`, `retrying`
- `task_type`: Filter by task type
- `limit`: Number of results (default: 50, max: 100)
- `offset`: Pagination offset

### Get Task Details
```http
GET /tasks/{task_id}
Authorization: Bearer <token>
```

### Retry Failed Task
```http
POST /tasks/{task_id}/retry
Authorization: Bearer <token>
```

### Cancel Task
```http
POST /tasks/{task_id}/cancel
Authorization: Bearer <token>
```

### Delete Completed Task
```http
DELETE /tasks/{task_id}
Authorization: Bearer <token>
```

### Task Statistics
```http
GET /tasks/stats
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "pending": 5,
    "running": 2,
    "completed": 145,
    "failed": 3,
    "total": 155
  }
}
```

### Registered Task Types
```http
GET /tasks/types
Authorization: Bearer <token>
```

### Register Task Type
```http
POST /tasks/types
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "webhook",
  "description": "HTTP webhook caller"
}
```

### Dead Letter Queue
```http
GET /tasks/dead-letter
Authorization: Bearer <token>
```

## 📊 Monitoring & Observability

### Create Event
```http
POST /monitoring/events
Authorization: Bearer <token>
Content-Type: application/json

{
  "event_type": "log",
  "source": "user-service",
  "message": "User login successful",
  "level": "info",
  "tags": {
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "ip_address": "192.168.1.100"
  }
}
```

### Query Events
```http
GET /monitoring/events?tags=user_id:123,level:error&limit=100
Authorization: Bearer <token>
```

**Query Parameters**:
- `tags`: Filter by tags (format: `key:value,key2:value2`)
- `event_type`: `log`, `metric`, `trace`, `alert`
- `source`: Filter by event source
- `level`: Filter by log level
- `limit`: Number of results

### Get Event by ID
```http
GET /monitoring/events/{event_id}
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "789e1234-e89b-12d3-a456-426614174000",
    "event_type": "log",
    "source": "user-service",
    "message": "User login successful",
    "level": "info",
    "tags": {
      "user_id": "123e4567-e89b-12d3-a456-426614174000"
    },
    "recorded_at": "2024-01-15T10:30:00Z",
    "created_at": "2024-01-15T10:30:05Z"
  }
}
```

### Submit Metric
```http
POST /monitoring/metrics
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "response_time_ms",
  "metric_type": "histogram",
  "value": 245.5,
  "labels": {
    "endpoint": "/api/v1/users",
    "method": "GET"
  }
}
```

### Query Metrics
```http
GET /monitoring/metrics?name=response_time_ms&metric_type=histogram&limit=100
Authorization: Bearer <token>
```

**Query Parameters**:
- `name`: Filter by metric name
- `metric_type`: `counter`, `gauge`, `histogram`, `summary`
- `start_time`: ISO 8601 datetime for time range start
- `end_time`: ISO 8601 datetime for time range end
- `limit`: Number of results

### List Alerts
```http
GET /monitoring/alerts
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "id": "alert-123e4567-e89b-12d3-a456-426614174000",
      "name": "High Error Rate",
      "query": "error_rate > 0.05",
      "status": "active",
      "threshold_value": 0.05,
      "created_by": "admin-456e7890-e89b-12d3-a456-426614174000",
      "created_at": "2024-01-15T09:00:00Z",
      "triggered_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

### Create Alert (Moderator+)
```http
POST /monitoring/alerts
Authorization: Bearer <moderator_token>
Content-Type: application/json

{
  "name": "Database Connection Alert",
  "description": "Alert when database connections exceed threshold",
  "query": "db_connections > 50",
  "threshold_value": 50
}
```

### List Incidents
```http
GET /monitoring/incidents?limit=50&offset=0
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "id": "incident-123e4567-e89b-12d3-a456-426614174000",
      "title": "Database Connection Issues",
      "description": "Users reporting login failures",
      "severity": "high",
      "status": "investigating",
      "started_at": "2024-01-15T09:30:00Z",
      "created_at": "2024-01-15T09:32:00Z",
      "assigned_to": "admin-456e7890-e89b-12d3-a456-426614174000"
    }
  ]
}
```

### Get Incident by ID
```http
GET /monitoring/incidents/{incident_id}
Authorization: Bearer <token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "incident-123e4567-e89b-12d3-a456-426614174000",
    "title": "Database Connection Issues",
    "description": "Users reporting login failures",
    "severity": "high",
    "status": "investigating",
    "started_at": "2024-01-15T09:30:00Z",
    "created_at": "2024-01-15T09:32:00Z",
    "updated_at": "2024-01-15T10:15:00Z",
    "assigned_to": "admin-456e7890-e89b-12d3-a456-426614174000",
    "root_cause": null,
    "resolved_at": null
  }
}
```

### Create Incident
```http
POST /monitoring/incidents
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Database Connection Issues",
  "description": "Users reporting login failures",
  "severity": "high",
  "assigned_to": "admin-456e7890-e89b-12d3-a456-426614174000"
}
```

### Update Incident
```http
PUT /monitoring/incidents/{incident_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "investigating",
  "assigned_to": "456e7890-e89b-12d3-a456-426614174000"
}
```

### Incident Timeline
```http
GET /monitoring/incidents/{incident_id}/timeline?limit=50
Authorization: Bearer <token>
```

### System Statistics (Moderator+)
```http
GET /monitoring/stats
Authorization: Bearer <moderator_token>
```

### Prometheus Metrics (Public)
```http
GET /monitoring/metrics/prometheus
```

## ❤️ Health Checks

### Basic Health
```http
GET /health
```

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-15T10:30:00Z",
    "version": "0.1.0"
  }
}
```

### Detailed Health
```http
GET /health/detailed
```

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "checks": {
      "database": "healthy",
      "task_processor": "healthy",
      "memory": "healthy"
    },
    "uptime_seconds": 3600,
    "version": "0.1.0"
  }
}
```

### Kubernetes Probes

**Liveness Probe**:
```http
GET /health/live
```

**Readiness Probe**:
```http
GET /health/ready
```

**Startup Probe**:
```http
GET /health/startup
```

## 👮 Admin Endpoints

### User Statistics (Admin)
```http
GET /admin/users/stats
Authorization: Bearer <admin_token>
```

**Response**:
```json
{
  "success": true,
  "data": {
    "total_users": 1250,
    "active_sessions": 45,
    "registrations_today": 12,
    "registrations_this_week": 89,
    "user_roles": {
      "user": 1200,
      "moderator": 45,
      "admin": 5
    }
  }
}
```

## 🔒 Authentication & Authorization

### Session Management
All endpoints (except health checks and public endpoints) require authentication via Bearer token:

```http
Authorization: Bearer <session_token>
```

### RBAC Permissions

| Role | Permissions |
|------|-------------|
| **User** | Own profile, own tasks, create incidents, create events/metrics |
| **Moderator** | User permissions + view all tasks/incidents, manage alerts, system statistics |
| **Admin** | Moderator permissions + user management, system configuration |

### Error Responses

**401 Unauthorized**:
```json
{
  "success": false,
  "error": "Authentication required"
}
```

**403 Forbidden**:
```json
{
  "success": false,
  "error": "Insufficient permissions"
}
```

**400 Bad Request**:
```json
{
  "success": false,
  "error": "Validation failed: email format is invalid"
}
```

**404 Not Found**:
```json
{
  "success": false,
  "error": "Task with id 123e4567-e89b-12d3-a456-426614174000 not found"
}
```

**500 Internal Server Error**:
```json
{
  "success": false,
  "error": "Internal server error"
}
```

## 📝 Request/Response Patterns

### Standard Response Format
All API responses follow this structure:

```typescript
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  request_id?: string;
}
```

### Pagination
List endpoints support pagination:

```http
GET /users?limit=20&offset=40
```

### Filtering
Many endpoints support filtering via query parameters:

```http
GET /tasks?status=pending&task_type=email&created_by=user123
```

### Rate Limiting
- **Default**: 100 requests per minute per IP
- **Authentication endpoints**: 10 requests per minute per IP
- **Headers included**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

## 🧪 Testing the API

### With cURL
```bash
# Get session token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' | \
  jq -r '.data.session_token')

# Use token for authenticated requests
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/users/me/profile
```

### With the Test Script
```bash
# Test all endpoints
./scripts/test-with-curl.sh

# Test specific host/port
./scripts/test-with-curl.sh localhost 8080
```

---

*For complete interactive documentation with request/response examples and the ability to test endpoints directly, visit `/api-docs` when running the server.*