use crate::helpers::test_app::{AuthToken, TestApp};
use starter::models::{CreateUserRequest, User};
use serde_json::json;

pub struct TestDataFactory {
    pub app: TestApp,
}

impl TestDataFactory {
    pub fn new(app: TestApp) -> Self {
        Self { app }
    }

    /// Creates a test user and returns the user data
    pub async fn create_user(&self, username: &str) -> User {
        let user_data = CreateUserRequest {
            username: username.to_string(),
            email: format!("{}@example.com", username),
            password: "SecurePass123!".to_string(),
            role: None,
        };

        let response = self.app.post_json("/auth/register", &user_data).await;
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
            "username_or_email": username,
            "password": "SecurePass123!"
        });

        let response = self.app.post_json("/auth/login", &login_data).await;
        assert_eq!(response.status(), 200);

        let token = self.app.extract_auth_token(response).await;

        (user, token)
    }

    /// Creates multiple users for testing pagination
    pub async fn create_multiple_users(&self, count: usize) -> Vec<User> {
        let mut users = Vec::new();

        for i in 0..count {
            let user = self
                .create_user(&format!("testuser{}", i))
                .await;
            users.push(user);
        }

        users
    }

    /// Creates a test task (requires authentication)
    pub async fn create_task(&self, task_type: &str, payload: serde_json::Value) -> serde_json::Value {
        // Create an authenticated user with unique name
        let unique_username = format!("taskuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let (_user, token) = self.create_authenticated_user(&unique_username).await;
        
        let task_data = json!({
            "task_type": task_type,
            "payload": payload,
            "priority": "normal"
        });

        let response = self.app.post_json_auth("/tasks", &task_data, &token.token).await;
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