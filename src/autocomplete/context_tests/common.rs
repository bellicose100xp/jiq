use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

pub fn tracker_for(query: &str) -> BraceTracker {
    let mut tracker = BraceTracker::new();
    tracker.rebuild(query);
    tracker
}

pub fn empty_field_names() -> Arc<HashSet<String>> {
    Arc::new(HashSet::new())
}

/// Extract all field names from a JSON value recursively (for tests).
pub fn field_names_from(value: &Value) -> Arc<HashSet<String>> {
    let mut fields = HashSet::new();
    collect_fields_recursive(value, &mut fields);
    Arc::new(fields)
}

fn collect_fields_recursive(value: &Value, fields: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                fields.insert(key.clone());
                collect_fields_recursive(val, fields);
            }
        }
        Value::Array(arr) => {
            for element in arr.iter().take(10) {
                collect_fields_recursive(element, fields);
            }
        }
        _ => {}
    }
}

pub fn create_array_of_objects_json() -> (Arc<Value>, ResultType) {
    let json = r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::ArrayOfObjects)
}
