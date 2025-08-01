# Debugging and Troubleshooting Guide

*This guide teaches systematic debugging from first principles, helping you isolate and fix problems at any layer of the full-stack application.*

## 🤔 Why Systematic Debugging Matters

### The Problem with Random Debugging

Most developers approach debugging by:
- **Random clicking** - Trying solutions without understanding the problem
- **Copy-paste fixes** - Applying Stack Overflow solutions blindly
- **Symptom chasing** - Fixing what they see, not what's causing it
- **Tool hopping** - Switching between debuggers without a plan

### First Principles Debugging Approach

Systematic debugging means:
- **Understanding the system** - Know how components interact
- **Isolating the problem** - Narrow down to the failing layer
- **Testing hypotheses** - Form theories and test them methodically
- **Building knowledge** - Learn from each debugging session

## 🧠 Mental Model: Layer-Based Problem Isolation

Understanding where problems occur helps you choose the right debugging tools:

```mermaid
graph TD
    subgraph "🌐 Frontend Layer (React)"
        UI[UI Components<br/>State, Rendering, Events]
        CLIENT_API[API Client<br/>HTTP requests, Response parsing]
        AUTH_CLIENT[Authentication<br/>Token management, Context]
    end
    
    subgraph "🔄 Network Layer"
        HTTP[HTTP Protocol<br/>Status codes, Headers, CORS]
        SERIALIZATION[Data Serialization<br/>JSON parsing, Type conversion]
    end
    
    subgraph "⚙️ Backend Layer (Rust)"
        ROUTING[Request Routing<br/>URL matching, Method validation]
        MIDDLEWARE[Middleware<br/>Auth, CORS, Logging]
        HANDLERS[Route Handlers<br/>Business logic, Validation]
        SERVICES[Services<br/>Data processing, External calls]
    end
    
    subgraph "🗄️ Database Layer"
        CONNECTION[Connection Pool<br/>Connection management]
        QUERIES[SQL Queries<br/>Query execution, Transactions]
        DATA[Data Storage<br/>Tables, Constraints, Indexes]
    end
    
    UI --> CLIENT_API
    CLIENT_API --> HTTP
    HTTP --> ROUTING
    ROUTING --> MIDDLEWARE
    MIDDLEWARE --> HANDLERS
    HANDLERS --> SERVICES
    SERVICES --> CONNECTION
    CONNECTION --> QUERIES
    QUERIES --> DATA
    
    classDef frontend fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef network fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef backend fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef database fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class UI,CLIENT_API,AUTH_CLIENT frontend
    class HTTP,SERIALIZATION network
    class ROUTING,MIDDLEWARE,HANDLERS,SERVICES backend
    class CONNECTION,QUERIES,DATA database
```

**Key Insight**: Most problems occur at layer boundaries. Understanding these boundaries helps you isolate issues quickly.

## The First Principles Debugging Process

### Step 1: Identify the Layer

When something breaks, first determine which layer is failing:

```mermaid
flowchart TD
    PROBLEM[🚨 Problem Occurs] --> OBSERVE[👁️ Observe Symptoms]
    OBSERVE --> QUESTION{Which Layer?}
    
    QUESTION -->|UI not updating| FRONTEND[🌐 Frontend Layer]
    QUESTION -->|Request failing| NETWORK[🔄 Network Layer]
    QUESTION -->|Server error| BACKEND[⚙️ Backend Layer]
    QUESTION -->|Data issues| DATABASE[🗄️ Database Layer]
    
    FRONTEND --> FRONTEND_DEBUG[Frontend Debugging Tools]
    NETWORK --> NETWORK_DEBUG[Network Debugging Tools]
    BACKEND --> BACKEND_DEBUG[Backend Debugging Tools]
    DATABASE --> DATABASE_DEBUG[Database Debugging Tools]
    
    classDef layer fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef debug fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    
    class FRONTEND,NETWORK,BACKEND,DATABASE layer
    class FRONTEND_DEBUG,NETWORK_DEBUG,BACKEND_DEBUG,DATABASE_DEBUG debug
```

### Step 2: Layer-Specific Investigation

Each layer has specific investigation techniques:

