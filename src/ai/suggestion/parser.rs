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

/// Classification of an AI response after parsing.
///
/// Distinguishes the three outcomes a response can have so the UI can react
/// appropriately instead of collapsing "the model had nothing to suggest"
/// into a parse error:
/// - [`ParseOutcome::Parsed`] - one or more usable suggestions extracted.
/// - [`ParseOutcome::Empty`] - the model explicitly returned an empty
///   suggestion list (`{"suggestions": []}`). This is the prompt's own
///   "no suggestions" sentinel: a valid, benign response, NOT a failure.
/// - [`ParseOutcome::Unparseable`] - the response did not match any expected
///   format (genuine malformed / non-compliant output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome {
    /// Response yielded at least one usable suggestion.
    Parsed(Vec<Suggestion>),
    /// Response was a valid, explicitly-empty suggestion list.
    Empty,
    /// Response did not match any expected format.
    Unparseable,
}

/// Outcome of attempting to parse a single JSON candidate string.
enum JsonParse {
    /// Parsed into at least one usable suggestion.
    Suggestions(Vec<Suggestion>),
    /// Valid JSON whose `suggestions` array was literally empty.
    EmptyArray,
    /// Not valid JSON of the expected shape, or every item was unusable.
    NotJson,
}

// =========================================================================
// Parsing Functions
// =========================================================================

/// Parse an AI response into a three-state [`ParseOutcome`].
///
/// Tries the strict JSON format first (fence-stripped, then via embedded-object
/// extraction), then falls back to the legacy numbered-text format. The key
/// distinction over [`parse_suggestions`] is that a valid but explicitly-empty
/// suggestion list (`{"suggestions": []}`) is reported as [`ParseOutcome::Empty`]
/// rather than being indistinguishable from unparseable garbage — so the UI can
/// show a calm "no suggestions" message instead of a parse-error banner.
///
/// # Requirements
/// - 5.2: Parse suggestions from AI responses
/// - 5.3: Extract query string from each suggestion
/// - 5.9: Distinguish parsed / empty / unparseable for the caller
pub fn parse_response(response: &str) -> ParseOutcome {
    // Track whether any candidate was a valid-but-empty suggestion list. We
    // only conclude `Empty` if NO usable suggestions turn up in any format —
    // genuine suggestions always win over the empty sentinel, preserving the
    // original parse priority.
    let mut saw_empty = false;

    // Try JSON format first — on the fence-stripped text.
    let cleaned_response = strip_markdown_fences(response);
    match try_parse_json(&cleaned_response) {
        JsonParse::Suggestions(parsed) => return ParseOutcome::Parsed(parsed),
        JsonParse::EmptyArray => saw_empty = true,
        JsonParse::NotJson => {}
    }

    // Last-resort: scan for a JSON object containing "suggestions" anywhere
    // in the response (handles extra prose, broken fences, streaming artifacts).
    if let Some(extracted) = extract_suggestions_json(response) {
        match try_parse_json(&extracted) {
            JsonParse::Suggestions(parsed) => return ParseOutcome::Parsed(parsed),
            JsonParse::EmptyArray => saw_empty = true,
            JsonParse::NotJson => {}
        }
    }

    // Fallback to legacy text format.
    let text_suggestions = parse_suggestions_text(response);
    if !text_suggestions.is_empty() {
        return ParseOutcome::Parsed(text_suggestions);
    }

    // No usable suggestions in any format. An explicit empty-suggestions
    // sentinel means "the model had nothing to suggest" (benign); anything
    // else is genuinely unparseable.
    if saw_empty {
        ParseOutcome::Empty
    } else {
        ParseOutcome::Unparseable
    }
}

/// Parse suggestions from AI response, returning only the usable suggestions.
///
/// Thin wrapper over [`parse_response`] that flattens both [`ParseOutcome::Empty`]
/// and [`ParseOutcome::Unparseable`] to an empty vec. Test-only convenience for
/// the many cases that assert on extracted suggestions, not the failure mode.
#[cfg(test)]
pub fn parse_suggestions(response: &str) -> Vec<Suggestion> {
    match parse_response(response) {
        ParseOutcome::Parsed(suggestions) => suggestions,
        ParseOutcome::Empty | ParseOutcome::Unparseable => Vec::new(),
    }
}

/// Find a `{...}` JSON object in the response that contains the literal
/// `"suggestions"` key, and return it as a standalone string. Returns
/// `None` if no balanced object is found.
fn extract_suggestions_json(response: &str) -> Option<String> {
    let start = response.find(r#""suggestions""#)?;
    // Walk backwards from the key to the enclosing '{'
    let before = &response[..start];
    let obj_start = before.rfind('{')?;
    let tail = &response[obj_start..];

    // Find matching closing brace, respecting string literals
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in tail.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(tail[..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Strip markdown code fences from response
///
/// Handles patterns like:
/// ```json
/// {"suggestions": [...]}
/// ```
fn strip_markdown_fences(response: &str) -> String {
    let trimmed = response.trim();

    if !trimmed.starts_with("```") {
        return response.to_string();
    }

    // Strip leading fence (```json, ```JSON, ```, optionally followed by newline)
    let after_opening = trimmed.trim_start_matches("```");
    let after_lang = after_opening
        .split_once('\n')
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| {
            // No newline: strip alphanumeric lang tag like "json"
            after_opening.trim_start_matches(|c: char| c.is_ascii_alphanumeric())
        });

    // Strip trailing fence
    let body = after_lang.trim();
    let body = body.strip_suffix("```").unwrap_or(body);

    body.trim().to_string()
}

/// Attempt to parse a JSON candidate string into suggestions.
///
/// Returns a three-way [`JsonParse`] so the caller can tell apart:
/// - valid JSON with usable suggestions ([`JsonParse::Suggestions`]);
/// - valid JSON whose `suggestions` array was explicitly empty
///   ([`JsonParse::EmptyArray`]) — the model's "nothing to suggest" sentinel;
/// - anything else ([`JsonParse::NotJson`]): malformed JSON, wrong shape, or a
///   non-empty array whose every item was unusable (bad `type`, etc.).
///
/// Expected format:
/// ```json
/// {
///   "suggestions": [
///     {"type": "fix", "query": ".users[]", "details": "Description"}
///   ]
/// }
/// ```
fn try_parse_json(response: &str) -> JsonParse {
    let ai_response: AiResponse = match serde_json::from_str(response) {
        Ok(parsed) => parsed,
        Err(_) => return JsonParse::NotJson,
    };

    // A literally-empty array is the model's explicit "no suggestions"
    // sentinel — distinct from a non-empty array we failed to make sense of.
    if ai_response.suggestions.is_empty() {
        return JsonParse::EmptyArray;
    }

    let suggestions: Vec<Suggestion> = ai_response
        .suggestions
        .into_iter()
        .filter_map(|json_sugg| {
            let suggestion_type = SuggestionType::parse_type(&json_sugg.suggestion_type)?;
            Some(Suggestion {
                query: super::sanitizer::sanitize_jq_query(&json_sugg.query),
                description: json_sugg.details,
                suggestion_type,
            })
        })
        .collect();

    if suggestions.is_empty() {
        // The array had entries but none were usable (e.g. invalid `type`).
        // Treat as unparseable so callers can fall through to other formats.
        JsonParse::NotJson
    } else {
        JsonParse::Suggestions(suggestions)
    }
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
