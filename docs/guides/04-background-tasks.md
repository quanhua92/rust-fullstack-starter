# Background Tasks System

*This guide explains how the async task processing system works, building on the reliability patterns you learned previously.*

## Why Background Tasks?

Some operations shouldn't block HTTP requests:
- **Sending emails**: Network calls can be slow
- **Processing data**: Heavy computations take time  
- **Calling webhooks**: External APIs might be unreliable
- **Generating reports**: File operations can take minutes

**The Solution**: Queue these operations as **background tasks** and process them asynchronously.

## System Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Request  │    │   Task Queue    │    │ Background      │
│                 │    │   (Database)    │    │ Worker          │
│ POST /tasks ────┼───▶│                 │◀───┼─ Processor      │
│ "Send email"    │    │ ┌─────────────┐ │    │                 │
│                 │    │ │   Tasks     │ │    │ ┌─────────────┐ │
│ Response: 201   │◀───┤ │   Table     │ │    │ │  Handlers   │ │
│ "Task created"  │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    │                 │    │                 │
                       │ ┌─────────────┐ │    │ ┌─────────────┐ │
                       │ │Retry Logic  │ │    │ │Circuit      │ │
                       │ └─────────────┘ │    │ │Breakers     │ │
                       └─────────────────┘    │ └─────────────┘ │
                                              └─────────────────┘
```

**Key Insight**: The HTTP API and the worker are **separate processes**. The database acts as the communication layer between them.

## Task Lifecycle

### States and Transitions
```
Create → [pending] → [running] → [completed]
            ↓            ↓
         [scheduled]  [failed] → [retrying] → [running]
                         ↓           ↓
                    [dead letter] [cancelled]
```

### State Definitions
- **pending**: Ready to be processed
- **scheduled**: Waiting until a specific time
- **running**: Currently being processed by a worker
- **completed**: Successfully finished
- **failed**: Failed after all retry attempts (dead letter)
- **retrying**: Failed but will try again later
- **cancelled**: Manually cancelled by user

## Data Model Deep Dive

### Tasks Table
```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR NOT NULL,           -- 'email', 'webhook', etc.
    payload JSONB NOT NULL,               -- Task-specific data
    status task_status NOT NULL DEFAULT 'pending',
    priority task_priority NOT NULL DEFAULT 'normal',
    
    -- Retry configuration
    retry_strategy JSONB,                 -- How to retry failures
    max_attempts INTEGER NOT NULL DEFAULT 3,
    current_attempt INTEGER NOT NULL DEFAULT 0,
    
    -- Error tracking
    last_error TEXT,                      -- Last failure message
    
    -- Timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_at TIMESTAMPTZ,             -- When to execute (NULL = now)
    started_at TIMESTAMPTZ,               -- When processing began
    completed_at TIMESTAMPTZ,             -- When finished
    
    -- Ownership
    created_by UUID REFERENCES users(id),
    
    -- Additional context
    metadata JSONB DEFAULT '{}'
);
```

### Why JSONB for Payload?
```rust
// Different task types need different data
EmailTask {
    to: "user@example.com",
    subject: "Welcome!",
    body: "Thanks for signing up"
}

WebhookTask {
    url: "https://api.example.com/notify",
    method: "POST", 
    payload: { "event": "user_created", "user_id": "123" }
}

// JSONB stores both flexibly while maintaining query performance
```

### Priority System
```rust
pub enum TaskPriority {
    Critical,  // Process immediately
    High,      // Process before normal
    Normal,    // Default priority (most tasks)
    Low,       // Process when nothing else pending
}

// Database query respects priority:
// ORDER BY priority DESC, created_at ASC
```

## Worker Architecture

### Main Processing Loop
```rust
pub async fn start_worker(&self) -> Result<()> {
    info!("Worker starting with {} concurrency", self.config.max_concurrent_tasks);
    
    let mut interval = interval(self.config.poll_interval);
    
    loop {
        interval.tick().await;
        
        if let Err(e) = self.process_batch().await {
            error!("Error processing task batch: {}", e);
            // Continue running - don't crash on individual failures
        }
    }
}

