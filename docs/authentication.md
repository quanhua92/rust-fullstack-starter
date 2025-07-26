# Authentication Guide

This guide explains how the session-based authentication system works in the Rust Full-Stack Starter.

## Overview

The starter implements session-based authentication using secure tokens stored in the database. This approach is simple, secure, and scalable for most applications.

## Authentication Flow

For complete API endpoint documentation with request/response examples, see [api-endpoints.md](./api-endpoints.md).

### 1. User Registration

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "user@example.com", 
    "password": "SecurePassword123!"
  }'
```

**Response:**
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

### 2. User Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "user@example.com",
    "password": "SecurePassword123!"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session_token": "ABc123...64-char-token...",
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

### 3. Accessing Protected Routes

Use the session token in the `Authorization` header:

```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer ABc123...64-char-token..."
```

**Response:**
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

### 4. Logout

```bash
curl -X POST http://localhost:3000/auth/logout \
  -H "Authorization: Bearer ABc123...64-char-token..."
```

## Implementation Details

### Session Token Generation

```rust
fn generate_session_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..64)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

### Authentication Middleware

The middleware validates session tokens on protected routes:

```rust
pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Bearer token from Authorization header
    let token = extract_bearer_token(&req)?;
    
    // Validate session and get user
    let mut conn = app_state.database.pool.acquire().await?;
    let user = services::validate_session_with_user(&mut conn, &token).await?;
    
    // Add user to request context
    req.extensions_mut().insert(AuthUser {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    });
    
    Ok(next.run(req).await)
}
```

### Password Security

Passwords are hashed using Argon2:

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

// Hashing during registration
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

// Verification during login
let parsed_hash = PasswordHash::new(&user.password_hash)?;
let is_valid = Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();
```

## Module Structure

### Auth Module (`src/auth/`)

- **`api.rs`** - HTTP endpoint handlers
- **`models.rs`** - Session and request/response models
- **`services.rs`** - Business logic for authentication
- **`middleware.rs`** - Route protection and user context
- **`cleanup.rs`** - Background session cleanup

### Users Module (`src/users/`)

- **`api.rs`** - User management endpoints  
- **`models.rs`** - User models and validation
- **`services.rs`** - User CRUD operations

## Security Features

### Session Management

- **Secure Tokens**: 64-character alphanumeric tokens
- **Expiration**: 24-hour automatic expiry
- **Activity Tracking**: Last activity timestamp updates
- **Cleanup**: Automatic expired session removal

### Role-Based Access

```rust
// Admin-only middleware
pub async fn admin_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = req.extensions().get::<AuthUser>()?;
    
    if auth_user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

### Input Validation

```rust
impl LoginRequest {
    pub fn validate(&self) -> Result<()> {
        if self.username_or_email.trim().is_empty() {
            return Err(Error::validation("username_or_email", "Cannot be empty"));
        }
        if self.password.is_empty() {
            return Err(Error::validation("password", "Cannot be empty"));
        }
        Ok(())
    }
}
```

## Testing Authentication

Use the provided test script:

```bash
./scripts/test_auth.sh
```

This script tests:
- ✅ User registration
- ✅ User login with token generation  
- ✅ Protected route access
- ✅ Unauthorized access blocking
- ✅ Session cleanup after logout
- ✅ Invalid credential rejection

## Extending Authentication

### Adding New Protected Routes

```rust
// In server.rs
let protected_routes = Router::new()
    .route("/api/profile", get(users_api::get_profile))
    .route("/api/settings", put(users_api::update_settings))
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
```

### Adding Role-Specific Routes

```rust
// Admin-only routes
let admin_routes = Router::new()
    .route("/admin/users", get(admin_api::list_users))
    .layer(middleware::from_fn(admin_middleware))
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
```

### Custom Authentication Logic

Create new services in `src/auth/services.rs`:

```rust
pub async fn reset_password(
    conn: &mut DbConn,
    email: &str,
    new_password: &str,
) -> Result<()> {
    // Implementation here
}

pub async fn verify_email(
    conn: &mut DbConn, 
    token: &str,
) -> Result<()> {
    // Implementation here
}
```

## Configuration

Session behavior can be configured via environment variables:

```bash
# Session expiry (in hours, default: 24)
STARTER__AUTH__SESSION_EXPIRY_HOURS=48

# Cleanup interval (in seconds, default: 3600)  
STARTER__AUTH__CLEANUP_INTERVAL_SECS=1800
```

## Best Practices

1. **Always validate tokens** on protected routes
2. **Use HTTPS** in production to protect tokens in transit
3. **Implement rate limiting** on auth endpoints
4. **Log authentication events** for security monitoring
5. **Regularly clean up** expired sessions
6. **Use strong passwords** and consider 2FA for sensitive applications
7. **Rotate session tokens** on privilege changes

## Common Patterns

### Optional Authentication

```rust
// Route that works with or without auth
pub async fn public_with_optional_auth(
    Extension(auth_user): Extension<Option<AuthUser>>,
) -> Json<ApiResponse<String>> {
    match auth_user {
        Some(user) => Json(ApiResponse::success(format!("Hello, {}!", user.username))),
        None => Json(ApiResponse::success("Hello, anonymous!".to_string())),
    }
}
```

### Service Function Pattern

```rust
// All service functions follow this pattern
pub async fn service_function(
    conn: &mut DbConn,          // Database connection first
    param1: Type1,              // Function parameters
    param2: Type2,
) -> Result<ReturnType> {
    // Business logic implementation
}
```

This authentication system provides a solid foundation that you can extend based on your application's specific requirements!