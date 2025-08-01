# Built-in Task Handlers Reference

*Complete reference for all included task handlers with payload schemas, configuration options, and usage examples.*

> **⚠️ Important**: All tasks are processed **asynchronously**. The API immediately returns task metadata (ID, status, timestamps) and processing happens later in the background worker. Task results are not returned in the API response.

## 📋 How to Monitor Task Results

Since tasks process asynchronously, use these methods to check outcomes:

### 0. Get Authentication Token
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "password123"}'

export TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email": "testuser", "password": "password123"}' \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])")

echo "Token: $TOKEN"
```

### 1. Check Task Status
```bash
# Get specific task by ID
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/tasks/TASK_ID_HERE"

# List recent tasks with status filter
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/tasks?status=completed&limit=10"
```

### 2. Monitor Worker Logs
```bash
# Follow worker logs to see processing details
tail -f /tmp/starter-worker.log

# Search for specific task processing
grep "Task TASK_ID_HERE" /tmp/starter-worker.log
```

### 3. Use Admin CLI (Bypasses API Auth)
```bash
# Check task statistics
cargo run -- admin task-stats

# List recent tasks with details
cargo run -- admin list-tasks --limit 5 --verbose
```

## Overview

The starter includes 6 built-in task handlers that demonstrate common background task patterns:

| Handler | Task Type | Purpose | Complexity | Status |
|---------|-----------|---------|------------|---------|
| EmailTaskHandler | `email` | Send email notifications | Simple | 🟡 Logs Only |
| DataProcessingTaskHandler | `data_processing` | Process data arrays | Simple | ✅ Working |
| WebhookTaskHandler | `webhook` | Send HTTP requests | Medium | ✅ Working |
| FileCleanupTaskHandler | `file_cleanup` | Clean up old files | Medium | 🟡 Simulated |
| ReportGenerationTaskHandler | `report_generation` | Generate reports | Complex | 🟡 Simulated |
| DelayTaskHandler | `delay_task` | Chaos testing delays | Simple | ✅ Working |

## EmailTaskHandler

### Purpose
Sends email notifications. Currently logs email details (replace with real email service).

### Task Type
`email`

### Payload Schema
```json
{
  "to": "recipient@example.com",        // Required: recipient email
  "subject": "Email Subject",           // Required: email subject
  "body": "Email content here",         // Required: email body
  "cc": ["cc1@example.com"],           // Optional: CC recipients
  "bcc": ["bcc1@example.com"],         // Optional: BCC recipients
  "template_id": "welcome_template",    // Optional: email template
  "variables": {                        // Optional: template variables
    "user_name": "John",
    "signup_date": "2024-01-01"
  }
}
```

### Configuration
No special configuration required. Replace logging with actual email service integration.

### Usage Examples

**Basic Email:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {
      "to": "user@example.com",
      "subject": "Welcome!",
      "body": "Thanks for signing up."
    }
  }'
```

**Email with CC/BCC:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {
      "to": "user@example.com",
      "subject": "Team Update",
      "body": "Weekly team meeting notes...",
      "cc": ["manager@example.com"],
      "bcc": ["archive@example.com"]
    }
  }'
```

### Error Conditions
- **Missing required fields**: Returns `InvalidPayload` error
- **Body contains "fail"**: Simulated failure for testing
- **Empty recipient**: Returns `InvalidPayload` error

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Sending email to: user@example.com, subject: Welcome!
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

### Implementation Details
```rust
// Simulates email sending with 500ms delay
// Fails if body contains "fail" (for testing)
// Returns metadata with recipient and sent timestamp
```

---

## DataProcessingTaskHandler

### Purpose
Processes arrays of data with various mathematical operations.

### Task Type
`data_processing`

### Payload Schema
```json
{
  "operation": "sum|count|average|max|min",  // Required: operation to perform
  "data": [1, 2, 3, 4, 5],                 // Required: array of numbers
  "options": {                              // Optional: processing options
    "precision": 2,                         // Decimal places for results
    "format": "json|csv"                    // Output format
  }
}
```

### Supported Operations
- **`sum`**: Calculate sum of all numbers
- **`count`**: Count array elements
- **`average`**: Calculate average (future enhancement)
- **`max`**: Find maximum value (future enhancement)
- **`min`**: Find minimum value (future enhancement)

### Usage Examples

**Sum Operation:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "data_processing",
    "payload": {
      "operation": "sum",
      "data": [10, 20, 30, 40, 50]
    }
  }'
```

**Count Operation:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "data_processing",
    "payload": {
      "operation": "count",
      "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    }
  }'