async fn process_batch(&self) -> Result<()> {
    // Fetch ready tasks from database
    let mut conn = self.database.pool.acquire().await?;
    let tasks = self.fetch_ready_tasks(&mut conn).await?;
    
    if tasks.is_empty() {
        return Ok(()); // Nothing to do
    }
    
    info!("Processing {} tasks", tasks.len());
    
    // Process tasks concurrently (but with limits)
    let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_tasks));
    let mut handles = vec![];
    
    for task in tasks {
        let permit = semaphore.clone().acquire_owned().await?;
        let processor = self.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = permit; // Hold permit for duration
            processor.process_single_task(task).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        if let Err(e) = handle.await? {
            error!("Task processing error: {}", e);
        }
    }
    
    Ok(())
}
```

### Task Processing Pipeline
```rust
async fn process_single_task(&self, task: Task) -> Result<()> {
    let task_id = task.id;
    
    // 1. Mark as running
    self.mark_task_running(task_id).await?;
    
    // 2. Find handler for task type
    let handler = self.get_handler(&task.task_type)?;
    
    // 3. Execute with timeout and circuit breaker protection
    let result = tokio::time::timeout(
        self.config.task_timeout,
        self.execute_with_circuit_breaker(&task, handler)
    ).await;
    
    // 4. Handle the result
    match result {
        Ok(Ok(task_result)) => {
            self.mark_task_completed(task_id, task_result).await?;
            info!("Task {} completed successfully", task_id);
        }
        Ok(Err(task_error)) => {
            self.handle_task_failure(task, task_error).await?;
        }
        Err(_timeout) => {
            let error = TaskError::Timeout(self.config.task_timeout);
            self.handle_task_failure(task, error).await?;
        }
    }
    
    Ok(())
}
```

### Circuit Breaker Integration
```rust
async fn execute_with_circuit_breaker(
    &self,
    task: &Task,
    handler: &dyn TaskHandler,
) -> Result<TaskResult, TaskError> {
    // Get or create circuit breaker for this task type
    let mut breakers = self.circuit_breakers.write().await;
    let breaker = breakers.entry(task.task_type.clone())
        .or_insert_with(|| CircuitBreaker::new(5, 3, Duration::from_secs(60)));
    
    // Execute through circuit breaker
    match breaker.call(|| async {
        let context = TaskContext::from_task(task);
        handler.handle(context).await
    }).await {
        Ok(result) => Ok(result),
        Err(CircuitBreakerError::Open) => {
            Err(TaskError::CircuitBreaker("Circuit breaker is open".to_string()))
        }
        Err(CircuitBreakerError::Inner(error)) => Err(error),
    }
}
```

## Task Handlers

### Handler Trait
```rust
#[async_trait]
pub trait TaskHandler: Send + Sync {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError>;
}