#### 🌐 Frontend Layer Questions
- Is the component rendering correctly?
- Is the state updating as expected?
- Are API calls being made?
- Are authentication tokens present?

#### 🔄 Network Layer Questions
- What HTTP status code is returned?
- Are request headers correct?
- Is the request body properly formatted?
- Are CORS issues preventing the request?

#### ⚙️ Backend Layer Questions
- Is the endpoint being hit?
- Is authentication working?
- Are validation errors occurring?
- Is business logic executing correctly?

#### 🗄️ Database Layer Questions
- Is the database connection healthy?
- Are queries executing successfully?
- Are transactions completing?
- Are constraints being violated?

## Frontend Debugging Strategies

### React DevTools Investigation

**Component State Debugging**:
```typescript
// Add debugging props to components in development
export function TaskList({ tasks }: TaskListProps) {
  // Debug component props and state
  if (process.env.NODE_ENV === 'development') {
    console.log('TaskList render:', { tasks, count: tasks.length });
  }
  
  return (
    <div>
      {tasks.map(task => (
        <TaskItem key={task.id} task={task} />
      ))}
    </div>
  );
}
```

**State Management Debugging**:
```typescript
// Debug TanStack Query state
export function useTasksWithDebug() {
  const query = useQuery({
    queryKey: ['tasks'],
    queryFn: tasksApi.getTasks,
    onSuccess: (data) => {
      console.log('✅ Tasks loaded:', data);
    },
    onError: (error) => {
      console.error('❌ Tasks failed:', error);
    }
  });
  
  // Log state changes in development
  useEffect(() => {
    if (process.env.NODE_ENV === 'development') {
      console.log('Query state changed:', {
        isLoading: query.isLoading,
        isError: query.isError,
        data: query.data,
        error: query.error
      });
    }
  }, [query.isLoading, query.isError, query.data, query.error]);
  
  return query;
}
```

### Authentication Debugging

**Token Investigation**:
```typescript
// Debug authentication state
export function useAuthWithDebug() {
  const auth = useAuth();
  
  useEffect(() => {
    const token = localStorage.getItem('session_token');
    console.log('Auth Debug:', {
      authenticated: auth.authenticated,
      user: auth.user,
      tokenExists: !!token,
      tokenLength: token?.length,
      loading: auth.loading
    });
  }, [auth]);
  
  return auth;
}
```

**Common Auth Issues**:
```typescript
// Check for common authentication problems
export function debugAuth() {
  const token = localStorage.getItem('session_token');
  
  if (!token) {
    console.error('🔑 No token found in localStorage');
    return;
  }
  
  // Check token format (should be UUID-like)
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(token)) {
    console.error('🔑 Token format appears invalid:', token);
    return;
  }
  
  // Test token with API
  fetch('/api/v1/auth/me', {
    headers: { 
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  })
  .then(response => {
    if (response.ok) {
      console.log('✅ Token is valid');
    } else {
      console.error('❌ Token validation failed:', response.status);
    }
  })
  .catch(error => {
    console.error('🌐 Network error during token validation:', error);
  });
}
```

### API Client Debugging

**Request/Response Logging**:
```typescript
// Enhanced API client with debugging
export const apiClient = {
  async request(url: string, options: RequestInit = {}) {
    const requestId = crypto.randomUUID().slice(0, 8);
    const startTime = performance.now();
    
    // Log outgoing request
    console.group(`🚀 [${requestId}] ${options.method || 'GET'} ${url}`);
    console.log('Headers:', options.headers);
    console.log('Body:', options.body);
    
    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });
      
      const duration = performance.now() - startTime;
      const responseBody = await response.clone().text();
      
      // Log response
      console.log(`📥 Response (${duration.toFixed(2)}ms):`, {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        body: responseBody
      });
      
      if (!response.ok) {
        console.error('❌ HTTP Error:', response.status, response.statusText);
        try {
          const errorData = JSON.parse(responseBody);
          console.error('Error details:', errorData);
        } catch {
          console.error('Raw error response:', responseBody);
        }
      }
      
      console.groupEnd();
      return response;
      
    } catch (error) {
      const duration = performance.now() - startTime;
      console.error(`💥 Network Error (${duration.toFixed(2)}ms):`, error);
      console.groupEnd();
      throw error;
    }
  }
};
```

