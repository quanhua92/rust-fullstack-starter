# User Management System

*This guide covers the comprehensive user lifecycle management system with 12 endpoints for profile management, administration, and analytics.*

## ⚡ TL;DR - Working User Management (5 minutes)

**Want working user management right now?** Here's the copy-paste version:

### Update Your Own Profile
```bash
# Get your session token first
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secure123"}' \
  | jq -r '.data.session_token')

# Update your profile
curl -X PUT http://localhost:3000/api/v1/users/me/profile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"email": "alice.new@example.com"}'

# Change your password
curl -X PUT http://localhost:3000/api/v1/users/me/password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"current_password": "secure123", "new_password": "NewPassword456!"}'
```

### Admin Operations (Requires Admin Role)
```bash
# Create new user
curl -X POST http://localhost:3000/api/v1/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username": "bob", "email": "bob@example.com", "password": "TempPass123!", "role": "user"}'

# Get user statistics
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:3000/api/v1/admin/users/stats
```

**That's it!** You now have comprehensive user management. Want to understand the RBAC patterns and implementation? Keep reading ↓

---

## System Overview

### Comprehensive User Lifecycle Management

The user management system handles the complete user lifecycle from creation to deletion, all with proper RBAC (Role-Based Access Control) enforcement. It extends the basic authentication system with **12 new endpoints** that demonstrate sophisticated authorization patterns.

### Key Features

✅ **Profile Self-Management** - Users control their own data  
✅ **Hierarchical Administration** - Moderators and admins have different capabilities  
✅ **Comprehensive Analytics** - Detailed user statistics for operational insights  
✅ **Security-First Design** - Password verification, audit trails, and data privacy  
✅ **RBAC Enforcement** - Demonstrates three authorization patterns  
✅ **Soft Delete Protection** - Data preservation with recovery options  

## User Management Endpoints Overview

| Endpoint | Method | Access Level | Description |
|----------|--------|-------------|-------------|
| **Profile Management** | | | |
| `/api/v1/users/me/profile` | PUT | User | Update own profile (username, email) |
| `/api/v1/users/me/password` | PUT | User | Change own password with verification |
| `/api/v1/users/me` | DELETE | User | Delete own account (soft delete) |
| **User Administration** | | | |
| `/api/v1/users` | GET | Moderator+ | List all users (paginated) |
| `/api/v1/users` | POST | Admin | Create new user account |
| `/api/v1/users/{id}` | GET | Owner/Moderator+ | Get user profile by ID |
| `/api/v1/users/{id}/profile` | PUT | Admin | Update any user's profile |
| `/api/v1/users/{id}/status` | PUT | Moderator+ | Activate/deactivate user accounts |
| `/api/v1/users/{id}/role` | PUT | Admin | Change user roles |
| `/api/v1/users/{id}/reset-password` | POST | Moderator+ | Force password reset |
| `/api/v1/users/{id}` | DELETE | Admin | Delete user account (admin) |
| **Analytics** | | | |
| `/api/v1/admin/users/stats` | GET | Admin | Comprehensive user statistics |

## Profile Self-Management

### Update Own Profile

Users can update their username and email address:

```bash
curl -X PUT http://localhost:3000/api/v1/users/me/profile \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice_updated",
    "email": "alice.new@example.com"
  }'
```

**Security Features:**
- Automatic email verification reset when email changes
- Username/email uniqueness validation across all users
- Conflict detection with descriptive error messages

### Change Own Password

Secure password changes with current password verification:

```bash
curl -X PUT http://localhost:3000/api/v1/users/me/password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "secure123",
    "new_password": "NewSecurePassword456!"
  }'
```

**Security Features:**
- Current password verification prevents unauthorized changes
- Strong password requirements enforced
- All existing sessions remain active (user choice)

### Delete Own Account

Self-service account deletion with confirmation:

```bash
curl -X DELETE http://localhost:3000/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "password": "secure123",
    "confirmation": "DELETE"
  }'
```

**Security Features:**
- Password verification ensures account owner consent
- Confirmation string ("DELETE") prevents accidental deletion
- Soft delete preserves data for 30-day recovery
- Automatic session invalidation across all devices

## Administrative Operations

### Create New User (Admin Only)

Admins can create users with specific roles:

```bash
curl -X POST http://localhost:3000/api/v1/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "bob",
    "email": "bob@example.com",
    "password": "TempPassword123!",
    "role": "user"
  }'
```

**Available Roles:** `user`, `moderator`, `admin`