```

### Error Conditions
- **Unknown operation**: Returns `InvalidPayload` error
- **Empty data array**: Returns `InvalidPayload` error
- **Non-numeric data**: Returns `InvalidPayload` error

### API Response
The API returns standard task metadata immediately:
```json
{
  "id": "task-uuid-here",
  "task_type": "data_processing",
  "status": "Pending",
  "priority": "Normal",
  "created_at": "2024-01-01T12:00:00Z",
  "scheduled_at": "2024-01-01T12:00:00Z"
}
```

Processing happens asynchronously in the background worker.

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Processing data with operation: sum  
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

---

## WebhookTaskHandler

### Purpose
Sends HTTP requests to external services for notifications or integrations.

### Task Type
`webhook`

### Payload Schema
```json
{
  "url": "https://api.example.com/webhook",     // Required: target URL
  "method": "POST|GET|PUT|DELETE",              // Required: HTTP method
  "payload": {                                  // Optional: request body
    "event": "user_created",
    "data": {"user_id": 123}
  },
  "headers": {                                  // Optional: custom headers
    "Authorization": "Bearer token",
    "Content-Type": "application/json",
    "X-Source": "starter-app"
  },
  "timeout_seconds": 30,                        // Optional: request timeout
  "retry_on_failure": true                      // Optional: enable retries
}
```

### Supported Methods
- **GET**: Query external services
- **POST**: Send data to external services
- **PUT**: Update external resources
- **DELETE**: Remove external resources

### Usage Examples

**Simple POST Webhook:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "webhook",
    "payload": {
      "url": "https://httpbin.org/post",
      "method": "POST",
      "payload": {
        "event": "task_completed",
        "timestamp": "2024-01-01T12:00:00Z"
      }
    }
  }'
```

**Webhook with Custom Headers:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "webhook",
    "payload": {
      "url": "https://api.service.com/notify",
      "method": "POST",
      "headers": {
        "Authorization": "Bearer secret-token",
        "X-Event-Source": "starter-app"
      },
      "payload": {
        "user_id": 123,
        "action": "account_created"
      },
      "timeout_seconds": 15
    }
  }'
```

### Error Conditions
- **Invalid URL**: Returns `InvalidPayload` error
- **Unsupported method**: Returns `InvalidPayload` error
- **URL contains "fail"**: Simulated failure for testing
- **Network timeout**: Returns `ExternalService` error

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Sending webhook POST to: https://httpbin.org/post
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

### Implementation Details
```rust
// Currently simulates HTTP requests (replace with actual HTTP client)
// Returns simulated response status and body
// Supports configurable timeout
```

---

## FileCleanupTaskHandler

### Purpose
Cleans up old files in specified directories based on age criteria.

### Task Type
`file_cleanup`

### Payload Schema
```json
{
  "file_path": "/tmp/uploads",              // Required: directory to clean
  "max_age_hours": 24,                      // Required: maximum file age
  "pattern": "*.tmp",                       // Optional: file pattern to match
  "recursive": true,                        // Optional: search subdirectories
  "dry_run": false,                         // Optional: simulate without deleting
  "exclude_patterns": [                     // Optional: patterns to exclude
    "*.keep",
    "important_*"
  ]
}
```

### Usage Examples

**Basic Directory Cleanup:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "file_cleanup",
    "payload": {
      "file_path": "/tmp/uploads",
      "max_age_hours": 48
    }
  }'
```

**Pattern-Based Cleanup:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "file_cleanup",
    "payload": {
      "file_path": "/var/log/app",
      "max_age_hours": 168,
      "pattern": "*.log",
      "exclude_patterns": ["error.log", "access.log"]
    }
  }'
```

**Dry Run (Test Mode):**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "file_cleanup",
    "payload": {
      "file_path": "/tmp/cache",
      "max_age_hours": 24,
      "dry_run": true
    }
  }'
```

### Error Conditions
- **Invalid path**: Returns `InvalidPayload` error
- **Path doesn't exist**: Returns `ExternalService` error
- **Permission denied**: Returns `ExternalService` error
- **Zero max_age_hours**: Returns `InvalidPayload` error

### API Response
The API returns standard task metadata immediately:
```json
{
  "id": "task-uuid-here",
  "task_type": "file_cleanup",
  "status": "Pending",
  "priority": "Normal",
  "created_at": "2024-01-01T12:00:00Z",
  "scheduled_at": "2024-01-01T12:00:00Z"
}
```

File cleanup happens asynchronously in the background worker.

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Cleaning up files in path: /tmp/uploads, max age: 24 hours
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

### Implementation Details
```rust
// Currently simulates file operations (replace with actual filesystem calls)
// Returns simulated file counts and bytes freed
// Supports dry run mode for testing
```