### Browser DevTools Usage

**Network Tab Investigation**:
1. **Failed Requests**: Look for red status codes (4xx, 5xx)
2. **Request Headers**: Verify `Authorization`, `Content-Type`
3. **Response Headers**: Check `Access-Control-Allow-Origin` for CORS
4. **Request/Response Bodies**: Inspect actual data being sent

**Console Debugging**:
```javascript
// Check authentication state
localStorage.getItem('session_token')

// Test API connectivity
fetch('/api/v1/health').then(r => r.json()).then(console.log)

// Check React Query cache
window.__REACT_QUERY_DEVTOOLS_CACHE__

// Inspect TanStack Router state
window.__TSR__
```

## Backend Debugging Strategies

### Rust Application Logging

**Structured Logging Setup**:
```rust
// starter/src/main.rs
use tracing::{info, error, debug, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Usage in route handlers
#[tracing::instrument(skip(state))]
pub async fn create_task(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    let span = span!(Level::INFO, "create_task", user_id = %user.id, task_type = %request.task_type);
    let _enter = span.enter();
    
    info!("Creating task for user {}", user.id);
    debug!("Task payload: {:?}", request.payload);
    
    // Business logic...
    
    info!("Task created successfully");
    Ok(Json(ApiResponse::success(task)))
}
```

**Error Context Enhancement**:
```rust
// starter/src/errors.rs
use tracing::error;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log detailed error information
        match &self {
            AppError::DatabaseError(e) => {
                error!("Database error: {:?}", e);
            }
            AppError::ValidationError(msg) => {
                error!("Validation error: {}", msg);
            }
            AppError::Unauthorized => {
                error!("Unauthorized access attempt");
            }
        }
        
        // Return user-friendly response
        let (status_code, error_code, message) = match &self {
            // ... error mapping
        };
        
        (status_code, Json(ErrorResponse { error: ErrorDetail { code: error_code.to_string(), message } })).into_response()
    }
}
```

### Admin CLI Debugging Tools

The application includes powerful CLI tools for debugging:

**Task System Debugging**:
```bash
# Check task statistics without API authentication
cargo run -- admin task-stats

# Detailed task statistics with filtering
cargo run -- admin task-stats --tag "chaos_test" --verbose

# List recent tasks with full details
cargo run -- admin list-tasks --limit 20 --verbose

# Check for stuck or problematic tasks
cargo run -- admin list-tasks --status processing --limit 50
```

**Database Debugging**:
```bash
# Direct database queries via admin CLI
cargo run -- admin db-query "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '1 day'"

# Check database connectivity
cargo run -- admin health-check --component database

# Clear test data
cargo run -- admin clear-completed --dry-run
cargo run -- admin clear-completed --older-than "7 days"
```

### Health Check Investigation

**Comprehensive Health Debugging**:
```bash
# Basic health check
curl http://localhost:3000/api/v1/health | jq

# Detailed health with dependency checks
curl http://localhost:3000/api/v1/health/detailed | jq

# Kubernetes-style health probes
curl http://localhost:3000/api/v1/health/live     # Minimal check
curl http://localhost:3000/api/v1/health/ready    # Dependency validation
curl http://localhost:3000/api/v1/health/startup  # Initialization status
```

**Health Check Response Analysis**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": "2h 30m 15s",
    "timestamp": "2024-01-15T10:30:00Z",
    "database": {
      "status": "healthy",
      "connection_pool": {
        "size": 10,
        "idle": 8,
        "used": 2
      }
    },
    "documentation_links": {
      "api_docs": "/api-docs",
      "openapi_spec": "/api-docs/openapi.json"
    }
  }
}
```

## Database Debugging Strategies

### Connection Pool Investigation

**Pool Status Monitoring**:
```rust
// starter/src/health/services.rs
use sqlx::postgres::PgPoolOptions;

