# Custom Task Types

*This guide shows you how to create your own background task types, from simple examples to advanced patterns.*

## Understanding Task Types

A **task type** is simply a string identifier that maps to a specific handler. When you create a task with `task_type: "my_custom_task"`, the worker system looks up the corresponding handler and executes it.

```rust
// Task creation
Task {
    task_type: "my_custom_task",  // ← This string maps to a handler
    payload: json!({
        "user_id": 123,
        "action": "send_welcome_email"
    }),
    // ... other fields
}

// Handler registration
processor.register_handler("my_custom_task", MyCustomHandler);
```

## Creating Your First Custom Task Type

### Step 1: Define Your Handler

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::tasks::{TaskHandler, TaskContext, TaskResult, TaskError};

pub struct WelcomeEmailHandler;

#[async_trait]
impl TaskHandler for WelcomeEmailHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Parse the payload into a structured type
        let payload: WelcomeEmailPayload = serde_json::from_value(context.payload)
            .map_err(|e| TaskError::InvalidPayload(format!("Welcome email: {}", e)))?;
        
        // Validate required fields
        if payload.user_id == 0 {
            return Err(TaskError::InvalidPayload("user_id is required".to_string()));
        }
        
        // Perform the actual work
        info!("Sending welcome email for user {}", payload.user_id);
        
        // Simulate fetching user data
        let user_email = format!("user{}@example.com", payload.user_id);
        let username = format!("User{}", payload.user_id);
        
        // Simulate email sending
        info!("To: {}", user_email);
        info!("Subject: Welcome, {}!", username);
        info!("Body: Welcome to our platform! Your account is ready.");
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Return success with useful metadata
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), json!(payload.user_id));
        metadata.insert("recipient".to_string(), json!(user_email));
        metadata.insert("sent_at".to_string(), json!(Utc::now()));
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({
                "email_sent": true,
                "recipient": user_email
            })),
            error: None,
            metadata,
        })
    }
}

#[derive(Deserialize)]
struct WelcomeEmailPayload {
    user_id: u64,
    template: Option<String>,  // Optional custom template
}
```

### Step 2: Register Your Handler

In your worker startup code:

```rust
// In main.rs or wherever you start the worker
async fn start_worker(database: Database) -> Result<()> {
    let mut processor = TaskProcessor::new(database, ProcessorConfig::default());
    
    // Register built-in handlers
    processor.register_handler("email".to_string(), EmailTaskHandler).await;
    processor.register_handler("webhook".to_string(), WebhookTaskHandler).await;
    
    // Register your custom handler
    processor.register_handler("welcome_email".to_string(), WelcomeEmailHandler).await;
    
    // Start processing
    processor.start_worker().await
}
```

### Step 3: Use Your Task Type

```bash
# Create a welcome email task
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "welcome_email",
    "payload": {
      "user_id": 123,
      "template": "premium_welcome"
    },
    "priority": "high"
  }'
```

## Design Patterns for Task Types

### Pattern 1: Simple Action Tasks

**Use Case**: Single, atomic operations
```rust
// User account cleanup
pub struct AccountCleanupHandler;

impl TaskHandler for AccountCleanupHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: AccountCleanupPayload = serde_json::from_value(context.payload)?;
        
        // Delete user sessions
        self.cleanup_sessions(payload.user_id).await?;
        
        // Remove temporary files
        self.cleanup_files(payload.user_id).await?;
        
        // Send notification
        self.notify_completion(payload.user_id).await?;
        
        Ok(TaskResult::success_simple())
    }
}

// Usage
{
  "task_type": "account_cleanup",
  "payload": {
    "user_id": 123,
    "reason": "account_deletion"
  }
}
```

### Pattern 2: Data Processing Tasks

**Use Case**: Transform or analyze data
```rust
pub struct UserReportHandler;

impl TaskHandler for UserReportHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: UserReportPayload = serde_json::from_value(context.payload)?;
        
        // Fetch data
        let users = self.fetch_users_in_date_range(
            payload.start_date, 
            payload.end_date
        ).await?;
        
        // Process data
        let report = UserReport {
            total_users: users.len(),
            active_users: users.iter().filter(|u| u.is_active).count(),
            new_signups: users.iter().filter(|u| u.created_recently()).count(),
            // ... more analytics
        };
        
        // Store or send report
        let report_url = self.save_report(&report, &payload.format).await?;
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({
                "report_url": report_url,
                "total_users": report.total_users
            })),
            error: None,
            metadata: HashMap::new(),
        })
    }
}