### Update User Profile (Admin Only)

Admins can update any user's profile, including email verification status:

```bash
curl -X PUT http://localhost:3000/api/v1/users/USER_ID/profile \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "updated_username",
    "email": "updated@example.com",
    "email_verified": true
  }'
```

### Manage User Status (Moderator+)

Moderators and admins can activate/deactivate accounts:

```bash
curl -X PUT http://localhost:3000/api/v1/users/USER_ID/status \
  -H "Authorization: Bearer $MODERATOR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "is_active": false,
    "reason": "Account suspended for community guidelines violation"
  }'
```

**Features:**
- Automatic session invalidation when deactivating
- Audit trail with reason tracking
- Immediate effect across all user sessions

### Update User Role (Admin Only)

Admins can promote/demote users:

```bash
curl -X PUT http://localhost:3000/api/v1/users/USER_ID/role \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "moderator",
    "reason": "Promoted for community management responsibilities"
  }'
```

### Reset User Password (Moderator+)

Force password reset for users:

```bash
curl -X POST http://localhost:3000/api/v1/users/USER_ID/reset-password \
  -H "Authorization: Bearer $MODERATOR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "new_password": "TempPassword123!",
    "require_change": true,
    "reason": "Password reset requested by user via support"
  }'
```

**Security Features:**
- Automatic session invalidation forces re-login
- Strong password generation recommended
- Audit trail with reason tracking

### Delete User Account (Admin Only)

Admin-initiated account deletion:

```bash
curl -X DELETE http://localhost:3000/api/v1/users/USER_ID \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Account deletion requested by user via support",
    "hard_delete": false
  }'
```

**Options:**
- `hard_delete: false` - Soft delete (default, preserves data)
- `hard_delete: true` - Permanent deletion (irreversible)

**Safety Features:**
- Admins cannot delete their own accounts via this endpoint
- Comprehensive audit trail with deletion reasons
- Session invalidation and cleanup

## User Analytics

### Comprehensive Statistics (Admin Only)

Get detailed user metrics:

```bash
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:3000/api/v1/admin/users/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_users": 150,
    "active_users": 142,
    "inactive_users": 8,
    "email_verified": 135,
    "email_unverified": 7,
    "by_role": {
      "user": 140,
      "moderator": 8,
      "admin": 2
    },
    "recent_registrations": {
      "last_24h": 5,
      "last_7d": 23,
      "last_30d": 67
    },
    "last_updated": "2024-01-02T12:00:00Z"
  }
}
```

**Metrics Included:**
- Total and active user counts
- Email verification status breakdown
- Role distribution analysis
- Registration trend analysis (24h, 7d, 30d)
- Real-time timestamp for data freshness

## RBAC Patterns in Action

The user management system demonstrates three sophisticated RBAC patterns:

### 1. Resource Ownership Pattern

**Concept:** Users can always access their own resources without additional authorization checks.

