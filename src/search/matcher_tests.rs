//! Tests for matcher

use super::*;
use proptest::prelude::*;

#[test]
fn test_empty_query() {
    let matches = SearchMatcher::find_all("hello world", "");
    assert!(matches.is_empty());
}

#[test]
fn test_empty_content() {
    let matches = SearchMatcher::find_all("", "hello");
    assert!(matches.is_empty());
}

#[test]
fn test_single_match() {
    let matches = SearchMatcher::find_all("hello world", "world");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].line, 0);
    assert_eq!(matches[0].col, 6);
    assert_eq!(matches[0].len, 5);
}

#[test]
fn test_case_insensitive() {
    let matches = SearchMatcher::find_all("Hello WORLD", "world");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 6);
}

#[test]
fn test_multiple_matches_same_line() {
    let matches = SearchMatcher::find_all("foo bar foo baz foo", "foo");
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].col, 0);
    assert_eq!(matches[1].col, 8);
    assert_eq!(matches[2].col, 16);
}

#[test]
fn test_multiple_lines() {
    let content = "line one\nline two\nline three";
    let matches = SearchMatcher::find_all(content, "line");
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].line, 0);
    assert_eq!(matches[1].line, 1);
    assert_eq!(matches[2].line, 2);
}

#[test]
fn test_unicode_content() {
    let content = "héllo wörld";
    let matches = SearchMatcher::find_all(content, "wörld");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 6); // Character position, not byte
}

#[test]
fn test_no_match() {
    let matches = SearchMatcher::find_all("hello world", "xyz");
    assert!(matches.is_empty());
}

// Feature: search-in-results, Property 5: Case-insensitive matching
// *For any* search query and results content, the matcher should find the
// same matches regardless of case differences between query and content.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_case_insensitive_matching(
        // Generate content with mixed case
        content in "[a-zA-Z0-9 \n]{1,200}",
        // Generate a query that's a substring we'll inject
        query in "[a-zA-Z]{1,10}",
    ) {
        // Skip empty queries (already tested separately)
        prop_assume!(!query.is_empty());

        // Create content with the query in different cases
        let content_with_lower = format!("{} {}", content, query.to_lowercase());
        let content_with_upper = format!("{} {}", content, query.to_uppercase());
        let content_with_mixed = format!("{} {}", content, query);

        // Search with lowercase query
        let matches_lower_query = SearchMatcher::find_all(&content_with_lower, &query.to_lowercase());
        let matches_upper_query = SearchMatcher::find_all(&content_with_lower, &query.to_uppercase());

        // Both should find at least the injected match
        prop_assert!(
            !matches_lower_query.is_empty(),
            "Lowercase query should find matches in content with lowercase text"
        );
        prop_assert!(
            !matches_upper_query.is_empty(),
            "Uppercase query should find matches in content with lowercase text"
        );

        // The number of matches should be the same regardless of query case
        prop_assert_eq!(
            matches_lower_query.len(),
            matches_upper_query.len(),
            "Match count should be same regardless of query case"
        );

        // Also verify with uppercase content
        let matches_in_upper = SearchMatcher::find_all(&content_with_upper, &query.to_lowercase());
        prop_assert!(
            !matches_in_upper.is_empty(),
            "Lowercase query should find matches in uppercase content"
        );

        // And mixed case content
        let matches_in_mixed = SearchMatcher::find_all(&content_with_mixed, &query.to_lowercase());
        prop_assert!(
            !matches_in_mixed.is_empty(),
            "Lowercase query should find matches in mixed case content"
        );
    }
}

// Lowercasing `İ` (U+0130) grows it from 2 to 3 bytes. Before the fix the
// byte offset found in the lowered line was used to slice the original
// line, which panicked here ("not a char boundary ... inside 'é'").
#[test]
fn test_length_changing_lowercase_does_not_panic() {
    let content = r#"  "k": "İxé""#;
    let matches = SearchMatcher::find_all(content, "é");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 10);
    assert_eq!(matches[0].len, 1);
}

#[test]
fn test_kelvin_sign_content_does_not_panic() {
    let content = "\u{212A}é";
    let matches = SearchMatcher::find_all(content, "é");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 1);
}

// `ẞ` (U+1E9E) shrinks from 3 bytes to 2 when lowercased.
#[test]
fn test_shrinking_lowercase_does_not_panic() {
    let content = r#"  "k": "ẞabc""#;
    let matches = SearchMatcher::find_all(content, "abc");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 9);
    assert_eq!(matches[0].len, 3);
}

// Before the fix a length-changing character earlier on the line shifted
// every later column by one, so the highlight landed on the wrong glyph.
#[test]
fn test_column_after_length_changing_char() {
    let content = r#"  "İstanbul": "value","#;
    let matches = SearchMatcher::find_all(content, "value");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 15);
    assert_eq!(matches[0].len, 5);
}

// Searching for a character that itself changes length: the reported
// span must cover the original character, not the query's char count.
#[test]
fn test_match_len_covers_original_chars() {
    let content = "aİb";
    let matches = SearchMatcher::find_all(content, "İ");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 1);
    assert_eq!(matches[0].len, 1);
}

#[test]
fn test_uppercase_query_on_lowercase_unicode_content() {
    let content = "straße";
    let matches = SearchMatcher::find_all(content, "STRASSE");
    assert!(matches.is_empty());
    let matches = SearchMatcher::find_all(content, "STRAßE");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].col, 0);
    assert_eq!(matches[0].len, 6);
}

// Every match must describe a real span of the original line: column and
// length in characters, both in range, with a non-empty length.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_unicode_matches_are_in_bounds(
        content in "[a-zA-Z İıẞßKéÉΣσς\\u{212A} ]{0,40}",
        query in "[a-zA-Z İıẞßKéÉΣσς\\u{212A}]{1,4}",
    ) {
        let matches = SearchMatcher::find_all(&content, &query);
        let chars: Vec<char> = content.chars().collect();
        for m in matches {
            let start = m.col as usize;
            let end = start + m.len as usize;
            prop_assert!(end <= chars.len(), "span {start}..{end} exceeds {} chars", chars.len());
            prop_assert!(m.len >= 1);
        }
    }
}