// Usage  
{
  "task_type": "user_report",
  "payload": {
    "start_date": "2024-01-01",
    "end_date": "2024-01-31", 
    "format": "pdf",
    "include_details": true
  }
}
```

### Pattern 3: Multi-Step Workflow Tasks

**Use Case**: Complex operations with multiple stages
```rust
pub struct OrderFulfillmentHandler;

impl TaskHandler for OrderFulfillmentHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: OrderPayload = serde_json::from_value(context.payload)?;
        let mut workflow_state = WorkflowState::new();
        
        // Step 1: Validate inventory
        match self.check_inventory(payload.order_id).await {
            Ok(available) if available => {
                workflow_state.inventory_checked = true;
            }
            Ok(_) => {
                return Err(TaskError::BusinessLogic("Insufficient inventory".to_string()));
            }
            Err(e) => return Err(TaskError::ExternalService(format!("Inventory check failed: {}", e))),
        }
        
        // Step 2: Process payment
        match self.process_payment(payload.order_id).await {
            Ok(payment_id) => {
                workflow_state.payment_processed = true;
                workflow_state.payment_id = Some(payment_id);
            }
            Err(e) => {
                // Rollback inventory hold
                self.release_inventory_hold(payload.order_id).await?;
                return Err(TaskError::ExternalService(format!("Payment failed: {}", e)));
            }
        }
        
        // Step 3: Create shipment
        let tracking_number = self.create_shipment(payload.order_id).await?;
        workflow_state.shipment_created = true;
        
        // Step 4: Send notifications
        self.notify_customer(payload.order_id, &tracking_number).await?;
        workflow_state.customer_notified = true;
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({
                "order_id": payload.order_id,
                "tracking_number": tracking_number,
                "workflow_state": workflow_state
            })),
            error: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("steps_completed".to_string(), json!(4));
                meta.insert("payment_id".to_string(), json!(workflow_state.payment_id));
                meta
            },
        })
    }
}

#[derive(Serialize)]
struct WorkflowState {
    inventory_checked: bool,
    payment_processed: bool,
    shipment_created: bool,
    customer_notified: bool,
    payment_id: Option<String>,
}
```

### Pattern 4: Scheduled/Recurring Tasks

**Use Case**: Tasks that run on a schedule
```rust
pub struct DailyBackupHandler;

impl TaskHandler for DailyBackupHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: BackupPayload = serde_json::from_value(context.payload)?;
        
        info!("Starting daily backup for: {}", payload.backup_type);
        
        // Perform backup
        let backup_result = match payload.backup_type.as_str() {
            "database" => self.backup_database().await?,
            "files" => self.backup_files(&payload.paths).await?,
            "logs" => self.backup_logs(&payload.log_retention).await?,
            _ => return Err(TaskError::InvalidPayload("Unknown backup type".to_string())),
        };
        
        // Schedule next backup (24 hours from now)
        if payload.recurring {
            self.schedule_next_backup(&payload).await?;
        }
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({
                "backup_size": backup_result.size_bytes,
                "backup_location": backup_result.location,
                "next_backup_scheduled": payload.recurring
            })),
            error: None,
            metadata: HashMap::new(),
        })
    }
}

// Create recurring task
{
  "task_type": "daily_backup",
  "payload": {
    "backup_type": "database",
    "recurring": true,
    "retention_days": 30
  },
  "scheduled_at": "2024-01-01T02:00:00Z"  // 2 AM
}
```

## Advanced Task Type Features

### Custom Retry Strategies

```rust
impl TaskHandler for CriticalTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Your task logic here
        
        // If this is a retry, use different logic
        if context.attempt > 0 {
            info!("Retry attempt {} for task {}", context.attempt, context.task_id);
            
            // Maybe use a different API endpoint for retries
            return self.handle_retry_logic(context).await;
        }
        
        self.handle_normal_logic(context).await
    }
}

