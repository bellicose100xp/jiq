//! JSON utility functions
//!
//! General-purpose utilities for JSON manipulation and analysis.

use serde_json::Value;

/// Size thresholds for dynamic depth (in bytes)
pub const SMALL_THRESHOLD: usize = 1024 * 1024; // 1 MB
pub const MEDIUM_THRESHOLD: usize = 10 * 1024 * 1024; // 10 MB
pub const LARGE_THRESHOLD: usize = 100 * 1024 * 1024; // 100 MB

/// Depth values for each tier
pub const SMALL_DEPTH: usize = 30;
pub const MEDIUM_DEPTH: usize = 20;
pub const LARGE_DEPTH: usize = 10;
pub const VERY_LARGE_DEPTH: usize = 5;

/// Calculate schema depth based on JSON size
///
/// Returns appropriate depth based on size thresholds:
/// - Small (< 1 MB): depth 30 (comprehensive)
/// - Medium (1-10 MB): depth 20 (balanced)
/// - Large (10-100 MB): depth 10 (focused)
/// - Very Large (> 100 MB): depth 5 (minimal)
pub fn calculate_schema_depth(json_size: usize) -> usize {
    if json_size < SMALL_THRESHOLD {
        SMALL_DEPTH
    } else if json_size < MEDIUM_THRESHOLD {
        MEDIUM_DEPTH
    } else if json_size < LARGE_THRESHOLD {
        LARGE_DEPTH
    } else {
        VERY_LARGE_DEPTH
    }
}

/// Extract a type-only schema from JSON
///
/// Recursively converts JSON values into a schema where leaf values are replaced
/// with their type names ("string", "number", "boolean", "null").
///
/// # Arguments
/// * `json` - The JSON string to extract schema from
/// * `max_depth` - Maximum depth to recurse (prevents huge schemas)
///
/// # Returns
/// * `Some(String)` - JSON schema string on success
/// * `None` - If JSON is invalid or extraction fails
///
/// # Examples
/// ```
/// use jiq::json::extract_json_schema;
///
/// let json = r#"{"name": "John", "age": 30}"#;
/// let schema = extract_json_schema(json, 5);
/// // Returns: Some(r#"{"name":"string","age":"number"}"#)
/// ```
pub fn extract_json_schema(json: &str, max_depth: usize) -> Option<String> {
    let value: Value = serde_json::from_str(json).ok()?;
    let schema_value = value_to_schema(&value, 0, max_depth)?;
    serde_json::to_string(&schema_value).ok()
}

/// Extract a type-only schema from JSON with dynamic depth based on input size
///
/// Automatically determines the appropriate extraction depth based on JSON size:
/// - Small files (< 1 MB): Full depth extraction
/// - Medium files (1-10 MB): Balanced extraction
/// - Large files (10-100 MB): Focused extraction
/// - Very large files (> 100 MB): Minimal extraction
///
/// # Arguments
/// * `json` - The JSON string to extract schema from
///
/// # Returns
/// * `Some(String)` - JSON schema string on success
/// * `None` - If JSON is invalid or extraction fails
///
/// # Examples
/// ```
/// use jiq::json::extract_json_schema_dynamic;
///
/// let json = r#"{"name": "John", "age": 30}"#;
/// let schema = extract_json_schema_dynamic(json);
/// // Returns: Some(r#"{"name":"string","age":"number"}"#)
/// ```
pub fn extract_json_schema_dynamic(json: &str) -> Option<String> {
    let depth = calculate_schema_depth(json.len());
    extract_json_schema(json, depth)
}

/// Convert a serde_json::Value to a schema Value recursively
fn value_to_schema(value: &Value, current_depth: usize, max_depth: usize) -> Option<Value> {
    // Stop recursion at max depth
    if current_depth >= max_depth {
        return Some(Value::String("...".to_string()));
    }

    match value {
        Value::Null => Some(Value::String("null".to_string())),
        Value::Bool(_) => Some(Value::String("boolean".to_string())),
        Value::Number(_) => Some(Value::String("number".to_string())),
        Value::String(_) => Some(Value::String("string".to_string())),
        Value::Array(arr) => {
            if arr.is_empty() {
                Some(Value::Array(vec![]))
            } else {
                // Use first element as representative for array schema
                let first_schema = value_to_schema(&arr[0], current_depth + 1, max_depth)?;
                Some(Value::Array(vec![first_schema]))
            }
        }
        Value::Object(map) => {
            let mut schema_map = serde_json::Map::new();
            for (key, val) in map {
                let val_schema = value_to_schema(val, current_depth + 1, max_depth)?;
                schema_map.insert(key.clone(), val_schema);
            }
            Some(Value::Object(schema_map))
        }
    }
}

#[cfg(test)]
#[path = "json_tests.rs"]
mod json_tests;
