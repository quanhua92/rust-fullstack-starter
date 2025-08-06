# Ownership-Based RBAC Pattern

*This guide explains the ownership-based Role-Based Access Control (RBAC) pattern used throughout the application, from concepts to implementation to migration strategies.*

## Overview

The ownership-based RBAC pattern provides secure, intuitive access control where:
- **Users own their data** - Can create, read, update, delete their own resources
- **Admins and Moderators have elevated access** - Can access and manage all resources
- **Bulk operations require elevated permissions** - Moderator+ role required for bulk create/update/delete

This pattern balances security with usability, ensuring users can manage their own data while providing administrative oversight capabilities.

## Architecture

### Database Schema Pattern

Every resource that uses ownership-based access control includes a `created_by` field:

```sql
CREATE TABLE example_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for ownership queries (critical for performance)
CREATE INDEX idx_example_table_created_by ON example_table(created_by);
```

### Connection Architecture

The application uses `DbConn = sqlx::PgConnection` to support both pool connections and transactions:

```rust
pub type DbConn = sqlx::PgConnection;

// For simple operations - use pool connection
let mut conn = pool.acquire().await?;
service_function(conn.as_mut(), request).await?;

// For operations requiring ownership checks - use transactions
let mut tx = pool.begin().await?;
let existing = get_resource_service(tx.as_mut(), id).await?;
rbac_services::can_access_own_resource(&auth_user, existing.created_by)?;
let updated = update_resource_service(tx.as_mut(), id, request).await?;
tx.commit().await?;
```

## Implementation Patterns

### Pattern 1: Individual CRUD Operations (Ownership-Based)

```rust
use crate::rbac::services as rbac_services;

pub async fn update_resource(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateResourceRequest>,
) -> Result<Json<ApiResponse<Resource>>> {
    // Begin transaction for atomic get-check-update
    let mut tx = app_state.database.pool.begin().await?;
    
    // Get existing resource to check ownership
    let existing = get_resource_service(tx.as_mut(), id).await?;
    
    // Ownership check: Users can access their own, Admin/Moderator can access all
    rbac_services::can_access_own_resource(&auth_user, existing.created_by)?;
    
    // Update resource
    let updated = update_resource_service(tx.as_mut(), id, request).await?;
    
    // Commit transaction
    tx.commit().await?;
    Ok(Json(ApiResponse::success(updated)))
}
```

### Pattern 2: Bulk Operations (Role-Based)

```rust
pub async fn bulk_create_resources(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkCreateRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<Resource>>>> {
    // Bulk operations require moderator or higher permissions
    rbac_services::require_moderator_or_higher(&auth_user)?;
    
    let mut conn = app_state.database.pool.acquire().await?;
    let response = bulk_create_service(conn.as_mut(), request).await?;
    Ok(Json(ApiResponse::success(response)))
}
```

### Pattern 3: Create Operations (Auto-Ownership)

```rust
pub async fn create_resource(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateResourceRequest>,
) -> Result<Json<ApiResponse<Resource>>> {
    let mut conn = app_state.database.pool.acquire().await?;
    
    // Automatically set created_by to current user
    let resource = create_resource_service(conn.as_mut(), request, auth_user.id).await?;
    Ok(Json(ApiResponse::success(resource)))
}
```

## Service Layer Implementation

### Service Function Signatures

```rust
// Create - requires created_by parameter
pub async fn create_resource_service(
    conn: &mut DbConn,
    request: CreateResourceRequest,
    created_by: Uuid,
) -> Result<Resource> {
    let resource = Resource::new(request.name, request.description, created_by);
    
    let created = sqlx::query_as!(
        Resource,
        "INSERT INTO resources (id, name, description, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, name, description, created_by, created_at, updated_at",
        resource.id,
        resource.name,
        resource.description,
        resource.created_by,
        resource.created_at,
        resource.updated_at
    )
    .fetch_one(&mut *conn)  // Use &mut *conn for reborrow
    .await
    .map_err(Error::from_sqlx)?;
    
    Ok(created)
}

// Read operations include created_by in SELECT
pub async fn get_resource_service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<Resource> {
    let resource = sqlx::query_as!(
        Resource,
        "SELECT id, name, description, created_by, created_at, updated_at 
         FROM resources WHERE id = $1",
        id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?
    .ok_or_else(|| Error::NotFound(format!("Resource with id {}", id)))?;
    
    Ok(resource)
}
```

## RBAC Functions

### Core Access Control Function

```rust
/// Check if a user can access a resource based on ownership and role
/// Admin/Moderator can access any resource, users can only access their own
pub fn can_access_own_resource(user: &AuthUser, resource_owner: Uuid) -> Result<(), Error> {
    match user.role {
        // Admin and Moderator can access any resource
        UserRole::Admin | UserRole::Moderator => Ok(()),
        // Users can only access their own resources
        UserRole::User => {
            if resource_owner == user.id {
                Ok(())
            } else {
                // Return 404 (not 403) to prevent information leakage
                Err(Error::NotFound("Resource not found".to_string()))
            }
        }
    }
}
```

### Role-Based Functions

```rust
/// Require moderator or higher for bulk operations
pub fn require_moderator_or_higher(user: &AuthUser) -> Result<(), Error> {
    match user.role {
        UserRole::Admin | UserRole::Moderator => Ok(()),
        UserRole::User => Err(Error::Forbidden("Insufficient permissions".to_string())),
    }
}
```

## Security Considerations

### Transaction Safety

All ownership checks use database transactions to ensure atomicity:

```rust
// ✅ Correct: Atomic get-check-update
let mut tx = pool.begin().await?;
let existing = get_resource_service(tx.as_mut(), id).await?;
can_access_own_resource(&auth_user, existing.created_by)?;
let updated = update_resource_service(tx.as_mut(), id, request).await?;
tx.commit().await?;

// ❌ Incorrect: Race condition possible
let mut conn = pool.acquire().await?;
let existing = get_resource_service(conn.as_mut(), id).await?;
can_access_own_resource(&auth_user, existing.created_by)?;
// Resource could be deleted/modified here by another request
let updated = update_resource_service(conn.as_mut(), id, request).await?;
```

### Information Leakage Prevention

Always return 404 (not 403) for ownership violations to prevent information leakage:

```rust
// ✅ Correct: No information leakage
if resource_owner != user.id {
    return Err(Error::NotFound("Resource not found".to_string()));
}

// ❌ Incorrect: Reveals resource existence
if resource_owner != user.id {
    return Err(Error::Forbidden("Access denied".to_string()));
}
```

## Migration Guide

### Adding Ownership to Existing Resources

1. **Add Database Column**:
```sql
-- Add the column (non-nullable)
ALTER TABLE existing_table ADD COLUMN created_by UUID;

-- Set default value for existing records (use admin user ID)
UPDATE existing_table SET created_by = (SELECT id FROM users WHERE role = 'admin' LIMIT 1);

-- Add NOT NULL constraint
ALTER TABLE existing_table ALTER COLUMN created_by SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE existing_table ADD CONSTRAINT fk_existing_table_created_by 
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE;

-- Add performance index
CREATE INDEX idx_existing_table_created_by ON existing_table(created_by);
```

2. **Update Model Structs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExistingResource {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,  // Add this field
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExistingResource {
    pub fn new(name: String, created_by: Uuid) -> Self {  // Add created_by parameter
        Self {
            id: Uuid::new_v4(),
            name,
            created_by,  // Set this field
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
```

3. **Update Service Functions**:
```rust
// Update create function signature
pub async fn create_existing_resource_service(
    conn: &mut DbConn,
    request: CreateExistingResourceRequest,
    created_by: Uuid,  // Add this parameter
) -> Result<ExistingResource> {
    let resource = ExistingResource::new(request.name, created_by);
    // ... rest of implementation with created_by in INSERT
}

// Update all SELECT queries to include created_by
pub async fn get_existing_resource_service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<ExistingResource> {
    let resource = sqlx::query_as!(
        ExistingResource,
        "SELECT id, name, created_by, created_at, updated_at   -- Add created_by here
         FROM existing_table WHERE id = $1",
        id
    )
    // ... rest of implementation
}
```

4. **Update API Handlers**:
```rust
// Update create handler
pub async fn create_existing_resource(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateExistingResourceRequest>,
) -> Result<Json<ApiResponse<ExistingResource>>> {
    let mut conn = app_state.database.pool.acquire().await?;
    let resource = create_existing_resource_service(conn.as_mut(), request, auth_user.id).await?;  // Pass auth_user.id
    Ok(Json(ApiResponse::success(resource)))
}

// Update update/delete handlers with ownership checks
pub async fn update_existing_resource(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateExistingResourceRequest>,
) -> Result<Json<ApiResponse<ExistingResource>>> {
    let mut tx = app_state.database.pool.begin().await?;
    let existing = get_existing_resource_service(tx.as_mut(), id).await?;
    rbac_services::can_access_own_resource(&auth_user, existing.created_by)?;  // Add ownership check
    let updated = update_existing_resource_service(tx.as_mut(), id, request).await?;
    tx.commit().await?;
    Ok(Json(ApiResponse::success(updated)))
}
```

## Testing Ownership Patterns

### Integration Tests

```rust
#[tokio::test]
async fn test_ownership_access_control() {
    let app_state = setup_test_database().await;
    
    // Create two users
    let user1 = create_test_user("user1").await;
    let user2 = create_test_user("user2").await;
    
    // User1 creates a resource
    let resource = create_resource_as_user(&app_state, &user1, "Test Resource").await;
    
    // User1 can access their own resource
    let result = get_resource_as_user(&app_state, &user1, resource.id).await;
    assert!(result.is_ok());
    
    // User2 cannot access User1's resource
    let result = get_resource_as_user(&app_state, &user2, resource.id).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Resource not found");
    
    // Admin can access any resource
    let admin = create_test_user_with_role("admin", UserRole::Admin).await;
    let result = get_resource_as_user(&app_state, &admin, resource.id).await;
    assert!(result.is_ok());
}
```

## Best Practices

1. **Always use transactions for ownership checks** - Prevents race conditions
2. **Return 404 for ownership violations** - Prevents information leakage
3. **Index created_by fields** - Essential for query performance
4. **Use ownership for individual operations** - Intuitive user experience
5. **Use role-based access for bulk operations** - Administrative control
6. **Test all access patterns thoroughly** - Security is critical

## Performance Considerations

1. **Index Strategy**:
   - Always create indexes on `created_by` fields
   - Consider composite indexes for common query patterns
   
2. **Query Optimization**:
   - Use `created_by` in WHERE clauses for automatic filtering
   - Avoid N+1 queries when loading related ownership data
   
3. **Connection Management**:
   - Use `conn.as_mut()` for pool connections
   - Use `tx.as_mut()` for transaction connections
   - Proper reborrow with `&mut *conn` in service functions

This ownership-based RBAC pattern provides a secure, scalable foundation for access control while maintaining code clarity and performance.