// When creating the task, specify custom retry strategy
{
  "task_type": "critical_task",
  "payload": { /* your data */ },
  "retry_strategy": {
    "type": "exponential",
    "base_delay_ms": 500,
    "multiplier": 1.5,
    "max_delay_ms": 30000,
    "max_attempts": 10
  }
}
```

### Task Dependencies

```rust
pub struct DependentTaskHandler;

impl TaskHandler for DependentTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: DependentPayload = serde_json::from_value(context.payload)?;
        
        // Check if prerequisite tasks are completed
        for prereq_id in &payload.prerequisite_task_ids {
            let prereq_status = self.get_task_status(*prereq_id).await?;
            
            if prereq_status != TaskStatus::Completed {
                // Reschedule this task for later
                return Err(TaskError::Retry(
                    "Waiting for prerequisite tasks".to_string(),
                    Duration::from_secs(60) // Check again in 1 minute
                ));
            }
        }
        
        // All prerequisites complete, proceed with work
        self.execute_dependent_work(payload).await
    }
}
```

### Error Recovery and Compensation

```rust
pub struct TransactionTaskHandler;

impl TaskHandler for TransactionTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: TransactionPayload = serde_json::from_value(context.payload)?;
        
        // Try to execute the transaction
        match self.execute_transaction(payload.clone()).await {
            Ok(result) => Ok(result),
            Err(TaskError::BusinessLogic(msg)) if msg.contains("duplicate") => {
                // Idempotency check - maybe this already succeeded
                if let Ok(existing) = self.find_existing_transaction(&payload.transaction_id).await {
                    info!("Transaction {} already exists, returning existing result", payload.transaction_id);
                    return Ok(TaskResult {
                        success: true,
                        output: Some(json!(existing)),
                        error: None,
                        metadata: HashMap::new(),
                    });
                }
                Err(TaskError::BusinessLogic(msg))
            }
            Err(e) => {
                // On failure, attempt compensation
                if context.attempt > 0 {
                    self.compensate_transaction(&payload.transaction_id).await?;
                }
                Err(e)
            }
        }
    }
}
```

## Testing Custom Task Types

### Unit Testing Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_welcome_email_handler() {
        let handler = WelcomeEmailHandler;
        
        let context = TaskContext {
            task_id: Uuid::new_v4(),
            task_type: "welcome_email".to_string(),
            payload: json!({
                "user_id": 123,
                "template": "premium"
            }),
            attempt: 0,
            metadata: HashMap::new(),
            created_by: Some(Uuid::new_v4()),
            created_at: Utc::now(),
        };
        
        let result = handler.handle(context).await.unwrap();
        
        assert!(result.success);
        assert!(result.output.is_some());
        assert_eq!(result.metadata.get("user_id"), Some(&json!(123)));
    }
    
    #[tokio::test]
    async fn test_welcome_email_invalid_payload() {
        let handler = WelcomeEmailHandler;
        
        let context = TaskContext {
            task_id: Uuid::new_v4(),
            task_type: "welcome_email".to_string(),
            payload: json!({
                "user_id": 0  // Invalid user ID
            }),
            attempt: 0,
            metadata: HashMap::new(),
            created_by: None,
            created_at: Utc::now(),
        };
        
        let result = handler.handle(context).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::InvalidPayload(_)));
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_welcome_email_end_to_end() {
    // Setup test database and processor
    let database = setup_test_database().await;
    let mut processor = TaskProcessor::new(database.clone(), ProcessorConfig::default());
    processor.register_handler("welcome_email".to_string(), WelcomeEmailHandler).await;
    
    // Create a task
    let task_id = create_task(
        &database,
        "welcome_email",
        json!({"user_id": 123}),
        None, // created_by
    ).await.unwrap();
    
    // Process the task
    processor.process_single_task_by_id(task_id).await.unwrap();
    
    // Verify results
    let task = get_task(&database, task_id).await.unwrap();
    assert_eq!(task.status, TaskStatus::Completed);
    assert!(task.completed_at.is_some());
}
```

## Task Type Organization

### Grouping by Domain