pub async fn get_database_health(database: &Database) -> DatabaseHealthInfo {
    let pool = &database.pool;
    
    let health_info = DatabaseHealthInfo {
        status: "healthy".to_string(),
        connection_pool: ConnectionPoolInfo {
            size: pool.size(),
            idle: pool.num_idle(),
            used: pool.size() - pool.num_idle(),
        },
        // Test actual connectivity
        connectivity: test_database_query(pool).await,
    };
    
    health_info
}

async fn test_database_query(pool: &PgPool) -> String {
    match sqlx::query_scalar!("SELECT 1 as test")
        .fetch_one(pool)
        .await 
    {
        Ok(_) => "connected".to_string(),
        Err(e) => {
            error!("Database connectivity test failed: {:?}", e);
            "disconnected".to_string()
        }
    }
}
```

### SQL Query Debugging

**Query Logging Setup**:
```bash
# Enable SQL query logging
export RUST_LOG=sqlx=debug,starter=info

# Start application with query logging
cargo run server
```

**Manual Database Investigation**:
```bash
# Connect to database directly
psql postgresql://starter_user:starter_pass@localhost:5432/starter_db

# Check active connections
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

# Check table sizes
SELECT 
  schemaname,
  tablename,
  attname,
  n_distinct,
  null_frac
FROM pg_stats 
WHERE schemaname = 'public';

# Check recent queries
SELECT query, state, query_start 
FROM pg_stat_activity 
WHERE state != 'idle' 
ORDER BY query_start DESC;
```

**Database Error Pattern Analysis**:
```sql
-- Check for constraint violations
SELECT 
  conname as constraint_name,
  contype as constraint_type,
  conrelid::regclass as table_name
FROM pg_constraint 
WHERE contype IN ('c', 'f', 'p', 'u');

-- Check for deadlocks
SELECT 
  blocked_locks.pid AS blocked_pid,
  blocked_activity.usename AS blocked_user,
  blocking_locks.pid AS blocking_pid,
  blocking_activity.usename AS blocking_user,
  blocked_activity.query AS blocked_statement,
  blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.GRANTED;