```rust
// Users can only update their own profiles
pub async fn update_own_profile(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // No additional auth needed - user is updating their own resource
    let user = user_services::update_user_profile(&mut conn, auth_user.id, request).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

**Security Principle:** "You own your data" - the most intuitive and secure default.

### 2. Hierarchical Access Pattern

**Concept:** Higher roles inherit all lower-role capabilities plus additional privileges.

```rust
// Moderators can manage user status, admins can do everything
pub async fn update_user_status(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Require moderator or higher role
    rbac_services::require_moderator_or_higher(&auth_user)?;
    
    let user = user_services::update_user_status(&mut conn, id, request).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

**Security Principle:** "Higher authority includes lower authority" - prevents privilege escalation bugs.

### 3. Cross-User Access Control

**Concept:** Different behavior based on relationship between requester and target resource.

```rust
pub async fn get_user_by_id(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Users can view their own profile, moderators+ can view any profile
    let target_user = user_services::find_user_by_id(&mut conn, id).await?;
    
    // Check authorization with target user's role
    rbac_services::can_access_user_profile(&auth_user, id, target_user.role)?;
    
    Ok(Json(ApiResponse::success(target_user.to_profile())))
}
```

**Security Principle:** "Context-aware authorization" - same endpoint, different behavior based on roles and ownership.

## Security Features

### Password Protection
- **Current password verification** for sensitive operations
- **Strong password requirements** enforced across all endpoints
- **Automatic session invalidation** after password changes

### Account Protection
- **Confirmation requirements** for destructive operations ("DELETE" string)
- **Soft delete by default** preserves data for recovery
- **Admin safety checks** prevent self-deletion via admin endpoints

### Audit Trail
- **Reason fields** for all administrative operations
- **Comprehensive logging** of role changes and account modifications
- **Database timestamps** track all profile updates automatically

### Data Privacy
- **Password hashes never returned** in API responses
- **404 responses instead of 403** to prevent user enumeration
- **Email verification status** tracked and managed properly

## Testing

### Integration Tests

The user management system includes 17 comprehensive integration tests:

```bash
# Run user management tests
cargo nextest run users::

# Run specific test categories
cargo nextest run test_update_own_profile
cargo nextest run test_admin_operations
cargo nextest run test_rbac_enforcement
```

**Test Coverage:**
- ✅ Profile management (update, password change, deletion)
- ✅ Account lifecycle (creation, deactivation, role changes)
- ✅ RBAC enforcement (role-based access control)
- ✅ Admin operations (user creation, management, analytics)
- ✅ Cross-user access control (ownership patterns)
- ✅ Security features (password verification, confirmations)
- ✅ Error handling (invalid inputs, unauthorized access)
- ✅ Data validation (email formats, username constraints)

### API Testing with curl

Comprehensive endpoint testing:

```bash
# Test all user management endpoints (included in 40+ endpoint tests)
./scripts/test-with-curl.sh

# Test with admin credentials (set environment variable)
STARTER__INITIAL_ADMIN_PASSWORD=admin123 ./scripts/test-with-curl.sh
```

**Test Features:**
- Automatic admin account detection and testing
- RBAC permission validation for all endpoints
- Error response format verification
- Password security testing (wrong passwords, confirmations)

## Implementation Architecture

### Module Organization

```rust
src/users/
├── api.rs          -- HTTP endpoints (12 user management handlers)
├── models.rs       -- Request/response types with validation
├── services.rs     -- Business logic (password hashing, RBAC checks)
└── mod.rs          -- Module exports and organization
```

### Service Layer Pattern

**Business Logic Separation:**
```rust
// Services handle business logic with database access
pub async fn update_user_profile(
    conn: &mut DbConn,
    user_id: Uuid,
    request: UpdateProfileRequest,
) -> Result<UserProfile> {
    // Input validation
    request.validate()?;
    
    // Uniqueness checks
    check_username_email_availability(conn, &request, user_id).await?;
    
    // Database update with automatic timestamp
    let user = sqlx::query_as!(/* update query */).await?;
    
    Ok(user.to_profile())
}

// API handlers coordinate and handle HTTP concerns
pub async fn update_own_profile(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state.database.pool.acquire().await?;
    
    // Business logic delegation
    let user = user_services::update_user_profile(&mut conn, auth_user.id, request).await?;
    
    // HTTP response formatting
    Ok(Json(ApiResponse::success(user)))
}
```

**Benefits:**
- **Testable:** Business logic can be tested without HTTP layer
- **Reusable:** Services can be called from CLI, background tasks, etc.
- **Focused:** Each layer has a single responsibility
- **Type-safe:** Rust's type system prevents many authorization bugs

## Common Questions

**Q: Can users update their own roles?**  
A: No. Role changes require admin privileges to prevent privilege escalation. Users can only update their profile information (username, email).

**Q: What happens when a user is deactivated?**  
A: All their active sessions are immediately invalidated across all devices, and they cannot log in until reactivated by a moderator or admin.

**Q: Can deleted users be recovered?**  
A: Yes, if soft delete was used (default). The account is marked inactive but data is preserved for 30 days. Hard delete is permanent and irreversible.

**Q: How do I make someone an admin?**  
A: Use the role update endpoint as an existing admin, or set `STARTER__INITIAL_ADMIN_PASSWORD` environment variable to create the initial admin account.

**Q: Can moderators create users?**  
A: No. User creation requires admin privileges. Moderators can manage existing users (status, password resets) but cannot create new accounts.

**Q: Are password changes logged?**  
A: Password changes trigger audit log entries with timestamps, but the actual passwords are never logged. Only hashed passwords are stored in the database.

## Next Steps

Now that you understand user management, explore related systems:

- **[Authentication & Authorization →](./02-authentication-and-authorization.md)** - Core auth concepts and RBAC implementation
- **[Testing →](./08-testing.md)** - Comprehensive testing strategies
- **[Web Frontend Integration →](./10-web-frontend-integration.md)** - React components for user management

---
*This user management system demonstrates production-ready patterns for user lifecycle management with security-first design.*