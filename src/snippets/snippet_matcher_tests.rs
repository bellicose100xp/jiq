use super::*;
use crate::snippets::Snippet;

fn create_snippet(name: &str) -> Snippet {
    Snippet {
        name: name.to_string(),
        query: ".".to_string(),
        description: None,
    }
}

fn create_snippets(names: &[&str]) -> Vec<Snippet> {
    names.iter().map(|name| create_snippet(name)).collect()
}

#[test]
fn test_empty_query_returns_all_indices() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select keys", "Flatten arrays", "Filter items"]);

    let result = matcher.filter("", &snippets);
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn test_whitespace_query_returns_all_indices() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select keys", "Flatten arrays"]);

    let result = matcher.filter("   ", &snippets);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_exact_match_scores_highest() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select keys", "Select all keys from object", "Flatten"]);

    let result = matcher.filter("Select keys", &snippets);
    assert!(!result.is_empty());
    assert_eq!(result[0], 0);
}

#[test]
fn test_fuzzy_matching() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select all keys", "Flatten arrays", "Filter items"]);

    let result = matcher.filter("slct", &snippets);
    assert!(result.contains(&0));
}

#[test]
fn test_case_insensitive() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select Keys", "SELECT KEYS"]);

    let result = matcher.filter("select keys", &snippets);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_no_matches_returns_empty() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Select keys", "Flatten arrays"]);

    let result = matcher.filter("xyz123", &snippets);
    assert!(result.is_empty());
}

#[test]
fn test_multi_term_and_matching() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&[
        "Select all keys from object",
        "Select items",
        "Get all keys",
        "Unrelated snippet",
    ]);

    let result = matcher.filter("select keys", &snippets);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 0);
}

#[test]
fn test_multi_term_order_independent() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Filter active users", "Users filter active"]);

    let result1 = matcher.filter("filter users", &snippets);
    let result2 = matcher.filter("users filter", &snippets);

    assert_eq!(result1.len(), result2.len());
}

#[test]
fn test_single_snippet() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&["Identity"]);

    let result = matcher.filter("id", &snippets);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_empty_snippets() {
    let matcher = SnippetMatcher::new();
    let snippets: Vec<Snippet> = vec![];

    let result = matcher.filter("test", &snippets);
    assert!(result.is_empty());
}

#[test]
fn test_scoring_prefers_better_matches() {
    let matcher = SnippetMatcher::new();
    let snippets = create_snippets(&[
        "Something with keys at the end",
        "keys",
        "The keys are here",
    ]);

    let result = matcher.filter("keys", &snippets);
    assert_eq!(result[0], 1);
}

#[test]
fn test_default_trait() {
    let matcher = SnippetMatcher::default();
    let snippets = create_snippets(&["test"]);

    let result = matcher.filter("", &snippets);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_debug_trait() {
    let matcher = SnippetMatcher::new();
    let debug_output = format!("{:?}", matcher);
    assert!(debug_output.contains("SnippetMatcher"));
}
