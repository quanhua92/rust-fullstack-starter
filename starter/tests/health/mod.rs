use crate::helpers::*;
use reqwest::StatusCode;

#[tokio::test]
async fn test_health_endpoint() {
    let app = spawn_app().await;

    let response = app.get("/health").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "healthy");
}

#[tokio::test]
async fn test_health_detailed() {
    let app = spawn_app().await;

    let response = app.get("/health/detailed").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "healthy");
    assert_json_field_exists(&json["data"], "checks");
}

#[tokio::test]
async fn test_liveness_probe() {
    let app = spawn_app().await;

    let response = app.get("/health/live").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "alive");
    assert_eq!(json["data"]["probe"], "liveness");
    assert_json_field_exists(&json["data"], "timestamp");
}

#[tokio::test]
async fn test_readiness_probe() {
    let app = spawn_app().await;

    let response = app.get("/health/ready").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "ready");
    assert_eq!(json["data"]["probe"], "readiness");
    assert_json_field_exists(&json["data"], "timestamp");
    assert_json_field_exists(&json["data"], "checks");
    
    // Verify database and application checks are included
    let checks = json["data"]["checks"].as_object().unwrap();
    assert_json_field_exists(&serde_json::Value::Object(checks.clone()), "database");
    assert_json_field_exists(&serde_json::Value::Object(checks.clone()), "application");
    assert_eq!(checks["database"]["status"], "healthy");
    assert_eq!(checks["application"]["status"], "healthy");
}

#[tokio::test]
async fn test_startup_probe() {
    let app = spawn_app().await;

    let response = app.get("/health/startup").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "started");
    assert_eq!(json["data"]["probe"], "startup");
    assert_json_field_exists(&json["data"], "timestamp");
    assert_json_field_exists(&json["data"], "checks");
    
    // Verify database and schema checks are included
    let checks = json["data"]["checks"].as_object().unwrap();
    assert_json_field_exists(&serde_json::Value::Object(checks.clone()), "database");
    assert_json_field_exists(&serde_json::Value::Object(checks.clone()), "schema");
    assert_eq!(checks["database"]["status"], "healthy");
    assert_eq!(checks["schema"]["status"], "healthy");
}

#[tokio::test]
async fn test_database_health_check() {
    let app = spawn_app().await;

    let response = app.get("/health/detailed").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();

    // Verify database check is included
    let checks = json["data"]["checks"].as_object().unwrap();
    assert_json_field_exists(&serde_json::Value::Object(checks.clone()), "database");
    assert_eq!(checks["database"]["status"], "healthy");
}

#[tokio::test]
async fn test_health_includes_version() {
    let app = spawn_app().await;

    let response = app.get("/health").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "version");
}

#[tokio::test]
async fn test_health_includes_uptime() {
    let app = spawn_app().await;

    // Wait a moment to ensure uptime is measurable
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let response = app.get("/health").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "uptime");

    // Verify uptime is a positive number
    let uptime = json["data"]["uptime"].as_f64().unwrap();
    assert!(uptime > 0.0, "Uptime should be positive");
}
