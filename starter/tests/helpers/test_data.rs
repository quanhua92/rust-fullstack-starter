use crate::helpers::test_app::{AuthToken, TestApp};
use serde_json::json;
use starter::models::{CreateUserRequest, User};

pub struct TestDataFactory {
    pub app: TestApp,
}

impl TestDataFactory {
    pub fn new(app: TestApp) -> Self {
        Self { app }
    }

    /// Create a new TestDataFactory with task types pre-registered
    pub async fn new_with_task_types(app: TestApp) -> Self {
        let factory = Self { app };
        factory.register_task_types().await;
        factory
    }

    /// Creates a test user and returns the user data
    pub async fn create_user(&self, username: &str) -> User {
        let user_data = CreateUserRequest {
            username: username.to_string(),
            email: format!("{username}@example.com"),
            password: "SecurePass123!".to_string(),
            role: None,
        };

        let response = self
            .app
            .post_json("/api/v1/auth/register", &user_data)
            .await;
        assert_eq!(response.status(), 200);

        let json: serde_json::Value = response.json().await.unwrap();
        // The response has ApiResponse format: { "success": true, "data": UserProfile, "message": "..." }
        let user_data = &json["data"];
        User {
            id: uuid::Uuid::parse_str(user_data["id"].as_str().unwrap()).unwrap(),
            username: user_data["username"].as_str().unwrap().to_string(),
            email: user_data["email"].as_str().unwrap().to_string(),
            password_hash: "".to_string(), // Not returned in response
            role: user_data["role"].as_str().unwrap().to_string(),
            is_active: user_data["is_active"].as_bool().unwrap(),
            email_verified: user_data["email_verified"].as_bool().unwrap(),
            created_at: chrono::Utc::now(), // Parse from response if needed
            updated_at: chrono::Utc::now(),
            last_login_at: None,
        }
    }

    /// Creates a user and returns an authenticated token
    pub async fn create_authenticated_user(&self, username: &str) -> (User, AuthToken) {
        // Create user
        let user = self.create_user(username).await;

        // Login to get token
        let login_data = json!({
            "username": username,
            "password": "SecurePass123!"
        });

        let response = self.app.post_json("/api/v1/auth/login", &login_data).await;
        assert_eq!(response.status(), 200);

        let token = self.app.extract_auth_token(response).await;

        (user, token)
    }

    /// Creates multiple users for testing pagination
    pub async fn create_multiple_users(&self, count: usize) -> Vec<User> {
        let mut users = Vec::new();

        for i in 0..count {
            let user = self.create_user(&format!("testuser{i}")).await;
            users.push(user);
        }

        users
    }

    /// Register standard task types for testing
    pub async fn register_task_types(&self) {
        let task_types = [
            ("email", "Email notification tasks"),
            ("data_processing", "Data processing and analysis tasks"),
            ("file_cleanup", "File system cleanup tasks"),
            ("report_generation", "Report generation tasks"),
            ("webhook", "Webhook notification tasks"),
            (
                "delay_task",
                "Delay/sleep tasks for testing and chaos scenarios",
            ),
        ];

        for (task_type, description) in task_types.iter() {
            let task_type_data = json!({
                "task_type": task_type,
                "description": description
            });

            let response = self
                .app
                .post_json("/api/v1/tasks/types", &task_type_data)
                .await;
            // Don't assert - task type might already be registered
            if response.status() != 200 {
                eprintln!(
                    "Warning: Failed to register task type '{}': {}",
                    task_type,
                    response.status()
                );
            }
        }
    }

    /// Creates a test task (requires authentication)
    pub async fn create_task(
        &self,
        task_type: &str,
        payload: serde_json::Value,
    ) -> serde_json::Value {
        // Ensure task types are registered before creating tasks
        self.register_task_types().await;

        // Create an authenticated user with unique name
        let unique_username = format!("taskuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let (_user, token) = self.create_authenticated_user(&unique_username).await;

        let task_data = json!({
            "task_type": task_type,
            "payload": payload,
            "priority": "normal"
        });

        let response = self
            .app
            .post_json_auth("/api/v1/tasks", &task_data, &token.token)
            .await;
        assert_eq!(response.status(), 200);

        response.json().await.unwrap()
    }

    /// Creates a test task with custom metadata (requires authentication)
    pub async fn create_task_with_metadata(
        &self,
        task_type: &str,
        payload: serde_json::Value,
        metadata: serde_json::Value,
    ) -> serde_json::Value {
        // Ensure task types are registered before creating tasks
        self.register_task_types().await;

        // Create an authenticated user with unique name
        let unique_username = format!("taskuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let (_user, token) = self.create_authenticated_user(&unique_username).await;

        // Convert metadata Value to HashMap for the API
        let metadata_map: std::collections::HashMap<String, serde_json::Value> =
            if let Some(obj) = metadata.as_object() {
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            } else {
                std::collections::HashMap::new()
            };

        let task_data = json!({
            "task_type": task_type,
            "payload": payload,
            "priority": "normal",
            "metadata": metadata_map
        });

        let response = self
            .app
            .post_json_auth("/api/v1/tasks", &task_data, &token.token)
            .await;
        assert_eq!(response.status(), 200);

        response.json().await.unwrap()
    }
}

/// Test data builders for creating custom requests  
pub mod builders {
    use starter::models::CreateUserRequest;

    pub struct UserBuilder {
        username: String,
        email: String,
        password: String,
        role: Option<String>,
    }

    impl Default for UserBuilder {
        fn default() -> Self {
            Self {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                password: "SecurePass123!".to_string(),
                role: None,
            }
        }
    }

    impl UserBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_username(mut self, username: &str) -> Self {
            self.username = username.to_string();
            self
        }

        pub fn with_email(mut self, email: &str) -> Self {
            self.email = email.to_string();
            self
        }

        pub fn with_password(mut self, password: &str) -> Self {
            self.password = password.to_string();
            self
        }

        pub fn with_role(mut self, role: &str) -> Self {
            self.role = Some(role.to_string());
            self
        }

        pub fn build(self) -> CreateUserRequest {
            CreateUserRequest {
                username: self.username,
                email: self.email,
                password: self.password,
                role: self.role,
            }
        }
    }
}