---

## ReportGenerationTaskHandler

### Purpose
Generates various types of reports from application data.

### Task Type
`report_generation`

### Payload Schema
```json
{
  "report_type": "sales|users|activity|custom",   // Required: type of report
  "start_date": "2024-01-01",                     // Required: report start date
  "end_date": "2024-01-31",                       // Required: report end date
  "format": "pdf|csv|json|excel",                 // Optional: output format
  "filters": {                                    // Optional: report filters
    "department": "sales",
    "region": "US",
    "product_category": "software"
  },
  "options": {                                    // Optional: report options
    "include_charts": true,
    "include_summary": true,
    "group_by": "month"
  },
  "delivery": {                                   // Optional: how to deliver
    "method": "email|download|storage",
    "recipients": ["manager@example.com"],
    "storage_path": "/reports/monthly"
  }
}
```

### Supported Report Types
- **`sales`**: Sales performance and revenue data
- **`users`**: User registration and activity analytics
- **`activity`**: System usage and engagement metrics
- **`custom`**: Custom reports based on filters

### Supported Formats
- **`pdf`**: Formatted PDF document
- **`csv`**: Comma-separated values
- **`json`**: Structured JSON data
- **`excel`**: Microsoft Excel spreadsheet

### Usage Examples

**Monthly Sales Report:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "report_generation",
    "payload": {
      "report_type": "sales",
      "start_date": "2024-01-01",
      "end_date": "2024-01-31",
      "format": "pdf",
      "filters": {
        "region": "US"
      },
      "options": {
        "include_charts": true,
        "group_by": "week"
      }
    }
  }'
```

**User Activity CSV:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "report_generation",
    "payload": {
      "report_type": "users",
      "start_date": "2024-01-01",
      "end_date": "2024-01-31",
      "format": "csv",
      "delivery": {
        "method": "email",
        "recipients": ["analytics@example.com"]
      }
    }
  }'
```

**Custom Report with Filters:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "report_generation",
    "payload": {
      "report_type": "custom",
      "start_date": "2024-01-01",
      "end_date": "2024-01-31",
      "format": "json",
      "filters": {
        "department": "engineering",
        "project": "mobile-app"
      },
      "options": {
        "include_summary": true
      }
    }
  }'
```

### Error Conditions
- **Invalid date range**: Returns `InvalidPayload` error
- **Unknown report type**: Returns `InvalidPayload` error
- **Unsupported format**: Returns `InvalidPayload` error
- **Invalid date format**: Returns `InvalidPayload` error

### API Response
The API returns standard task metadata immediately:
```json
{
  "id": "task-uuid-here",
  "task_type": "report_generation",
  "status": "Pending",
  "priority": "Normal",
  "created_at": "2024-01-01T12:00:00Z",
  "scheduled_at": "2024-01-01T12:00:00Z"
}
```

Report generation happens asynchronously in the background worker.

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Generating sales report from 2024-01-01 to 2024-01-31
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

### Implementation Details
```rust
// Currently simulates report generation (replace with actual report engine)
// Returns simulated report URL and metadata
// Processing time varies by report complexity
```

---

## DelayTaskHandler

### Purpose
Simulates work with configurable delays, primarily used for chaos testing and load testing scenarios.

### Task Type
`delay_task`

### Payload Schema
```json
{
  "delay_seconds": 5,                           // Required: delay duration
  "task_id": "test-task-001",                  // Optional: task identifier
  "test_scenario": "worker-restart",           // Optional: test scenario name
  "deadline": "2024-01-01T12:05:00Z"          // Optional: task deadline (RFC3339)
}
```

### Usage Examples

**Basic Delay Task:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "delay_task",
    "payload": {
      "delay_seconds": 3,
      "task_id": "simple-delay-test"
    }
  }'
```

**Chaos Testing with Deadline:**
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "delay_task",
    "payload": {
      "delay_seconds": 10,
      "task_id": "chaos-test-001",
      "test_scenario": "multi-worker-chaos",
      "deadline": "2024-01-01T12:05:00Z"
    }
  }'
