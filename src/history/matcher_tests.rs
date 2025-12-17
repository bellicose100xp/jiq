//! Tests for history/matcher

use super::*;

#[test]
fn test_empty_query_returns_all_indices() {
    let matcher = HistoryMatcher::new();
    let entries = vec![".foo".to_string(), ".bar".to_string(), ".baz".to_string()];

    let result = matcher.filter("", &entries);
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn test_exact_match_scores_highest() {
    let matcher = HistoryMatcher::new();
    let entries = vec![
        ".items".to_string(),
        ".items[] | .name".to_string(),
        ".foo".to_string(),
    ];

    let result = matcher.filter(".items", &entries);
    assert!(!result.is_empty());
    assert_eq!(result[0], 0);
}

#[test]
fn test_fuzzy_matching() {
    let matcher = HistoryMatcher::new();
    let entries = vec![
        ".items[] | .name".to_string(),
        ".foo | .bar".to_string(),
        ".data.results".to_string(),
    ];

    let result = matcher.filter("itm", &entries);
    assert!(result.contains(&0));
}

#[test]
fn test_case_insensitive() {
    let matcher = HistoryMatcher::new();
    let entries = vec![".Items".to_string(), ".ITEMS".to_string()];

    let result = matcher.filter("items", &entries);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_no_matches_returns_empty() {
    let matcher = HistoryMatcher::new();
    let entries = vec![".foo".to_string(), ".bar".to_string()];

    let result = matcher.filter("xyz", &entries);
    assert!(result.is_empty());
}

#[test]
fn test_multi_word_search_ands_terms() {
    let matcher = HistoryMatcher::new();
    let entries = vec![
        ".organization.headquarters.facilities.buildings | .[].departments".to_string(),
        ".headquarters.offices".to_string(),
        ".buildings.floors".to_string(),
        ".unrelated.data".to_string(),
    ];

    // Both "headquarters" and "building" must match
    let result = matcher.filter("headquarters building", &entries);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 0); // Only first entry has both terms

    // Single term should match more
    let result = matcher.filter("headquarters", &entries);
    assert_eq!(result.len(), 2); // First two entries
}
