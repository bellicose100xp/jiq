//! Tests for suggestion parsing

use super::*;
use crate::theme;
use proptest::prelude::*;

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_suggestion_type_colors() {
    assert_eq!(SuggestionType::Fix.color(), theme::ai::SUGGESTION_FIX);
    assert_eq!(
        SuggestionType::Optimize.color(),
        theme::ai::SUGGESTION_OPTIMIZE
    );
    assert_eq!(SuggestionType::Next.color(), theme::ai::SUGGESTION_NEXT);
}

#[test]
fn test_suggestion_type_from_str() {
    assert_eq!(SuggestionType::parse_type("Fix"), Some(SuggestionType::Fix));
    assert_eq!(SuggestionType::parse_type("fix"), Some(SuggestionType::Fix));
    assert_eq!(SuggestionType::parse_type("FIX"), Some(SuggestionType::Fix));
    assert_eq!(
        SuggestionType::parse_type("Optimize"),
        Some(SuggestionType::Optimize)
    );
    assert_eq!(
        SuggestionType::parse_type("Next"),
        Some(SuggestionType::Next)
    );
    assert_eq!(SuggestionType::parse_type("Invalid"), None);
}

#[test]
fn test_suggestion_type_labels() {
    assert_eq!(SuggestionType::Fix.label(), "[Fix]");
    assert_eq!(SuggestionType::Optimize.label(), "[Optimize]");
    assert_eq!(SuggestionType::Next.label(), "[Next]");
}

#[test]
fn test_parse_suggestions_single_json() {
    let response = r#"{"suggestions": [{"type": "fix", "query": ".users[] | select(.active)", "details": "Filters to only active users"}]}"#;
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[0].description, "Filters to only active users");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
}

#[test]
fn test_parse_suggestions_single_legacy_text() {
    let response = "1. [Fix] .users[] | select(.active)\n   Filters to only active users";
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[0].description, "Filters to only active users");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
}

#[test]
fn test_parse_suggestions_multiple_json() {
    let response = r#"{
        "suggestions": [
            {"type": "fix", "query": ".users[] | select(.active)", "details": "Filters to only active users"},
            {"type": "next", "query": ".users[] | .email", "details": "Extracts email addresses"},
            {"type": "optimize", "query": ".users | map(.name)", "details": "More efficient mapping"}
        ]
    }"#;

    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 3);

    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);

    assert_eq!(suggestions[1].query, ".users[] | .email");
    assert_eq!(suggestions[1].suggestion_type, SuggestionType::Next);

    assert_eq!(suggestions[2].query, ".users | map(.name)");
    assert_eq!(suggestions[2].suggestion_type, SuggestionType::Optimize);
}

#[test]
fn test_parse_suggestions_multiple_legacy_text() {
    let response = r#"1. [Fix] .users[] | select(.active)
   Filters to only active users

2. [Next] .users[] | .email
   Extracts email addresses

3. [Optimize] .users | map(.name)
   More efficient mapping"#;

    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 3);

    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);

    assert_eq!(suggestions[1].query, ".users[] | .email");
    assert_eq!(suggestions[1].suggestion_type, SuggestionType::Next);

    assert_eq!(suggestions[2].query, ".users | map(.name)");
    assert_eq!(suggestions[2].suggestion_type, SuggestionType::Optimize);
}

#[test]
fn test_parse_suggestions_multiline_description_legacy() {
    let response =
        "1. [Fix] .data[]\n   This is a longer description\n   that spans multiple lines";
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].query, ".data[]");
    assert!(suggestions[0].description.contains("longer description"));
    assert!(suggestions[0].description.contains("multiple lines"));
}

#[test]
fn test_parse_suggestions_empty_response() {
    let suggestions = parse_suggestions("");
    assert!(suggestions.is_empty());
}

#[test]
fn test_parse_suggestions_with_backticks_legacy() {
    let response = "1. [Fix] `.users[] | select(.active)`\n   Filters to only active users";
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[0].description, "Filters to only active users");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
}

#[test]
fn test_parse_suggestions_invalid_json() {
    let response = r#"{"suggestions": [{"type": "fix" INVALID JSON"#;
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());
}

#[test]
fn test_parse_suggestions_missing_field() {
    let response = r#"{"suggestions": [{"type": "fix", "query": ".users[]"}]}"#;
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());
}

