use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

// Include security tests
mod security_tests;

#[tokio::test]
async fn test_create_event() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user
    let unique_username = format!("eventuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let event_data = json!({
        "event_type": "log",
        "source": "test-service",
        "message": "Test log message",
        "level": "info",
        "tags": {
            "environment": "test",
            "service": "api"
        },
        "payload": {
            "request_id": "req-123",
            "user_id": "user-456"
        }
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "success");
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "event_type", &json!("log"));
    assert_json_field(&json["data"], "source", &json!("test-service"));
    assert_json_field(&json["data"], "message", &json!("Test log message"));
}

#[tokio::test]
async fn test_get_events_with_filters() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("eventuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple events
    let events = vec![
        json!({
            "event_type": "log",
            "source": "web-service",
            "message": "Web request",
            "level": "info"
        }),
        json!({
            "event_type": "metric",
            "source": "db-service",
            "message": "Database metric",
            "level": "debug"
        }),
    ];

    for event_data in &events {
        let response = app
            .post_json_auth("/api/v1/monitoring/events", event_data, &token.token)
            .await;
        assert_status(&response, StatusCode::OK);
    }

    // Test filtering by event_type
    let response = app
        .get_auth("/api/v1/monitoring/events?event_type=log", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");

    let data = json["data"].as_array().unwrap();
    assert!(!data.is_empty());
    // All returned events should be log type
    for event in data {
        assert_json_field(event, "event_type", &json!("log"));
    }
}

#[tokio::test]
async fn test_get_events_with_tag_filters() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create events with different tags
    let events = vec![
        json!({
            "event_type": "log",
            "source": "web-service",
            "message": "User login",
            "level": "info",
            "tags": {
                "user_id": "123",
                "environment": "production",
                "action": "login"
            }
        }),
        json!({
            "event_type": "log",
            "source": "api-service",
            "message": "API call",
            "level": "info",
            "tags": {
                "user_id": "456",
                "environment": "production",
                "action": "api_call"
            }
        }),
        json!({
            "event_type": "log",
            "source": "web-service",
            "message": "User logout",
            "level": "info",
            "tags": {
                "user_id": "123",
                "environment": "development",
                "action": "logout"
            }
        }),
    ];

    // Create all events
    for event_data in &events {
        let response = app
            .post_json_auth("/api/v1/monitoring/events", event_data, &token.token)
            .await;
        assert_status(&response, StatusCode::OK);
    }

    // Test single tag filter - should return 2 events with user_id:123
    let response = app
        .get_auth("/api/v1/monitoring/events?tags=user_id:123", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    // Test multiple tag filter - should return 1 event with user_id:123 AND environment:production
    let response = app
        .get_auth(
            "/api/v1/monitoring/events?tags=user_id:123,environment:production",
            &token.token,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_json_field(&data[0]["tags"], "action", &json!("login"));

    // Test non-matching tag filter - should return 0 events
    let response = app
        .get_auth(
            "/api/v1/monitoring/events?tags=nonexistent:value",
            &token.token,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 0);

    // Test invalid tag format - should return 400 error
    let response = app
        .get_auth(
            "/api/v1/monitoring/events?tags=invalid_format",
            &token.token,
        )
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"]
        .as_str()
        .expect("Expected error message in response");
    assert!(error_message.contains("Invalid tag format"));
}

#[tokio::test]
async fn test_create_metric() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("metricuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let metric_data = json!({
        "name": "cpu_usage",
        "metric_type": "gauge",
        "value": 75.5,
        "labels": {
            "host": "server-01",
            "environment": "test"
        }
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "name", &json!("cpu_usage"));
    assert_json_field(&json["data"], "metric_type", &json!("gauge"));
    assert_json_field(&json["data"], "value", &json!(75.5));
}

#[tokio::test]
async fn test_create_alert_requires_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create regular user (not moderator)
    let unique_username = format!("regularuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let alert_data = json!({
        "name": "High CPU Usage",
        "description": "CPU usage above 80%",
        "query": "cpu_usage > 80",
        "threshold_value": 80.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/alerts", &alert_data, &token.token)
        .await;

    // Should be forbidden for regular user
    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_alert_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create moderator user
    let unique_username = format!("moderator_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    let alert_data = json!({
        "name": "High CPU Usage",
        "description": "CPU usage above 80%",
        "query": "cpu_usage > 80",
        "threshold_value": 80.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/alerts", &alert_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "name", &json!("High CPU Usage"));
    assert_json_field(&json["data"], "status", &json!("active"));
}

#[tokio::test]
async fn test_create_incident() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("incidentuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let incident_data = json!({
        "title": "Service outage",
        "description": "API service is responding with 500 errors",
        "severity": "high"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "title", &json!("Service outage"));
    assert_json_field(&json["data"], "severity", &json!("high"));
    assert_json_field(&json["data"], "status", &json!("open"));
}

#[tokio::test]
async fn test_update_incident() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("incidentuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create incident
    let incident_data = json!({
        "title": "Database connection issues",
        "description": "Connection pool exhausted",
        "severity": "medium"
    });

    let create_response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    assert_status(&create_response, StatusCode::OK);
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let incident_id = create_json["data"]["id"].as_str().unwrap();

    // Update incident
    let update_data = json!({
        "status": "investigating",
        "root_cause": "Database server overloaded"
    });

    let update_response = app
        .put_json_auth(
            &format!("/api/v1/monitoring/incidents/{incident_id}"),
            &update_data,
            &token.token,
        )
        .await;

    assert_status(&update_response, StatusCode::OK);
    let update_json: serde_json::Value = update_response.json().await.unwrap();
    assert_json_field(&update_json["data"], "status", &json!("investigating"));
    assert_json_field(
        &update_json["data"],
        "root_cause",
        &json!("Database server overloaded"),
    );
}

#[tokio::test]
async fn test_get_incident_timeline() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("timelineuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create incident
    let incident_data = json!({
        "title": "Timeline test incident",
        "description": "Testing timeline functionality",
        "severity": "low"
    });

    let incident_response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    let incident_json: serde_json::Value = incident_response.json().await.unwrap();
    let incident_id = incident_json["data"]["id"].as_str().unwrap();

    // Create some events around the incident timeframe
    let event_data = json!({
        "event_type": "log",
        "source": "timeline-service",
        "message": "Timeline test event",
        "level": "info"
    });

    app.post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    // Get timeline
    let timeline_response = app
        .get_auth(
            &format!("/api/v1/monitoring/incidents/{incident_id}/timeline"),
            &token.token,
        )
        .await;

    assert_status(&timeline_response, StatusCode::OK);
    let timeline_json: serde_json::Value = timeline_response.json().await.unwrap();
    assert_json_field_exists(&timeline_json["data"], "incident_id");
    assert_json_field_exists(&timeline_json["data"], "entries");
    assert_json_field_exists(&timeline_json["data"], "total_count");
}

#[tokio::test]
async fn test_get_monitoring_stats_requires_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Test with regular user - should be forbidden
    let unique_username = format!("regularuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app.get_auth("/api/v1/monitoring/stats", &token.token).await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_monitoring_stats_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Test with moderator - should succeed
    let unique_username = format!("moderator_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    let response = app.get_auth("/api/v1/monitoring/stats", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "total_events");
    assert_json_field_exists(&json["data"], "total_metrics");
    assert_json_field_exists(&json["data"], "active_alerts");
    assert_json_field_exists(&json["data"], "open_incidents");
    assert_json_field_exists(&json["data"], "events_last_hour");
    assert_json_field_exists(&json["data"], "metrics_last_hour");
}

#[tokio::test]
async fn test_invalid_event_type() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let invalid_event = json!({
        "event_type": "invalid_type",
        "source": "test-service",
        "message": "Test message"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &invalid_event, &token.token)
        .await;

    // Should return validation error for invalid event type
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_prometheus_metrics_endpoint() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("prometheus_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app
        .get_auth("/api/v1/monitoring/metrics/prometheus", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);

    // Should return Prometheus format text
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());

    let text = response.text().await.unwrap();
    assert!(text.contains("# HELP"));
    assert!(text.contains("# TYPE"));
    assert!(text.contains("monitoring_total_events"));
    assert!(text.contains("monitoring_total_metrics"));
}

// ===== SECURITY TESTS FOR TAG PARSING VALIDATION =====

#[tokio::test]
async fn test_tag_parsing_security_too_many_tags() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a query with more than 50 tag pairs (the limit)
    let mut tag_pairs = Vec::new();
    for i in 0..51 {
        tag_pairs.push(format!("key{}:value{}", i, i));
    }
    let tags_query = tag_pairs.join(",");

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/events?tags={}", tags_query),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Too many tag pairs"));
    assert!(error_message.contains("maximum 50"));
}

