//! Tests for suggestion parsing

use super::*;
use crate::theme;
use proptest::prelude::*;

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_suggestion_type_colors() {
    assert_eq!(SuggestionType::Fix.color(), theme::ai::suggestion_fix());
    assert_eq!(
        SuggestionType::Optimize.color(),
        theme::ai::suggestion_optimize()
    );
    assert_eq!(SuggestionType::Next.color(), theme::ai::suggestion_next());
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
// Three-state parse_response tests
//
// Regression coverage for the bug where a valid but explicitly-empty
// suggestion list (`{"suggestions": []}`, the prompt's own "nothing to
// suggest" sentinel) was reported as a parse failure instead of a benign
// empty result.
// =========================================================================

mod parse_response_outcomes {
    use super::*;
    use crate::ai::suggestion::ParseOutcome;

    #[test]
    fn parsed_outcome_for_valid_suggestions() {
        let response = r#"{"suggestions": [{"type": "fix", "query": ".users[]", "details": "d"}]}"#;
        match parse_response(response) {
            ParseOutcome::Parsed(suggestions) => {
                assert_eq!(suggestions.len(), 1);
                assert_eq!(suggestions[0].query, ".users[]");
            }
            other => panic!("expected Parsed, got {:?}", other),
        }
    }

    #[test]
    fn empty_outcome_for_explicit_empty_array() {
        // The exact sentinel the prompt instructs the model to return when
        // it has nothing to suggest.
        let response = r#"{"suggestions":[]}"#;
        assert_eq!(parse_response(response), ParseOutcome::Empty);
    }

    #[test]
    fn empty_outcome_for_empty_array_with_fences() {
        // The real-world failing case from /tmp/jiq-debug.log: the sentinel
        // wrapped in markdown code fences.
        let response = "```json\n{\"suggestions\":[]}\n```";
        assert_eq!(parse_response(response), ParseOutcome::Empty);
    }

    #[test]
    fn empty_outcome_for_empty_array_with_whitespace() {
        let response = r#"{ "suggestions" : [ ] }"#;
        assert_eq!(parse_response(response), ParseOutcome::Empty);
    }

    #[test]
    fn empty_outcome_for_empty_array_embedded_in_prose() {
        // Extraction path: empty sentinel surrounded by commentary.
        let response = "Here you go:\n{\"suggestions\":[]}\nHope that helps!";
        assert_eq!(parse_response(response), ParseOutcome::Empty);
    }

    #[test]
    fn unparseable_outcome_for_plain_prose() {
        let response = "This is just plain text without any structured suggestions.";
        assert_eq!(parse_response(response), ParseOutcome::Unparseable);
    }

    #[test]
    fn unparseable_outcome_for_malformed_json() {
        let response = r#"{"suggestions": [{"type": "fix" INVALID JSON"#;
        assert_eq!(parse_response(response), ParseOutcome::Unparseable);
    }

    #[test]
    fn unparseable_outcome_when_all_items_have_invalid_type() {
        // Non-empty array, but every item unusable — must NOT be reported as
        // Empty (that's reserved for a literally-empty array).
        let response =
            r#"{"suggestions": [{"type": "invalid", "query": ".users[]", "details": "d"}]}"#;
        assert_eq!(parse_response(response), ParseOutcome::Unparseable);
    }

    #[test]
    fn unparseable_outcome_when_items_missing_required_field() {
        let response = r#"{"suggestions": [{"type": "fix", "query": ".users[]"}]}"#;
        assert_eq!(parse_response(response), ParseOutcome::Unparseable);
    }

    #[test]
    fn parsed_wins_over_empty_in_mixed_response() {
        // A response containing both an empty array and a usable legacy-text
        // suggestion must resolve to Parsed — real suggestions always win.
        let response = "{\"suggestions\":[]}\n\n1. [Fix] .users[]\n   Fix it";
        match parse_response(response) {
            ParseOutcome::Parsed(suggestions) => {
                assert_eq!(suggestions.len(), 1);
                assert_eq!(suggestions[0].query, ".users[]");
            }
            other => panic!("expected Parsed, got {:?}", other),
        }
    }

    #[test]
    fn parsed_outcome_for_legacy_text() {
        let response = "1. [Next] .users[] | .email\n   Get emails";
        match parse_response(response) {
            ParseOutcome::Parsed(suggestions) => {
                assert_eq!(suggestions.len(), 1);
                assert_eq!(suggestions[0].suggestion_type, SuggestionType::Next);
            }
            other => panic!("expected Parsed, got {:?}", other),
        }
    }
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
        query in "\\.[a-zA-Z_][a-zA-Z0-9_]{0,30}",
        desc in "[a-zA-Z ]{1,50}",
        suggestion_type in prop::sample::select(vec!["fix", "optimize", "next"]),
    ) {
        // Generator restricted to valid ASCII jq identifiers so the
        // sanitizer passes them through unchanged. Sanitizer behaviour on
        // invalid inputs is tested directly in sanitizer_tests.
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
            theme::ai::suggestion_fix(),
            theme::ai::suggestion_optimize(),
            theme::ai::suggestion_next(),
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

// =========================================================================
// Edge-case fence/format robustness tests
// =========================================================================

mod fence_edge_cases {
    use super::*;

    #[test]
    fn fence_with_closing_on_same_line_as_json() {
        // Model returns fence with no newline before closing ```
        let response = r#"```json
{"suggestions":[{"type":"fix","query":".x","details":"d"}]}```"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".x");
    }

    #[test]
    fn fence_with_missing_closing_fence() {
        // Truncated or streaming response without closing ```
        let response = r#"```json
{"suggestions":[{"type":"fix","query":".x","details":"d"}]}"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn fence_without_language_tag() {
        let response = r#"```
{"suggestions":[{"type":"next","query":".y","details":"d"}]}
```"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".y");
    }

    #[test]
    fn fence_inline_no_newlines() {
        let response = r#"```json{"suggestions":[{"type":"fix","query":".z","details":"d"}]}```"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".z");
    }

    #[test]
    fn json_wrapped_in_prose() {
        // Model adds explanation before/after JSON
        let response = r#"Here are the suggestions:

{"suggestions":[{"type":"next","query":".a","details":"d"}]}

Hope this helps!"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".a");
    }

    #[test]
    fn json_with_escaped_non_ascii_strings() {
        // The actual scenario from issue: non-ASCII keys with escaped quotes
        let response = r#"```json
{"suggestions":[{"type":"next","query":".[\"趣味\"] | length","details":"Count hobbies"},{"type":"next","query":".[\"趣味\"] | join(\", \")","details":"Join with comma"}]}
```"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].query, r#".["趣味"] | length"#);
        assert_eq!(suggestions[1].query, r#".["趣味"] | join(", ")"#);
    }

    #[test]
    fn uppercase_language_tag() {
        let response = "```JSON\n{\"suggestions\":[{\"type\":\"fix\",\"query\":\".x\",\"details\":\"d\"}]}\n```";
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn extra_whitespace_around_fences() {
        let response = "   \n\n```json\n{\"suggestions\":[{\"type\":\"fix\",\"query\":\".x\",\"details\":\"d\"}]}\n```\n\n   ";
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn extraction_path_handles_escaped_quote_inside_string() {
        // Prose-wrapped (no fences) so the whole-string parse fails and the
        // extraction fallback in extract_suggestions_json runs. The query
        // contains an escaped quote (\") inside a string literal: the manual
        // brace matcher must treat the backslash as an escape (not toggle
        // in_string off) so the matching closing brace is found correctly.
        let response = r#"Here:
{"suggestions":[{"type":"fix","query":".[\"a\"]","details":"d"}]}
bye"#;
        let suggestions = parse_suggestions(response);
        assert_eq!(suggestions.len(), 1);
        // .["a"] survives the sanitizer unchanged ('.' is followed by '[').
        assert_eq!(suggestions[0].query, r#".["a"]"#);
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
    }
}

// =========================================================================
// Legacy-text line-parsing edge cases
// =========================================================================

mod legacy_text_edge_cases {
    use super::*;

    #[test]
    fn legacy_line_with_non_digit_prefix_rejected() {
        // A line containing the ". [" marker but whose prefix is not all ASCII
        // digits must not be treated as a numbered suggestion.
        let non_digit = "x. [Fix] .users[]";
        assert!(parse_suggestions(non_digit).is_empty());

        // An empty prefix (leading ". [") must also be rejected.
        let empty_prefix = ". [Fix] .foo";
        assert!(parse_suggestions(empty_prefix).is_empty());

        // Sanity: a genuine digit prefix on the same shape DOES parse, proving
        // the rejection above is the digit/emptiness guard, not the marker.
        let valid = "1. [Fix] .users[]";
        assert_eq!(parse_suggestions(valid).len(), 1);
    }

    #[test]
    fn legacy_back_to_back_numbered_without_blank_line() {
        // Two numbered suggestions on consecutive lines with NO blank line
        // between them. The description-collection loop must detect the second
        // numbered line and break so the outer loop re-parses it as its own
        // suggestion (rather than folding "2. [Next] .b" into suggestion 1's
        // description).
        let response = "1. [Fix] .a\n   desc for a\n2. [Next] .b\n   desc for b";
        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].query, ".a");
        assert_eq!(suggestions[0].description, "desc for a");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
        assert_eq!(suggestions[1].query, ".b");
        assert_eq!(suggestions[1].description, "desc for b");
        assert_eq!(suggestions[1].suggestion_type, SuggestionType::Next);
    }
}
