//! Tests for history/storage

use super::*;

#[test]
fn test_deduplicate_keeps_first_occurrence() {
    let entries = vec![
        "a".to_string(),
        "b".to_string(),
        "a".to_string(),
        "c".to_string(),
        "b".to_string(),
    ];
    let result = deduplicate(&entries);
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_trim_to_max() {
    let entries: Vec<String> = (0..1500).map(|i| format!("entry{}", i)).collect();
    let trimmed = trim_to_max(&entries);
    assert_eq!(trimmed.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(trimmed[0], "entry0");
}