```

## Common Problem Patterns and Solutions

### Authentication Issues

#### Problem: "Invalid credentials" on valid login

**Investigation Steps**:
1. **Check password hashing**:
   ```bash
   # Test password hashing directly
   cargo run -- admin verify-password --username "testuser" --password "testpass"
   ```

2. **Database user lookup**:
   ```sql
   SELECT id, username, email, password_hash, created_at 
   FROM users 
   WHERE username = 'testuser' OR email = 'testuser';
   ```

3. **Argon2 verification**:
   ```rust
   // In authentication service
   use argon2::{Argon2, PasswordHash, PasswordVerifier};
   
   pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
       let parsed_hash = PasswordHash::new(hash)
           .map_err(|e| AuthError::InvalidPasswordHash(e.to_string()))?;
       
       match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
           Ok(()) => Ok(true),
           Err(_) => Ok(false),
       }
   }
   ```

#### Problem: Token expires immediately

**Investigation Steps**:
1. **Check session creation**:
   ```sql
   SELECT session_token, expires_at, created_at, is_active 
   FROM user_sessions 
   WHERE user_id = 'user-uuid-here' 
   ORDER BY created_at DESC;
   ```

2. **Verify token format**:
   ```bash
   # Token should be UUID format
   echo "token-here" | grep -E '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
   ```

3. **Check system time**:
   ```bash
   # Ensure server time is correct
   date
   # Check timezone
   timedatectl status
   ```

### Task Processing Issues

#### Problem: Tasks stuck in "processing" state

**Investigation Steps**:
1. **Check worker status**:
   ```bash
   # Is worker running?
   ps aux | grep starter | grep worker
   
   # Check worker logs
   tail -f /tmp/starter-worker-0.log   # Default worker (ID 0)
   # tail -f /tmp/starter-worker-*.log # All workers
   ```

2. **Database investigation**:
   ```sql
   -- Find stuck tasks
   SELECT id, task_type, status, created_at, updated_at, retry_count
   FROM tasks 
   WHERE status = 'processing' 
   AND updated_at < NOW() - INTERVAL '10 minutes';
   
   -- Check for worker registration
   SELECT task_type, description, created_at 
   FROM task_types;
   ```

3. **Reset stuck tasks**:
   ```bash
   # Reset tasks to pending for retry
   cargo run -- admin reset-stuck-tasks --older-than "10 minutes"
   ```

#### Problem: High task failure rates

**Investigation Steps**:
1. **Error pattern analysis**:
   ```sql
   SELECT 
     task_type,
     error_message,
     COUNT(*) as failure_count
   FROM tasks 
   WHERE status = 'failed' 
   AND created_at > NOW() - INTERVAL '1 day'
   GROUP BY task_type, error_message
   ORDER BY failure_count DESC;
   ```

2. **Dead letter queue investigation**:
   ```bash
   # Check failed tasks in detail
   curl http://localhost:3000/api/v1/tasks/dead-letter | jq '.[0:5]'
   ```

3. **Task handler debugging**:
   ```rust
   // Add detailed logging to task handlers
   #[async_trait]
   impl TaskHandler for EmailTaskHandler {
       async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
           info!("Processing email task: {:?}", context.payload);
           
           match send_email(&context.payload).await {
               Ok(result) => {
                   info!("Email sent successfully: {:?}", result);
                   Ok(TaskResult::Success)
               }
               Err(e) => {
                   error!("Email send failed: {:?}", e);
                   Err(TaskError::ProcessingError(e.to_string()))
               }
           }
       }
   }
   ```

### Network and CORS Issues

#### Problem: CORS errors in browser

**Investigation Steps**:
1. **Check CORS configuration**:
   ```rust
   // starter/src/main.rs
   use tower_http::cors::CorsLayer;
   
   let cors = CorsLayer::new()
       .allow_origin(Any)  // Development only!
       .allow_methods(Any)
       .allow_headers(Any);
   ```

2. **Browser network inspection**:
   - Look for preflight OPTIONS requests
   - Check `Access-Control-Allow-Origin` header
   - Verify request/response headers

3. **Manual CORS testing**:
   ```bash
   # Test CORS headers
   curl -H "Origin: http://localhost:5173" \
        -H "Access-Control-Request-Method: POST" \
        -H "Access-Control-Request-Headers: X-Requested-With" \
        -X OPTIONS \
        http://localhost:3000/api/v1/auth/login
   ```

### Performance Issues

#### Problem: Slow database queries

**Investigation Steps**:
1. **Query performance analysis**:
   ```sql
   -- Enable query timing
   \timing on
   
   -- Check slow queries
   SELECT query, mean_time, calls, total_time 
   FROM pg_stat_statements 
   ORDER BY mean_time DESC 
   LIMIT 10;
   ```

2. **Index analysis**:
   ```sql
   -- Check missing indexes
   SELECT 
     schemaname,
     tablename,
     attname,
     n_distinct,
     correlation
   FROM pg_stats 
   WHERE schemaname = 'public' 
   AND n_distinct > 100;
   ```

3. **Connection pool tuning**:
   ```rust
   // Adjust pool settings
   let pool = PgPoolOptions::new()
       .max_connections(20)  // Increase for high load
       .min_connections(5)   // Maintain minimum connections
       .acquire_timeout(Duration::from_secs(8))
       .idle_timeout(Duration::from_secs(600))
       .max_lifetime(Duration::from_secs(1800))
       .connect(&database_url)
       .await?;
   ```

## Debugging Workflow Template

### 1. Problem Description Template

```markdown
## Problem Description
- **What were you trying to do?**
- **What did you expect to happen?**
- **What actually happened?**
- **How can the problem be reproduced?**

## Environment Information
- **Browser**: Chrome 120.0.0.0
- **Operating System**: macOS 14.2
- **Application Version**: git commit hash
- **Database**: PostgreSQL 16

## Error Information
- **Error Messages**: (copy exact error text)
- **HTTP Status Codes**: (if applicable)
- **Console Logs**: (browser console output)
- **Server Logs**: (backend application logs)
```

### 2. Systematic Investigation Checklist

```markdown
## Investigation Checklist

### Layer Identification
- [ ] Frontend issue (UI, state, API calls)
- [ ] Network issue (HTTP, CORS, connectivity)
- [ ] Backend issue (routing, logic, validation)
- [ ] Database issue (queries, connections, data)

