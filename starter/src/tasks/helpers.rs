//! Convenience helpers and macros for task handling

/// Macro for extracting required string fields from task payload
///
/// Usage:
/// ```rust
/// let to = require_field!(payload, "to")?;
/// let subject = require_field!(payload, "subject")?;
/// ```
#[macro_export]
macro_rules! require_field {
    ($payload:expr, $field:literal) => {
        $payload
            .get($field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| $crate::tasks::types::TaskError::missing_field($field))
    };
}

/// Macro for extracting optional string fields from task payload
///
/// Usage:
/// ```rust
/// let optional_field = optional_field!(payload, "optional_field");
/// ```
#[macro_export]
macro_rules! optional_field {
    ($payload:expr, $field:literal) => {
        $payload.get($field).and_then(|v| v.as_str())
    };
}

/// Macro for extracting required fields of specific types
///
/// Usage:
/// ```rust
/// let count = require_typed_field!(payload, "count", as_i64)?;
/// let enabled = require_typed_field!(payload, "enabled", as_bool)?;
/// ```
#[macro_export]
macro_rules! require_typed_field {
    ($payload:expr, $field:literal, $method:ident) => {
        $payload
            .get($field)
            .and_then(|v| v.$method())
            .ok_or_else(|| {
                $crate::tasks::types::TaskError::invalid_field_type($field, stringify!($method))
            })
    };
}

/// Helper function to extract multiple required string fields at once
///
/// Usage:
/// ```rust
/// let (to, subject, body) = extract_fields!(payload, "to", "subject", "body")?;
/// ```
#[macro_export]
macro_rules! extract_fields {
    ($payload:expr, $($field:literal),+) => {
        (|| -> Result<_, $crate::tasks::types::TaskError> {
            Ok(($(
                $payload
                    .get($field)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| $crate::tasks::types::TaskError::missing_field($field))?,
            )+))
        })()
    };
}

/// Helper for creating task metadata
pub fn create_metadata() -> std::collections::HashMap<String, serde_json::Value> {
    std::collections::HashMap::new()
}

/// Helper for adding common metadata fields
pub fn add_timestamp_metadata(metadata: &mut std::collections::HashMap<String, serde_json::Value>) {
    metadata.insert(
        "processed_at".to_string(),
        serde_json::json!(chrono::Utc::now()),
    );
}

#[cfg(test)]
mod tests {
    
    use serde_json::json;

    #[test]
    fn test_require_field_macro() -> Result<(), crate::tasks::types::TaskError> {
        let payload = json!({
            "to": "test@example.com",
            "subject": "Test Subject"
        });

        // Test successful extraction
        let to = require_field!(payload, "to")?;
        assert_eq!(to, "test@example.com");

        // Test missing field should return error
        let result = require_field!(payload, "missing");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_optional_field_macro() {
        let payload = json!({
            "to": "test@example.com"
        });

        let to = optional_field!(payload, "to");
        assert_eq!(to, Some("test@example.com"));

        let missing = optional_field!(payload, "missing");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_extract_fields_macro() -> Result<(), crate::tasks::types::TaskError> {
        let payload = json!({
            "to": "test@example.com",
            "subject": "Test Subject",
            "body": "Test Body"
        });

        let (to, subject, body) = extract_fields!(payload, "to", "subject", "body")?;
        assert_eq!(to, "test@example.com");
        assert_eq!(subject, "Test Subject");
        assert_eq!(body, "Test Body");

        Ok(())
    }
}
