//! Suggestion parsing for AI responses
//!
//! Parses structured suggestions from AI responses in JSON format:
//! ```json
//! {
//!   "suggestions": [
//!     {"type": "fix", "query": ".users[] | select(.active)", "details": "Filters to only active users"},
//!     {"type": "next", "query": ".users[] | .email", "details": "Extracts email addresses"}
//!   ]
//! }
//! ```

use ratatui::style::Color;
use serde::Deserialize;

use crate::theme;

// =========================================================================
// JSON Response Types
// =========================================================================

/// AI response wrapper containing suggestions array
#[derive(Deserialize, Debug)]
struct AiResponse {
    suggestions: Vec<JsonSuggestion>,
}

/// Single suggestion in JSON format from AI
#[derive(Deserialize, Debug)]
struct JsonSuggestion {
    #[serde(rename = "type")]
    suggestion_type: String,
    query: String,
    details: String,
}

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
            SuggestionType::Fix => theme::ai::SUGGESTION_FIX,
            SuggestionType::Optimize => theme::ai::SUGGESTION_OPTIMIZE,
            SuggestionType::Next => theme::ai::SUGGESTION_NEXT,
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

/// Parse suggestions from AI response
///
/// Tries JSON format first, falls back to legacy text format
///
/// # Requirements
/// - 5.2: Parse suggestions from AI responses
/// - 5.3: Extract query string from each suggestion
/// - 5.9: Return empty vec if parsing fails (fallback to raw response)
pub fn parse_suggestions(response: &str) -> Vec<Suggestion> {
    // Strip markdown code fences if present
    let cleaned_response = strip_markdown_fences(response);

    // Try JSON format first
    if let Ok(parsed) = parse_suggestions_json(&cleaned_response)
        && !parsed.is_empty()
    {
        return parsed;
    }

    // Fallback to legacy text format
    parse_suggestions_text(response)
}

/// Strip markdown code fences from response
///
/// Handles patterns like:
/// ```json
/// {"suggestions": [...]}
/// ```
fn strip_markdown_fences(response: &str) -> String {
    let trimmed = response.trim();

    // Check for markdown code fence pattern
    if let Some(first_newline) = trimmed.find('\n') {
        let first_line = &trimmed[..first_newline];

        // Check if it starts with ```json or ```
        if first_line.trim().starts_with("```") {
            // Find the closing ```
            let content_start = first_newline + 1;
            if let Some(closing_fence_pos) = trimmed[content_start..].rfind("\n```") {
                let content_end = content_start + closing_fence_pos;
                return trimmed[content_start..content_end].trim().to_string();
            }
        }
    }

    // No code fences found, return original
    response.to_string()
}

/// Parse suggestions from JSON format
///
/// Expected format:
/// ```json
/// {
///   "suggestions": [
///     {"type": "fix", "query": ".users[]", "details": "Description"}
///   ]
/// }
/// ```
fn parse_suggestions_json(response: &str) -> Result<Vec<Suggestion>, serde_json::Error> {
    let ai_response: AiResponse = serde_json::from_str(response)?;

    let suggestions = ai_response
        .suggestions
        .into_iter()
        .filter_map(|json_sugg| {
            let suggestion_type = SuggestionType::parse_type(&json_sugg.suggestion_type)?;
            Some(Suggestion {
                query: json_sugg.query,
                description: json_sugg.details,
                suggestion_type,
            })
        })
        .collect();

    Ok(suggestions)
}

/// Parse suggestions from legacy text format
///
/// Expected format:
/// ```text
/// 1. [Fix] .users[] | select(.active)
///    Filters to only active users
///
/// 2. [Next] .users[] | .email
///    Extracts email addresses from users
/// ```
fn parse_suggestions_text(response: &str) -> Vec<Suggestion> {
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
#[path = "parser_tests.rs"]
mod parser_tests;
