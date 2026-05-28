use super::*;
use serde_json::{Value, json};

fn collect(values: Vec<Value>) -> Vec<String> {
    let refs: Vec<&Value> = values.iter().collect();
    collect_distinct_strings(&refs)
}

#[test]
fn empty_input_returns_empty() {
    assert!(collect_distinct_strings(&[]).is_empty());
}

#[test]
fn single_string_value() {
    let v = json!("hello");
    assert_eq!(collect_distinct_strings(&[&v]), vec!["hello"]);
}

#[test]
fn ignores_non_string_scalars() {
    let nums = vec![json!(1), json!(2.5), json!(true), json!(null)];
    assert!(collect(nums).is_empty());
}

#[test]
fn dedups_repeated_strings() {
    let vs = vec![json!("a"), json!("a"), json!("b"), json!("a")];
    // freq-sorted: "a"(3) first, "b"(1)
    assert_eq!(collect(vs), vec!["a", "b"]);
}

#[test]
fn frequency_descending_with_alpha_tiebreak() {
    let vs = vec![
        json!("zebra"),
        json!("apple"),
        json!("zebra"),
        json!("mango"),
        json!("apple"),
    ];
    // "apple" 2, "zebra" 2, "mango" 1 → tiebreak alpha: apple, zebra, mango
    assert_eq!(collect(vs), vec!["apple", "zebra", "mango"]);
}

#[test]
fn walks_into_terminal_array_of_strings() {
    // Path `.tags` lands on ["red", "blue", "red"] — collector walks into it.
    let arr = json!(["red", "blue", "red"]);
    assert_eq!(collect_distinct_strings(&[&arr]), vec!["red", "blue"]);
}

#[test]
fn ignores_arrays_with_non_strings() {
    let arr = json!([1, 2, "kept", 3]);
    assert_eq!(collect_distinct_strings(&[&arr]), vec!["kept"]);
}

#[test]
fn objects_at_leaf_are_skipped() {
    // navigate_multi can return objects; we don't suggest object representations.
    let obj = json!({ "k": "v" });
    assert!(collect_distinct_strings(&[&obj]).is_empty());
}

#[test]
fn utf8_strings_preserved() {
    let vs = vec![json!("üñ"), json!("café"), json!("üñ")];
    let r = collect(vs);
    assert_eq!(r, vec!["üñ", "café"]);
}

#[test]
fn caps_at_max_values_per_path() {
    let mut vs = Vec::with_capacity(MAX_VALUES_PER_PATH + 50);
    for i in 0..(MAX_VALUES_PER_PATH + 50) {
        vs.push(Value::String(format!("v{}", i)));
    }
    let r = collect(vs);
    assert_eq!(r.len(), MAX_VALUES_PER_PATH);
}

#[test]
fn cap_applies_to_terminal_array_walk() {
    // A single navigated value that IS an array of >cap distinct strings.
    let big: Vec<Value> = (0..MAX_VALUES_PER_PATH + 50)
        .map(|i| Value::String(format!("v{}", i)))
        .collect();
    let arr = Value::Array(big);
    let r = collect_distinct_strings(&[&arr]);
    assert_eq!(r.len(), MAX_VALUES_PER_PATH);
}

#[test]
fn mixed_string_and_terminal_array_values() {
    // Simulates navigating a path that returns both bare strings and arrays.
    let s1 = json!("alpha");
    let s2 = json!("beta");
    let arr = json!(["alpha", "gamma"]);
    let r = collect_distinct_strings(&[&s1, &s2, &arr]);
    // alpha appears twice (once direct, once in arr), beta once, gamma once.
    // Alphabetical tiebreak between beta and gamma.
    assert_eq!(r, vec!["alpha", "beta", "gamma"]);
}
