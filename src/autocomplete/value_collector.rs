//! Distinct string-value collection from already-navigated JSON values.
//!
//! Takes the `Vec<&Value>` produced by `json_navigator::navigate_multi` and
//! produces a deduplicated, frequency-sorted (alphabetical tiebreaker),
//! capped list of distinct strings.
//!
//! Walks INTO terminal arrays so that paths landing on `["red", "blue"]` count
//! each string element rather than the array as a whole.

use serde_json::Value;
use std::collections::HashMap;

/// Per-path cap on distinct values returned. Bounds the suggestion list.
pub const MAX_VALUES_PER_PATH: usize = 10_000;

/// Collect distinct string values from a list of already-navigated JSON
/// values. Each leaf string is counted; arrays at the leaf are walked into so
/// `["a", "b", "a"]` produces `["a", "b"]` with `a` ranked first.
///
/// Output is sorted by descending frequency, alphabetical tiebreaker, capped
/// at `MAX_VALUES_PER_PATH`.
pub fn collect_distinct_strings(values: &[&Value]) -> Vec<String> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for value in values {
        accumulate(value, &mut counts);
        if counts.len() >= MAX_VALUES_PER_PATH {
            break;
        }
    }
    finalize(counts)
}

fn accumulate(value: &Value, counts: &mut HashMap<String, u32>) {
    match value {
        Value::String(s) => {
            count_one(counts, s);
        }
        Value::Array(arr) => {
            for element in arr {
                if let Value::String(s) = element {
                    count_one(counts, s);
                    if counts.len() >= MAX_VALUES_PER_PATH {
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}

fn count_one(counts: &mut HashMap<String, u32>, s: &str) {
    if let Some(c) = counts.get_mut(s) {
        *c += 1;
    } else if counts.len() < MAX_VALUES_PER_PATH {
        counts.insert(s.to_string(), 1);
    }
}

fn finalize(counts: HashMap<String, u32>) -> Vec<String> {
    let mut entries: Vec<(String, u32)> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries.into_iter().map(|(s, _)| s).collect()
}

#[cfg(test)]
#[path = "value_collector_tests.rs"]
mod value_collector_tests;
