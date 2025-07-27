use reqwest::StatusCode;
use serde_json::Value;

/// Assert response status code
pub fn assert_status(response: &reqwest::Response, expected: StatusCode) {
    let actual = response.status();
    assert_eq!(actual, expected, "Expected status {expected}, got {actual}");
}

/// Assert JSON field value
pub fn assert_json_field(json: &Value, field: &str, expected: &Value) {
    let actual = &json[field];
    assert_eq!(
        actual, expected,
        "Expected field '{field}' to be {expected}, got {actual}"
    );
}

/// Assert JSON field exists
pub fn assert_json_field_exists(json: &Value, field: &str) {
    assert!(!json[field].is_null(), "Expected field '{field}' to exist");
}

/// Extract error message from response
pub async fn extract_error_message(response: reqwest::Response) -> String {
    let json: Value = response
        .json()
        .await
        .expect("Failed to parse error response");
    json["error"]["message"]
        .as_str()
        .unwrap_or("Unknown error")
        .to_string()
}

/// Wait for condition with timeout
pub async fn wait_for<F, Fut>(mut condition: F, timeout_ms: u64) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    false
}

/// Generate random string
pub fn random_string(length: usize) -> String {
    use rand::Rng;
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

/// Generate random email
pub fn random_email() -> String {
    format!("{}@example.com", random_string(10))
}

/// Assert timestamp is recent (within last 5 seconds)
pub fn assert_recent_timestamp(timestamp: &str) {
    let parsed = chrono::DateTime::parse_from_rfc3339(timestamp).expect("Invalid timestamp format");
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(parsed.with_timezone(&chrono::Utc));

    assert!(
        diff.num_seconds() < 5,
        "Timestamp {} is not recent ({}s ago)",
        timestamp,
        diff.num_seconds()
    );
}