```

### Error Conditions
- **Past deadline**: Returns `Execution` error if current time exceeds deadline
- **Insufficient time**: Returns `Execution` error if remaining time < delay_seconds
- **Invalid deadline format**: Ignores malformed deadline strings

### API Response
The API returns standard task metadata immediately:
```json
{
  "id": "task-uuid-here",
  "task_type": "delay_task",
  "status": "Pending",
  "priority": "Normal",
  "created_at": "2024-01-01T12:00:00Z",
  "scheduled_at": "2024-01-01T12:00:00Z"
}
```

Delay processing happens asynchronously in the background worker.

### Expected Worker Log Output
```
INFO starter::tasks::handlers: Processing delay task: test-delay (scenario: general, delay: 2s, attempt: 0)
INFO starter::tasks::handlers: Task test-delay starting 2s delay work...
INFO starter::tasks::handlers: Task test-delay completed successfully after 2s
INFO starter::tasks::processor: Task TASK_ID completed successfully
```

### Implementation Details
```rust
// Deadline-aware task processing
// Checks deadline before and after work
// Used extensively in chaos testing scenarios
// Supports multi-attempt scenarios with attempt tracking
```

---

## Common Configuration

### Retry Strategies
All handlers support custom retry strategies. Default configuration:

```json
{
  "retry_strategy": {
    "type": "exponential",
    "base_delay_ms": 1000,
    "multiplier": 2.0,
    "max_delay_ms": 300000,
    "max_attempts": 3
  }
}
```

### Priority Levels
All task types support priority levels:
- **`critical`**: Processed immediately
- **`high`**: Processed before normal
- **`normal`**: Default priority
- **`low`**: Processed when queue is empty

### Metadata and Context
All handlers receive:
- **Task ID**: Unique identifier
- **Attempt number**: Current retry attempt
- **Created by**: User who created the task
- **Created at**: Task creation timestamp
- **Custom metadata**: Additional key-value data

## Testing Built-in Handlers

### Integration Test Script
```bash
# Test all built-in handlers (11 task tests included)
cargo nextest run tasks::
```

### Manual Testing
```bash
# Start services (background mode)
./scripts/server.sh 3000
./scripts/worker.sh

# Alternative: Foreground mode (separate terminals)
# Terminal 1: ./scripts/server.sh 3000 -f
# Terminal 2: ./scripts/worker.sh -f

# Register test user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"password123"}'

# Get authentication token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"test","password":"password123"}' \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])")

# Test each handler...
```

### Unit Tests
Each handler includes comprehensive unit tests:
```bash
cargo test tasks::handlers::tests
```

## Extending Built-in Handlers

### Replacing Simulated Operations
The built-in handlers use simulated operations for demonstration. Replace with real implementations:

```rust
// Email handler - replace with actual email service
impl EmailTaskHandler {
    async fn send_actual_email(&self, payload: &EmailPayload) -> Result<()> {
        // Replace with SMTP, SendGrid, AWS SES, etc.
        let email_service = SmtpService::new(&self.config);
        email_service.send_email(payload).await
    }
}

// File cleanup - replace with actual filesystem operations
impl FileCleanupTaskHandler {
    async fn cleanup_files(&self, payload: &FileCleanupPayload) -> Result<CleanupResult> {
        // Replace with actual file system operations
        use tokio::fs;
        
        let mut files_deleted = 0;
        let mut bytes_freed = 0;
        
        // Actual file deletion logic here...
        
        Ok(CleanupResult { files_deleted, bytes_freed })
    }
}
```

### Adding Configuration Options
Extend handlers with configuration support:

```rust
pub struct ConfigurableEmailHandler {
    smtp_config: SmtpConfig,
    template_service: Arc<TemplateService>,
    rate_limiter: Arc<RateLimiter>,
}

impl ConfigurableEmailHandler {
    pub fn new(config: EmailHandlerConfig) -> Self {
        Self {
            smtp_config: config.smtp,
            template_service: Arc::new(TemplateService::new(config.templates)),
            rate_limiter: Arc::new(RateLimiter::new(config.rate_limit)),
        }
    }
}
```

## Complete Workflow Example

Here's a complete example showing how to create a task and monitor its completion:

```bash
# 1. Create a task
RESPONSE=$(curl -s -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "data_processing",
    "payload": {
      "operation": "sum",
      "data": [10, 20, 30, 40, 50]
    }
  }')

# 2. Extract task ID
TASK_ID=$(echo "$RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])")
echo "Created task: $TASK_ID"

# 3. Monitor in real-time (in another terminal)
tail -f /tmp/starter-worker.log | grep "$TASK_ID"

# 4. Check final status
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/tasks/$TASK_ID"

# 5. Alternative: Use admin CLI
cargo run -- admin list-tasks --limit 1
```

## Next Steps

- **[Custom Task Types →](./06-task-types.md)** - Create your own task handlers
- **[Task Registry →](./07-task-registry.md)** - Organize multiple handlers
- **[Troubleshooting →](../troubleshooting.md)** - Debug handler issues

---
*These built-in handlers provide working examples and starting points for your own background task implementations.*