#[tokio::test]
async fn test_tag_parsing_security_key_too_long() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a tag key longer than 100 characters (the limit)
    let long_key = "a".repeat(101);
    let tags_query = format!("{}:value", long_key);

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/events?tags={}", tags_query),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Tag key too long"));
    assert!(error_message.contains("maximum 100 characters"));
}

#[tokio::test]
async fn test_tag_parsing_security_value_too_long() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a tag value longer than 500 characters (the limit)
    let long_value = "a".repeat(501);
    let tags_query = format!("key:{}", long_value);

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/events?tags={}", tags_query),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Tag value too long"));
    assert!(error_message.contains("maximum 500 characters"));
}

#[tokio::test]
async fn test_tag_parsing_security_invalid_characters() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test various invalid characters in tag keys
    let invalid_keys = vec![
        ("key$injection", "key%24injection:value"),
        ("key;drop", "key%3Bdrop:value"),
        ("key'or'1", "key%27or%271:value"),
        ("key\"quote", "key%22quote:value"),
        ("key<script>", "key%3Cscript%3E:value"),
        ("key|pipe", "key%7Cpipe:value"),
        ("key%percent", "key%25percent:value"),
        ("key#hash", "key%23hash:value"),
        ("key@at", "key%40at:value"),
        ("key+plus", "key%2Bplus:value"),
        ("key=equals", "key%3Dequals:value"),
        ("key space", "key%20space:value"),
        ("key\ttab", "key%09tab:value"),
        ("key\nnewline", "key%0Anewline:value"),
    ];

    for (invalid_key, encoded_query) in invalid_keys {
        let response = app
            .get_auth(
                &format!("/api/v1/monitoring/events?tags={}", encoded_query),
                &token.token,
            )
            .await;

        assert_status(&response, StatusCode::BAD_REQUEST);
        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap();
        assert!(
            error_message.contains("alphanumeric characters, underscores, hyphens, and dots")
                || error_message.contains("Invalid tag format"),
            "Failed for key: {} with error: {}",
            invalid_key,
            error_message
        );
    }
}

