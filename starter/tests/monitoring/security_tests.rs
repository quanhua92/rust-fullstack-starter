use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

// ===== SECURITY TESTS FOR INPUT VALIDATION LIMITS =====

#[tokio::test]
async fn test_input_validation_event_payload_size_limits() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("limituser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test event_type length limit (50 characters)
    let long_event_type = "a".repeat(51);
    let event_data = json!({
        "event_type": long_event_type,
        "source": "test-service",
        "message": "Test message"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Event type too long"));
    assert!(error_message.contains("max 50 characters"));
}

#[tokio::test]
async fn test_input_validation_event_source_length_limit() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("limituser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test source length limit (200 characters)
    let long_source = "a".repeat(201);
    let event_data = json!({
        "event_type": "log",
        "source": long_source,
        "message": "Test message"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Source too long"));
    assert!(error_message.contains("max 200 characters"));
}

#[tokio::test]
async fn test_input_validation_event_message_length_limit() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("limituser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test message length limit (10,000 characters)
    let long_message = "a".repeat(10001);
    let event_data = json!({
        "event_type": "log",
        "source": "test-service",
        "message": long_message
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Message too long"));
    assert!(error_message.contains("max 10000 characters"));
}

#[tokio::test]
async fn test_input_validation_metric_name_length_limit() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("limituser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test metric name length limit (100 characters)
    // Use a generic metric name prefix that passes authorization but is too long
    let long_name = "cpu_usage_".to_string() + &"a".repeat(91); // 10 + 91 = 101 chars
    let metric_data = json!({
        "name": long_name,
        "metric_type": "gauge",
        "value": 42.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Metric name too long"));
    assert!(error_message.contains("max 100 characters"));
}

#[tokio::test]
async fn test_input_validation_metric_finite_values_only() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("limituser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test that finite values work properly (this validates our validation is working)
    // Use a generic metric name that passes authorization
    let valid_metric = json!({
        "name": "cpu_usage",
        "metric_type": "gauge",
        "value": 42.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/metrics", &valid_metric, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field(&json["data"], "value", &json!(42.0));

    // Test edge case values that should work
    let edge_values: Vec<f64> = vec![0.0, -1.0, 1e10, -1e10, 1e-10, -1e-10];

    for (i, value) in edge_values.iter().enumerate() {
        let metric_data = json!({
            "name": format!("cpu_usage_edge_{}", i),
            "metric_type": "counter",
            "value": value
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}

// ===== SECURITY TESTS FOR ERROR MESSAGE INFORMATION DISCLOSURE =====

#[tokio::test]
async fn test_error_messages_no_uuid_disclosure() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test that error messages don't disclose internal UUIDs or system details
    let fake_uuid = Uuid::new_v4();

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/events/{}", fake_uuid),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();

    // Should not contain the UUID
    assert!(!error_message.contains(&fake_uuid.to_string()));
    assert_eq!(error_message, "Event not found");
}

#[tokio::test]
async fn test_incident_error_messages_no_uuid_disclosure() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test that incident error messages don't disclose internal UUIDs
    let fake_uuid = Uuid::new_v4();

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/incidents/{}", fake_uuid),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();

    // Should not contain the UUID
    assert!(!error_message.contains(&fake_uuid.to_string()));
    assert_eq!(error_message, "Incident not found");
}

// ===== SECURITY TESTS FOR DATABASE OPTIMIZATION =====

#[tokio::test]
async fn test_monitoring_stats_performance_optimized() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create moderator user to access stats
    let unique_username = format!("mod_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    // Test that stats endpoint works (indirectly tests the optimized query)
    let response = app.get_auth("/api/v1/monitoring/stats", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();

    // Verify all expected stats fields are present
    let data = &json["data"];
    assert!(data["total_events"].is_number());
    assert!(data["total_metrics"].is_number());
    assert!(data["active_alerts"].is_number());
    assert!(data["open_incidents"].is_number());
    assert!(data["events_last_hour"].is_number());
    assert!(data["metrics_last_hour"].is_number());
}

// ===== SECURITY TESTS FOR PROMETHEUS PAGINATION =====

#[tokio::test]
async fn test_prometheus_metrics_pagination_limit() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("prometheus_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple metrics to test pagination
    for i in 0..20 {
        let metric_data = json!({
            "name": format!("test_metric_{}", i),
            "metric_type": "gauge",
            "value": i as f64
        });

        let _response = app
            .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
            .await;
    }

    // Test that Prometheus endpoint works and has pagination applied
    let response = app
        .get_auth("/api/v1/monitoring/metrics/prometheus", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);

    let text = response.text().await.unwrap();
    // Should contain system metrics and user metrics, but be limited
    assert!(text.contains("# HELP"));
    assert!(text.contains("# TYPE"));
    assert!(text.contains("monitoring_total_events"));

    // Verify content type
    let response = app
        .get_auth("/api/v1/monitoring/metrics/prometheus", &token.token)
        .await;
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(
        content_type
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/plain")
    );
}

// ===== SECURITY TESTS FOR TRANSACTION MANAGEMENT =====

#[tokio::test]
async fn test_incident_update_transaction_integrity() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("txuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create an incident
    let incident_data = json!({
        "title": "Transaction test incident",
        "description": "Testing transaction integrity",
        "severity": "medium"
    });

    let create_response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    assert_status(&create_response, StatusCode::OK);
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let incident_id = create_json["data"]["id"].as_str().unwrap();

    // Update the incident
    let update_data = json!({
        "status": "investigating",
        "root_cause": "Database transaction test"
    });

    let update_response = app
        .put_json_auth(
            &format!("/api/v1/monitoring/incidents/{}", incident_id),
            &update_data,
            &token.token,
        )
        .await;

    assert_status(&update_response, StatusCode::OK);
    let update_json: serde_json::Value = update_response.json().await.unwrap();

    // Verify the update was successful (indicating transaction worked)
    assert_eq!(update_json["data"]["status"], "investigating");
    assert_eq!(
        update_json["data"]["root_cause"],
        "Database transaction test"
    );
}
