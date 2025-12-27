use crate::autocomplete::*;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

pub fn tracker_for(query: &str) -> BraceTracker {
    let mut tracker = BraceTracker::new();
    tracker.rebuild(query);
    tracker
}

pub fn create_array_of_objects_json() -> (Arc<Value>, ResultType) {
    let json = r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#;
    let parsed = serde_json::from_str::<Value>(json).unwrap();
    (Arc::new(parsed), ResultType::ArrayOfObjects)
}
