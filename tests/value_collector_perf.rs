//! Performance gate for the value-autocomplete data sources.
//!
//! Builds a ~10 MB synthetic JSON in-memory and asserts:
//! - The lazy `JqExecutor::all_string_values()` precompute fits within budget
//!   on first call (it runs synchronously on the keystroke that triggers
//!   value autocomplete with no path-scoped match).
//! - Subsequent accesses are zero-cost (Arc clone only).

use jiq::autocomplete::value_collector::collect_distinct_strings;
use jiq::query::executor::JqExecutor;
use serde_json::{Value, json};
use std::time::Instant;

const TARGET_OBJECTS: usize = 100_000;
const STATUSES: &[&str] = &[
    "active", "inactive", "pending", "archived", "draft", "review", "approved", "rejected",
];

fn build_fixture() -> String {
    let pad: String = "x".repeat(64);
    let mut arr = Vec::with_capacity(TARGET_OBJECTS);
    for i in 0..TARGET_OBJECTS {
        arr.push(json!({
            "status": STATUSES[i % STATUSES.len()],
            "blob": pad,
        }));
    }
    serde_json::to_string(&Value::Array(arr)).expect("serialize fixture")
}

// Perf budgets are deliberately loose so coverage / debug runs (which can be
// 5-10x slower than release) don't flake. The release binary easily beats
// these by an order of magnitude.

#[test]
fn all_string_values_first_call_completes() {
    let json = build_fixture();
    let executor = JqExecutor::new(json);
    let start = Instant::now();
    let values = executor.all_string_values();
    let elapsed = start.elapsed();
    assert!(!values.is_empty());
    assert!(
        elapsed.as_secs() < 30,
        "all_string_values first call took {:?} on 10MB fixture (budget: 30s; expect <50ms in release)",
        elapsed
    );
    eprintln!("all_string_values first call: {:?}", elapsed);
}

#[test]
fn all_string_values_warm_call_is_arc_clone() {
    let json = build_fixture();
    let executor = JqExecutor::new(json);
    let _ = executor.all_string_values();
    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = executor.all_string_values();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "100k warm calls took {:?} (budget: 5s for debug/coverage; expect <50ms in release)",
        elapsed
    );
    eprintln!("100k warm calls: {:?}", elapsed);
}

#[test]
fn collect_distinct_strings_on_navigated_values() {
    // Simulate the per-keystroke walk on a 100k-element array path.
    let parsed: Value = serde_json::from_str(&build_fixture()).expect("parse fixture");
    let arr = match &parsed {
        Value::Array(a) => a,
        _ => panic!("expected array"),
    };
    let navigated: Vec<&Value> = arr
        .iter()
        .filter_map(|el| el.as_object().and_then(|m| m.get("status")))
        .collect();
    let start = Instant::now();
    let strings = collect_distinct_strings(&navigated);
    let elapsed = start.elapsed();
    assert_eq!(strings.len(), STATUSES.len());
    assert!(
        elapsed.as_secs() < 10,
        "collect_distinct_strings on 100k values took {:?} (budget: 10s for debug/coverage; expect <50ms in release)",
        elapsed
    );
    eprintln!("collect_distinct_strings on 100k: {:?}", elapsed);
}