#[tokio::test]
async fn test_tag_parsing_security_valid_characters() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test valid characters in tag keys
    let valid_keys = vec![
        "key123",
        "key_underscore",
        "key-hyphen",
        "key.dot",
        "Key_With-Multiple.Valid123",
        "a",
        "A",
        "1",
        "_",
        "-",
        ".",
    ];

    for valid_key in valid_keys {
        let tags_query = format!("{}:value", valid_key);
        let response = app
            .get_auth(
                &format!("/api/v1/monitoring/events?tags={}", tags_query),
                &token.token,
            )
            .await;

        // Should not fail due to character validation (may fail for other reasons like no matching events)
        let status = response.status();
        assert!(
            status == StatusCode::OK || status == StatusCode::NOT_FOUND,
            "Valid key '{}' was rejected with status: {}",
            valid_key,
            status
        );
    }
}

#[tokio::test]
async fn test_tag_parsing_security_duplicate_keys() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let tags_query = "key:value1,key:value2"; // Duplicate key

    let response = app
        .get_auth(
            &format!("/api/v1/monitoring/events?tags={}", tags_query),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Duplicate tag keys are not allowed"));
}

#[tokio::test]
async fn test_tag_parsing_security_empty_keys_values() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("taguser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test empty key and value combinations
    let invalid_tags = vec![
        ":value",  // Empty key
        "key:",    // Empty value
        " :value", // Whitespace-only key
        "key: ",   // Whitespace-only value
        "",        // Empty string
        ":",       // Just colon
        " : ",     // Just whitespace and colon
    ];

    for invalid_tag in invalid_tags {
        let response = app
            .get_auth(
                &format!("/api/v1/monitoring/events?tags={}", invalid_tag),
                &token.token,
            )
            .await;

        if !invalid_tag.is_empty() && invalid_tag != " " {
            assert_status(&response, StatusCode::BAD_REQUEST);
            let json: serde_json::Value = response.json().await.unwrap();
            let error_message = json["error"]["message"].as_str().unwrap();
            assert!(
                error_message.contains("Tag keys and values cannot be empty")
                    || error_message.contains("Invalid tag format"),
                "Failed for tag: '{}' with error: {}",
                invalid_tag,
                error_message
            );
        }
    }
}

// ===== SECURITY TESTS FOR AUTHORIZATION BOUNDARY CONDITIONS =====

