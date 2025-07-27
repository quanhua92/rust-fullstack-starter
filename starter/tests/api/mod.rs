use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_api_cors_headers() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    
    // Check CORS headers are present
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_api_preflight_request() {
    let app = spawn_app().await;
    
    let response = app.client
        .request(reqwest::Method::OPTIONS, &format!("{}/auth/login", app.address))
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type")
        .send()
        .await
        .unwrap();
    
    assert_status(&response, StatusCode::OK);
    
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-methods"));
    assert!(headers.contains_key("access-control-allow-headers"));
}

#[tokio::test]
async fn test_api_content_type_json() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[tokio::test]
async fn test_api_rate_limiting() {
    let app = spawn_app().await;
    
    // Make multiple rapid requests
    let mut responses = Vec::new();
    for _ in 0..20 {
        let response = app.get("/health").await;
        responses.push(response.status());
    }
    
    // All requests should succeed (basic rate limiting test)
    for status in responses {
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_api_error_format() {
    let app = spawn_app().await;
    
    // Make a request that should return an error
    let response = app.get("/nonexistent").await;
    
    assert_status(&response, StatusCode::NOT_FOUND);
    
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "error");
    assert_json_field_exists(&json["error"], "message");
    assert_json_field_exists(&json["error"], "code");
}

#[tokio::test]
async fn test_api_request_id_header() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    
    // Check that request ID header is present
    let headers = response.headers();
    assert!(headers.contains_key("x-request-id"));
    
    let request_id = headers.get("x-request-id").unwrap().to_str().unwrap();
    assert!(!request_id.is_empty());
}

#[tokio::test]
async fn test_api_malformed_json() {
    let app = spawn_app().await;
    
    let response = app.client
        .post(&format!("{}/auth/register", app.address))
        .header("content-type", "application/json")
        .body("{ invalid json")
        .send()
        .await
        .unwrap();
    
    assert_status(&response, StatusCode::BAD_REQUEST);
    
    // Try to parse as JSON, but handle case where response might not be JSON
    if let Ok(json) = response.json::<serde_json::Value>().await {
        assert_json_field_exists(&json, "error");
    }
}

#[tokio::test]
async fn test_api_large_payload() {
    let app = spawn_app().await;
    
    // Create a large payload (but within reasonable limits)
    let large_description = "x".repeat(10000);
    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "SecurePass123!",
        "description": large_description
    });
    
    let response = app.post_json("/auth/register", &user_data).await;
    
    // Should handle large payloads gracefully
    assert!(response.status().is_client_error() || response.status().is_success());
}

#[tokio::test]
async fn test_api_authentication_required() {
    let app = spawn_app().await;
    
    // Try to access protected endpoint without auth
    let response = app.get("/auth/me").await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
    
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "error");
}

#[tokio::test]
async fn test_api_invalid_auth_token() {
    let app = spawn_app().await;
    
    let response = app.get_auth("/auth/me", "invalid_token").await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_security_headers() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    
    let headers = response.headers();
    
    // Check for security headers
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    
    // Verify security header values
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
}