pub struct TaskContext {
    pub task_id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub attempt: i32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

pub struct TaskResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### Example Handler: Email Tasks
```rust
pub struct EmailTaskHandler;

#[async_trait]
impl TaskHandler for EmailTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Parse the email payload
        let email_data: EmailPayload = serde_json::from_value(context.payload)
            .map_err(|e| TaskError::InvalidPayload(format!("Email payload: {}", e)))?;
        
        // Validate required fields
        if email_data.to.is_empty() {
            return Err(TaskError::InvalidPayload("Missing 'to' field".to_string()));
        }
        
        if email_data.subject.is_empty() {
            return Err(TaskError::InvalidPayload("Missing 'subject' field".to_string()));
        }
        
        // Simulate email sending (replace with real email service)
        info!("Sending email to: {}", email_data.to);
        info!("Subject: {}", email_data.subject);
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Simulate occasional failures for testing
        if email_data.body.contains("fail") {
            return Err(TaskError::ExternalService("Email service error".to_string()));
        }
        
        // Return success with metadata
        let mut metadata = HashMap::new();
        metadata.insert("recipient".to_string(), json!(email_data.to));
        metadata.insert("sent_at".to_string(), json!(Utc::now()));
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({"status": "sent"})),
            error: None,
            metadata,
        })
    }
}

#[derive(Deserialize)]
struct EmailPayload {
    to: String,
    subject: String,
    body: String,
}
```

## Retry Logic Implementation

### Retry Strategy Configuration
```rust
// Stored in task.retry_strategy JSONB field
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RetryStrategy {
    Exponential {
        base_delay_ms: u64,
        multiplier: f64,
        max_delay_ms: u64,
        max_attempts: u32,
    },
    Linear {
        base_delay_ms: u64,
        increment_ms: u64,
        max_delay_ms: u64,
        max_attempts: u32,
    },
    Fixed {
        interval_ms: u64,
        max_attempts: u32,
    },
}

impl RetryStrategy {
    pub fn calculate_next_retry(&self, attempt: u32) -> Option<DateTime<Utc>> {
        let delay_ms = match self {
            RetryStrategy::Exponential { base_delay_ms, multiplier, max_delay_ms, max_attempts } => {
                if attempt >= *max_attempts {
                    return None;
                }
                let delay = (*base_delay_ms as f64) * multiplier.powi(attempt as i32);
                (delay as u64).min(*max_delay_ms)
            }
            RetryStrategy::Linear { base_delay_ms, increment_ms, max_delay_ms, max_attempts } => {
                if attempt >= *max_attempts {
                    return None;
                }
                let delay = base_delay_ms + (increment_ms * attempt as u64);
                delay.min(*max_delay_ms)
            }
            RetryStrategy::Fixed { interval_ms, max_attempts } => {
                if attempt >= *max_attempts {
                    return None;
                }
                *interval_ms
            }
        };
        
        Some(Utc::now() + Duration::milliseconds(delay_ms as i64))
    }
}
```

### Failure Handling
```rust
async fn handle_task_failure(&self, task: Task, error: TaskError) -> Result<()> {
    let new_attempt = task.current_attempt + 1;
    
    // Parse retry strategy or use default
    let retry_strategy = task.retry_strategy
        .as_ref()
        .and_then(|json| serde_json::from_value(json.clone()).ok())
        .unwrap_or_else(|| RetryStrategy::Exponential {
            base_delay_ms: 1000,
            multiplier: 2.0,
            max_delay_ms: 300000,
            max_attempts: 3,
        });
    
    if let Some(retry_at) = retry_strategy.calculate_next_retry(new_attempt) {
        // Schedule for retry
        self.schedule_task_retry(task.id, retry_at, new_attempt, &error).await?;
        info!("Task {} scheduled for retry {} at {}", task.id, new_attempt, retry_at);
    } else {
        // Max attempts reached - move to dead letter
        self.mark_task_failed(task.id, &error).await?;
        warn!("Task {} moved to dead letter queue after {} attempts", task.id, new_attempt);
    }
    
    Ok(())
}
```

## Built-in Task Types

### 1. Email Tasks
```json
{
  "task_type": "email",
  "payload": {
    "to": "user@example.com",
    "subject": "Welcome!",
    "body": "Thanks for signing up"
  }
}
```

### 2. Data Processing Tasks
```json
{
  "task_type": "data_processing",
  "payload": {
    "operation": "sum",
    "data": [1, 2, 3, 4, 5]
  }
}
```

### 3. Webhook Tasks
```json
{
  "task_type": "webhook", 
  "payload": {
    "url": "https://api.example.com/notify",
    "method": "POST",
    "payload": {"event": "user_created"}
  }
}
```

### 4. File Cleanup Tasks
```json
{
  "task_type": "file_cleanup",
  "payload": {
    "file_path": "/tmp/uploads",
    "max_age_hours": 24
  }
}
```

### 5. Report Generation Tasks
```json
{
  "task_type": "report_generation",
  "payload": {
    "report_type": "sales",
    "start_date": "2024-01-01",
    "end_date": "2024-01-31"
  }
}
```

## API Integration

### Creating Tasks via HTTP
```bash
# Create an email task
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {
      "to": "user@example.com",
      "subject": "Hello",
      "body": "This is a test email"
    },
    "priority": "normal"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "task_type": "email",
    "status": "pending",
    "priority": "normal",
    "created_at": "2024-01-01T12:00:00Z"
  }
}
```

### Monitoring Tasks
```bash
# Get task statistics
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tasks/stats

# List your tasks
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks?limit=10"

# Get specific task details
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks/uuid-here"
```

## Development Workflow

### Starting the System
```bash
# 1. Start infrastructure
./scripts/dev.sh

# 2. Start HTTP server (handles API requests)
./scripts/server.sh 3000

# 3. Start worker (processes background tasks)
./scripts/worker.sh

# 4. Monitor logs
tail -f /tmp/starter-server-3000.log
tail -f /tmp/starter-worker.log
```

### Testing End-to-End
```bash
# Run task system tests (11 comprehensive tests)
cargo nextest run tasks::

# Run all tests including background tasks
cargo nextest run
```

The task integration tests cover:
1. Task creation via API
2. Different task types and priorities
3. Background worker processing
4. Task status tracking and updates
5. Statistics and monitoring
6. Error handling and retry logic

### Manual Testing
```bash
# Create test user and get token
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"password123"}'

TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"test","password":"password123"}' \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])")

# Create tasks
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"email","payload":{"to":"test@example.com","subject":"Test","body":"Hello"}}'

# Monitor processing
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/tasks/stats
```

## Configuration

### Worker Settings
```bash
# Environment variables
STARTER__WORKER__CONCURRENCY=4                    # Parallel tasks
STARTER__WORKER__POLL_INTERVAL_SECS=5             # Check frequency
STARTER__WORKER__MAX_RETRIES=3                    # Default max attempts
STARTER__WORKER__RETRY_BACKOFF_BASE_SECS=2        # Default base delay
```

### Task Defaults
```rust
// Default retry strategy for new tasks
impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::Exponential {
            base_delay_ms: 1000,    // Start with 1 second
            multiplier: 2.0,        // Double each time
            max_delay_ms: 300000,   // Cap at 5 minutes
            max_attempts: 3,        // Try 3 times total
        }
    }
}
```

## Troubleshooting

### Common Issues

**Tasks stay in pending status**
```bash
# Check if worker is running
./scripts/status.sh

# Check worker logs
tail -f /tmp/starter-worker.log

# Restart worker
./scripts/stop-worker.sh
./scripts/worker.sh
```

**High failure rate**
```bash
# Check failed tasks
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks?status=failed&limit=5"

# Look for patterns in errors
grep "ERROR" /tmp/starter-worker.log
```

**Performance issues**
```bash
# Check task queue depth
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tasks/stats

# Adjust worker concurrency
STARTER__WORKER__CONCURRENCY=8 ./scripts/worker.sh
```

## Next Steps

Now that you understand how the background task system works, learn how to extend it:

- **[Custom Task Types →](./05-task-types.md)** - Create your own task handlers
- **[Task Registry →](./06-task-registry.md)** - Organize and manage task handlers

---
## Next Steps

Now that you understand the background task system, explore related concepts:

- **[Testing Guide →](./07-testing.md)** - Learn how to test your task handlers with the comprehensive testing framework
- **[Reliability Patterns →](../reliability.md)** - Understand the circuit breakers and retry strategies used by the task system
- **[Custom Task Types →](./05-task-types.md)** - Build your own task handlers for specific use cases

## Testing Your Tasks

The starter includes comprehensive integration tests for the task system. See how to test:

```bash
# Run task-related tests
cargo nextest run tasks::

# Test task creation
cargo nextest run test_create_task

# Test task authentication
cargo nextest run test_task_retry_mechanism
```

Example task test pattern:
```rust
#[tokio::test]
async fn test_my_custom_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create authenticated user (tasks require auth)
    let (_user, token) = factory.create_authenticated_user("testuser").await;
    
    // Create task
    let task_response = factory.create_task("my_task_type", json!({
        "data": "test_payload"
    })).await;
    
    // Verify task was created correctly
    assert_eq!(task_response["data"]["task_type"], "my_task_type");
    assert_eq!(task_response["data"]["status"], "Pending");
}
```

---

*This background task system demonstrates how to build reliable, scalable async processing using database queues and the reliability patterns you learned earlier.*