#[tokio::test]
async fn test_event_source_authorization_system_sources_require_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create regular user
    let unique_username = format!("user_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let system_sources = vec![
        "system-auth",
        "health-check",
        "monitoring-stats",
        "system-database",
        "health-api",
        "monitoring-alerts",
    ];

    for system_source in system_sources {
        let event_data = json!({
            "event_type": "log",
            "source": system_source,
            "message": "System event"
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
            .await;

        assert_status(&response, StatusCode::FORBIDDEN);
        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap();
        assert!(error_message.contains("not authorized"));
        assert!(error_message.contains(&unique_username));
    }
}

#[tokio::test]
async fn test_event_source_authorization_system_sources_allowed_for_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create moderator user
    let unique_username = format!("mod_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    let system_sources = vec!["system-auth", "health-check", "monitoring-stats"];

    for system_source in system_sources {
        let event_data = json!({
            "event_type": "log",
            "source": system_source,
            "message": "System event from moderator"
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_event_source_authorization_generic_sources_allowed() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("user_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let allowed_sources = vec![
        "app-frontend",
        "web-server",
        "api-gateway",
        "test-suite",
        "db-migration",
        "generic_source",
        "simple",
    ];

    for source in allowed_sources {
        let event_data = json!({
            "event_type": "log",
            "source": source,
            "message": "Generic event"
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_event_source_authorization_user_owned_sources() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (user, token) = factory.create_authenticated_user(&unique_username).await;

    let user_owned_sources = vec![
        format!("{}-service", unique_username),
        format!("user-{}-worker", user.id),
        format!("{}-application", unique_username),
        format!("{}-module", unique_username),
    ];

    for source in user_owned_sources {
        let event_data = json!({
            "event_type": "log",
            "source": source,
            "message": "User-owned event"
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_event_source_authorization_other_user_sources_blocked() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let user1_username = format!("user1_{}", &Uuid::new_v4().to_string()[..8]);
    let user2_username = format!("user2_{}", &Uuid::new_v4().to_string()[..8]);

    let (user1, _) = factory.create_authenticated_user(&user1_username).await;
    let (_user2, token2) = factory.create_authenticated_user(&user2_username).await;

    let other_user_sources = vec![
        format!("{}-service", user1_username),
        format!("user-{}-worker", user1.id),
        format!("unknown_user-service"),
        format!("other-user-application"),
    ];

    for source in other_user_sources {
        let event_data = json!({
            "event_type": "log",
            "source": source,
            "message": "Unauthorized event"
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/events", &event_data, &token2.token)
            .await;

        assert_status(&response, StatusCode::FORBIDDEN);
        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap();
        assert!(error_message.contains("not authorized"));
    }
}

#[tokio::test]
async fn test_metric_name_authorization_system_metrics_require_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create regular user
    let unique_username = format!("user_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let system_metrics = vec![
        "system_cpu_usage",
        "health_check_duration",
        "monitoring_event_count",
        "system_memory_usage",
        "health_database_latency",
        "monitoring_alert_count",
    ];

    for system_metric in system_metrics {
        let metric_data = json!({
            "name": system_metric,
            "metric_type": "gauge",
            "value": 50.0
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
            .await;

        assert_status(&response, StatusCode::FORBIDDEN);
        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap();
        assert!(error_message.contains("not authorized"));
        assert!(error_message.contains(&unique_username));
    }
}

#[tokio::test]
async fn test_metric_name_authorization_generic_metrics_allowed() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("user_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let allowed_metrics = vec![
        "http_requests_total",
        "response_time_seconds",
        "cpu_usage",
        "memory_usage_bytes",
        "disk_usage_percent",
        "network_bytes_sent",
        "error_rate_percent",
        "latency_ms",
        "throughput_rps",
        "requests_per_second",
        "active_connections_count",
        "queue_size_items",
    ];

    for metric in allowed_metrics {
        let metric_data = json!({
            "name": metric,
            "metric_type": "gauge",
            "value": 42.0
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_metric_name_authorization_user_owned_metrics() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (user, token) = factory.create_authenticated_user(&unique_username).await;

    let user_owned_metrics = vec![
        format!("{}_response_time", unique_username),
        format!("user_{}_{}", user.id, "request_count"),
        format!("{}_service_uptime", unique_username),
        format!("{}_worker_processed", unique_username),
    ];

    for metric in user_owned_metrics {
        let metric_data = json!({
            "name": metric,
            "metric_type": "counter",
            "value": 123.0
        });

        let response = app
            .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }
}