### Frontend Debugging (if applicable)
- [ ] Checked React DevTools for component state
- [ ] Verified API client requests in Network tab
- [ ] Confirmed authentication token exists and format
- [ ] Checked console for JavaScript errors

### Backend Debugging (if applicable)
- [ ] Verified endpoint is being hit (logs)
- [ ] Checked authentication middleware behavior
- [ ] Confirmed request validation passes
- [ ] Verified database operations complete

### Database Debugging (if applicable)
- [ ] Confirmed database connectivity
- [ ] Checked connection pool status
- [ ] Verified queries execute successfully
- [ ] Checked for constraint violations

### Resolution
- [ ] Root cause identified
- [ ] Fix implemented and tested
- [ ] Documentation updated (if needed)
- [ ] Prevention measures considered
```

## Advanced Debugging Techniques

### Distributed Tracing Setup

**OpenTelemetry Integration**:
```rust
// Add to Cargo.toml
// tracing-opentelemetry = "0.21"
// opentelemetry = "0.21"

use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry::trace::TracerProvider;

pub fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("rust-fullstack-starter")
        .install_simple()?;

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

### Load Testing and Performance Debugging

**Simple Load Testing**:
```bash
# Install hey (HTTP load testing tool)
# brew install hey  # macOS
# sudo apt install hey  # Ubuntu

# Load test authentication endpoint
hey -n 1000 -c 10 -m POST \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass"}' \
  http://localhost:3000/api/v1/auth/login

# Load test with authentication
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass"}' | jq -r '.data.session_token')

hey -n 500 -c 5 \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/tasks
```

### Memory and Resource Debugging

**Resource Monitoring**:
```bash
# Monitor application resource usage
# Install htop, iotop for detailed monitoring

# Monitor specific process
PID=$(pgrep -f "starter server")
top -p $PID

# Memory usage over time
while true; do
  ps -p $PID -o pid,ppid,cmd,%mem,%cpu --no-headers
  sleep 5
done

# Database connection monitoring
watch -n 2 "psql postgresql://starter_user:starter_pass@localhost:5432/starter_db -c \"SELECT count(*) as active_connections FROM pg_stat_activity WHERE state = 'active';\""
```

## Testing Your Debugging Skills

### Practice Scenarios

Create these problems intentionally to practice debugging:

1. **Authentication Failure Scenario**:
   ```bash
   # Corrupt a user's password hash in database
   UPDATE users SET password_hash = 'invalid_hash' WHERE username = 'testuser';
   
   # Practice: Debug why login fails
   ```

2. **Task Processing Failure**:
   ```bash
   # Stop the worker process
   pkill -f "starter worker"
   
   # Create tasks via API
   # Practice: Debug why tasks stay in pending state
   ```

3. **Database Connection Issues**:
   ```bash
   # Simulate connection exhaustion
   # Practice: Debug connection pool problems
   ```

### Debugging Exercises

1. **Trace a Successful Request**: Follow a login request through all layers
2. **Identify Layer Boundaries**: Map where each type of error occurs
3. **Performance Investigation**: Find the slowest operation in your system
4. **Error Correlation**: Connect frontend errors to backend logs

## Next Steps: Building Debugging Expertise

### 1. Practice Regularly
- Set aside time each week for debugging practice
- Intentionally break things to understand failure modes
- Document common problems and solutions

### 2. Build Tools
- Create debugging scripts for common investigations
- Set up monitoring dashboards
- Automate health checks and alerts

### 3. Share Knowledge
- Document debugging solutions for your team
- Create runbooks for common issues
- Teach others your debugging techniques

### 4. Learn from Each Bug
- Keep a debugging journal
- Note patterns in problems you encounter
- Identify areas where better tooling would help

**Related Guides**:
- [Web Frontend Integration](10-web-frontend-integration.md) - Full-stack debugging strategies
- [Testing Guide](08-testing.md) - Testing approaches that prevent bugs
- [Chaos Testing](09-chaos-testing.md) - Proactive failure discovery
- [Production Deployment](../production-deployment.md) - Production debugging tools

---

*Systematic debugging is a skill that improves with practice. Use these first principles and layer-based approaches to become a more effective problem solver.*