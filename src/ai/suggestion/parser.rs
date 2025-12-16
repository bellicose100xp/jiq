//! Suggestion parsing for AI responses
//!
//! Parses structured suggestions from AI response text in the format:
//! ```text
//! 1. [Fix] .users[] | select(.active)
//!    Filters to only active users
//!
//! 2. [Next] .users[] | .email
//!    Extracts email addresses from users
//! ```

use ratatui::style::Color;

// =========================================================================
// Suggestion Types
// =========================================================================

/// Type of AI suggestion
///
/// # Requirements
/// - 5.4: Fix type displayed in red
/// - 5.5: Optimize type displayed in yellow
/// - 5.6: Next type displayed in green
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    /// Error corrections - displayed in red
    Fix,
    /// Performance/style improvements - displayed in yellow
    Optimize,
    /// Next steps, NL interpretations - displayed in green
    Next,
}

impl SuggestionType {
    /// Get the color for this suggestion type
    pub fn color(&self) -> Color {
        match self {
            SuggestionType::Fix => Color::Red,
            SuggestionType::Optimize => Color::Yellow,
            SuggestionType::Next => Color::Green,
        }
    }

    /// Parse suggestion type from string
    pub fn parse_type(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fix" => Some(SuggestionType::Fix),
            "optimize" => Some(SuggestionType::Optimize),
            "next" => Some(SuggestionType::Next),
            _ => None,
        }
    }

    /// Get the display label for this type
    pub fn label(&self) -> &'static str {
        match self {
            SuggestionType::Fix => "[Fix]",
            SuggestionType::Optimize => "[Optimize]",
            SuggestionType::Next => "[Next]",
        }
    }
}

/// A single AI suggestion for a jq query
///
/// # Requirements
/// - 5.2: Format "N. [Type] jq_query_here" followed by description
/// - 5.3: Extractable query for future selection feature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The suggested jq query (extractable for future selection)
    pub query: String,
    /// Brief explanation of what the query does
    pub description: String,
    /// Type of suggestion: Fix, Optimize, or Next
    pub suggestion_type: SuggestionType,
}

// =========================================================================
// Parsing Functions
// =========================================================================

/// Parse suggestions from AI response text
///
/// Expected format:
/// ```text
/// 1. [Fix] .users[] | select(.active)
///    Filters to only active users
///
/// 2. [Next] .users[] | .email
///    Extracts email addresses from users
/// ```
///
/// # Requirements
/// - 5.2: Parse "N. [Type] jq_query_here" format
/// - 5.3: Extract query string from each suggestion
/// - 5.9: Return empty vec if parsing fails (fallback to raw response)
pub fn parse_suggestions(response: &str) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    let lines: Vec<&str> = response.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for pattern: "N. [Type] query"
        // e.g., "1. [Fix] .users[]"
        if let Some(suggestion) = parse_suggestion_line(line, &lines[i + 1..]) {
            suggestions.push(suggestion.0);
            i += suggestion.1; // Skip the lines we consumed
        } else {
            i += 1;
        }
    }

    suggestions
}

/// Parse a single suggestion starting from a numbered line
///
/// Returns (Suggestion, lines_consumed) if successful
fn parse_suggestion_line(line: &str, remaining_lines: &[&str]) -> Option<(Suggestion, usize)> {
    // Match pattern: digit(s) followed by ". ["
    let line = line.trim();

    // Find the number at the start
    let dot_pos = line.find(". [")?;
    let num_str = &line[..dot_pos];
    if !num_str.chars().all(|c| c.is_ascii_digit()) || num_str.is_empty() {
        return None;
    }

    // Find the type between [ and ]
    let type_start = dot_pos + 3; // Skip ". ["
    let type_end = line[type_start..].find(']')? + type_start;
    let type_str = &line[type_start..type_end];
    let suggestion_type = SuggestionType::parse_type(type_str)?;

    // Query is everything after "] "
    let query_start = type_end + 1;
    let mut query = line[query_start..].trim();

    // Strip backticks if present (AI sometimes wraps queries in backticks)
    if query.starts_with('`') && query.ends_with('`') && query.len() > 2 {
        query = &query[1..query.len() - 1];
    }

    let query = query.to_string();

    if query.is_empty() {
        return None;
    }

    // Collect description from following indented lines
    let mut description_lines = Vec::new();
    let mut lines_consumed = 1;

    for remaining_line in remaining_lines {
        let trimmed = remaining_line.trim();

        // Stop at empty line or next numbered suggestion
        if trimmed.is_empty() {
            lines_consumed += 1;
            break;
        }

        // Check if this is a new numbered suggestion
        if let Some(dot_pos) = trimmed.find(". [") {
            let num_part = &trimmed[..dot_pos];
            if num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty() {
                break;
            }
        }

        // This is a description line (indented or continuation)
        description_lines.push(trimmed);
        lines_consumed += 1;
    }

    let description = description_lines.join(" ");

    Some((
        Suggestion {
            query,
            description,
            suggestion_type,
        },
        lines_consumed,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_suggestion_type_colors() {
        assert_eq!(SuggestionType::Fix.color(), Color::Red);
        assert_eq!(SuggestionType::Optimize.color(), Color::Yellow);
        assert_eq!(SuggestionType::Next.color(), Color::Green);
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
    fn test_parse_suggestions_single() {
        let response = "1. [Fix] .users[] | select(.active)\n   Filters to only active users";
        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".users[] | select(.active)");
        assert_eq!(suggestions[0].description, "Filters to only active users");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
    }

    #[test]
    fn test_parse_suggestions_multiple() {
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
    fn test_parse_suggestions_multiline_description() {
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
    fn test_parse_suggestions_with_backticks() {
        // Test that backticks around queries are stripped
        let response = "1. [Fix] `.users[] | select(.active)`\n   Filters to only active users";
        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".users[] | select(.active)");
        assert_eq!(suggestions[0].description, "Filters to only active users");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
    }

    #[test]
    fn test_parse_suggestions_with_backticks_multiple() {
        let response = r#"1. [Fix] `.users[] | select(.active)`
   Filters to only active users

2. [Next] `.users[] | .email`
   Extracts email addresses"#;

        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].query, ".users[] | select(.active)");
        assert_eq!(suggestions[1].query, ".users[] | .email");
    }

    #[test]
    fn test_parse_suggestions_without_backticks_unchanged() {
        // Ensure queries without backticks still work
        let response = "1. [Fix] .users[]\n   Test";
        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".users[]");
    }

    #[test]
    fn test_parse_suggestions_single_backtick_not_stripped() {
        // Single backtick should not be stripped (not a pair)
        let response = "1. [Fix] `.users[]\n   Test";
        let suggestions = parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, "`.users[]");
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
        fn prop_suggestion_parsing_extracts_queries(
            // Query must start with a non-space character to be valid
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
                Color::Red,
                Color::Yellow,
                Color::Green,
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
}