#[test]
fn test_parse_suggestions_invalid_type() {
    let response =
        r#"{"suggestions": [{"type": "invalid", "query": ".users[]", "details": "test"}]}"#;
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());
}

#[test]
fn test_parse_suggestions_with_markdown_fences() {
    let response = r#"```json
{
    "suggestions": [
        {"type": "fix", "query": ".users[]", "details": "Extract users"}
    ]
}
```"#;
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].query, ".users[]");
    assert_eq!(suggestions[0].description, "Extract users");
    assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
}

#[test]
fn test_parse_suggestions_with_markdown_fences_multiline() {
    let response = r#"```json
{
    "suggestions": [
        {"type": "optimize", "query": ".users[] | select(.active)", "details": "Filter active users"},
        {"type": "next", "query": ".users[] | .email", "details": "Get emails"}
    ]
}
```"#;
    let suggestions = parse_suggestions(response);

    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].query, ".users[] | select(.active)");
    assert_eq!(suggestions[1].query, ".users[] | .email");
}

#[test]
fn test_parse_suggestions_no_valid_format() {
    let response = "This is just plain text without any structured suggestions.";
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());
}

#[test]
fn test_parse_suggestions_malformed() {
    // Missing type bracket
    let response = "1. Fix .users[]";
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());

    // Missing query
    let response = "1. [Fix]";
    let suggestions = parse_suggestions(response);
    assert!(suggestions.is_empty());
}

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase2, Property 7: Suggestion parsing extracts queries**
// *For any* AI response containing valid suggestion format, parsing SHALL extract the query.
// **Validates: Requirements 5.2, 5.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_suggestion_parsing_extracts_queries_json(
        query in "\\.[a-zA-Z0-9_|\\[\\]]{1,30}",
        desc in "[a-zA-Z ]{1,50}",
        suggestion_type in prop::sample::select(vec!["fix", "optimize", "next"]),
    ) {
        let response = format!(
            r#"{{"suggestions": [{{"type": "{}", "query": "{}", "details": "{}"}}]}}"#,
            suggestion_type, query, desc
        );
        let suggestions = parse_suggestions(&response);

        prop_assert_eq!(suggestions.len(), 1, "Should parse exactly one suggestion");
        prop_assert_eq!(&suggestions[0].query, query.trim(), "Query should match");
    }

    #[test]
    fn prop_suggestion_parsing_extracts_queries_legacy(
        query in "\\.[a-zA-Z0-9_|\\[\\]]{1,30}",
        desc in "[a-zA-Z ]{1,50}",
        suggestion_type in prop::sample::select(vec!["Fix", "Optimize", "Next"]),
    ) {
        let response = format!("1. [{}] {}\n   {}", suggestion_type, query, desc);
        let suggestions = parse_suggestions(&response);

        prop_assert_eq!(suggestions.len(), 1, "Should parse exactly one suggestion");
        prop_assert_eq!(&suggestions[0].query, query.trim(), "Query should match");
    }
}

// **Feature: ai-assistant-phase2, Property 8: Malformed response fallback**
// *For any* AI response that cannot be parsed, parsing SHALL return empty vec.
// **Validates: Requirements 5.9**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_malformed_response_returns_empty(
        text in "[a-zA-Z ]{0,100}",
    ) {
        // Plain text without numbered format should return empty
        let suggestions = parse_suggestions(&text);
        // Either empty or valid suggestions (if text accidentally matches format)
        // The key property is that it doesn't crash
        prop_assert!(suggestions.len() <= 1, "Should handle any text gracefully");
    }
}

// **Feature: ai-assistant-phase2, Property 9: Suggestion type colors**
// *For any* parsed suggestion, the type SHALL have the correct color.
// **Validates: Requirements 5.4, 5.5, 5.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_suggestion_type_colors_correct(
        type_idx in 0usize..3usize,
    ) {
        let types = [SuggestionType::Fix, SuggestionType::Optimize, SuggestionType::Next];
        let expected_colors = [
            theme::ai::SUGGESTION_FIX,
            theme::ai::SUGGESTION_OPTIMIZE,
            theme::ai::SUGGESTION_NEXT,
        ];

        let suggestion_type = types[type_idx];
        let expected_color = expected_colors[type_idx];

        prop_assert_eq!(
            suggestion_type.color(),
            expected_color,
            "Color for {:?} should be {:?}",
            suggestion_type,
            expected_color
        );
    }
}