```rust
// User-related tasks
mod user_tasks {
    pub struct WelcomeEmailHandler;
    pub struct AccountCleanupHandler;
    pub struct PasswordResetHandler;
}

// Billing-related tasks
mod billing_tasks {
    pub struct InvoiceGenerationHandler;
    pub struct PaymentProcessingHandler;
    pub struct SubscriptionRenewalHandler;
}

// System maintenance tasks
mod system_tasks {
    pub struct BackupHandler;
    pub struct LogCleanupHandler;
    pub struct HealthCheckHandler;
}
```

### Registry Pattern

```rust
pub struct TaskRegistry {
    handlers: HashMap<String, Box<dyn TaskHandler>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        
        // Register all handlers
        registry.register_user_tasks();
        registry.register_billing_tasks();
        registry.register_system_tasks();
        
        registry
    }
    
    fn register_user_tasks(&mut self) {
        self.register("welcome_email", Box::new(WelcomeEmailHandler));
        self.register("account_cleanup", Box::new(AccountCleanupHandler));
        self.register("password_reset", Box::new(PasswordResetHandler));
    }
    
    fn register_billing_tasks(&mut self) {
        self.register("invoice_generation", Box::new(InvoiceGenerationHandler));
        self.register("payment_processing", Box::new(PaymentProcessingHandler));
        self.register("subscription_renewal", Box::new(SubscriptionRenewalHandler));
    }
    
    pub fn register(&mut self, task_type: &str, handler: Box<dyn TaskHandler>) {
        self.handlers.insert(task_type.to_string(), handler);
    }
    
    pub fn get_handler(&self, task_type: &str) -> Option<&dyn TaskHandler> {
        self.handlers.get(task_type).map(|h| h.as_ref())
    }
}
```

## Best Practices

### 1. Payload Design
```rust
// Good: Structured, validated payloads
#[derive(Deserialize)]
struct EmailPayload {
    to: String,
    subject: String,
    body: String,
    template_id: Option<String>,
    variables: Option<HashMap<String, String>>,
}

// Avoid: Unstructured JSON blobs
// payload: { "data": "user@example.com,Welcome,Hello there" }
```

### 2. Error Handling
```rust
impl TaskHandler for MyHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Be specific about error types
        match external_service_call().await {
            Ok(result) => Ok(result),
            Err(ApiError::RateLimit) => {
                // Temporary error - retry with longer delay
                Err(TaskError::RateLimited(Duration::from_secs(300)))
            }
            Err(ApiError::InvalidData) => {
                // Permanent error - don't retry
                Err(TaskError::InvalidPayload("Bad data from API".to_string()))
            }
            Err(ApiError::NetworkError) => {
                // Temporary error - normal retry
                Err(TaskError::ExternalService("Network issue".to_string()))
            }
        }
    }
}
```

### 3. Idempotency
```rust
impl TaskHandler for IdempotentHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let payload: MyPayload = serde_json::from_value(context.payload)?;
        
        // Check if work already completed
        if let Ok(existing_result) = self.check_existing_work(&payload.operation_id).await {
            info!("Operation {} already completed", payload.operation_id);
            return Ok(existing_result);
        }
        
        // Perform work with unique operation ID
        let result = self.perform_work_with_id(&payload).await?;
        
        // Store result for future idempotency checks
        self.store_operation_result(&payload.operation_id, &result).await?;
        
        Ok(result)
    }
}
```

### 4. Resource Management
```rust
impl TaskHandler for ResourceIntensiveHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Use bounded channels for memory control
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        // Process data in chunks
        let payload: LargeDataPayload = serde_json::from_value(context.payload)?;
        
        for chunk in payload.data.chunks(1000) {
            let processed = self.process_chunk(chunk).await?;
            
            if tx.send(processed).await.is_err() {
                break; // Receiver dropped
            }
        }
        
        drop(tx); // Signal completion
        
        // Collect results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }
        
        Ok(TaskResult {
            success: true,
            output: Some(json!({"processed_count": results.len()})),
            error: None,
            metadata: HashMap::new(),
        })
    }
}
```

## Next Steps

Now that you can create custom task types, learn how to organize them:

- **[Task Registry →](./06-task-registry.md)** - Organize and manage task handlers
- **[Built-in Handlers Reference →](../reference/task-handlers.md)** - Learn from existing examples

---
*Custom task types let you extend the background task system for your specific needs while maintaining the same reliability and monitoring features.*