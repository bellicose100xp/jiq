//! Bracket pair matching for syntax highlighting.
//!
//! This module provides functionality to find matching bracket pairs at the cursor position.
//! It supports parentheses (), square brackets [], and curly braces {}.

use crate::editor::mode::TextObjectScope;
use crate::editor::text_objects::find_bracket_bounds;

/// Finds matching bracket pair positions when cursor is on a bracket.
///
/// Returns `Some((open_pos, close_pos))` if cursor is on a bracket character
/// and a matching pair exists. Returns `None` if cursor is not on a bracket
/// or no matching bracket is found.
///
/// # Parameters
/// - `query`: The query string to search in
/// - `cursor_pos`: Current cursor position (0-indexed character position)
///
/// # Returns
/// - `Some((open_pos, close_pos))`: Positions of opening and closing brackets
/// - `None`: Cursor not on bracket or no match found
///
/// # Examples
/// ```
/// use jiq::syntax_highlight::bracket_matcher::find_matching_bracket;
///
/// let query = "map(.)";
/// let result = find_matching_bracket(query, 3); // cursor on '('
/// assert_eq!(result, Some((3, 5))); // positions of '(' and ')'
/// ```
pub fn find_matching_bracket(query: &str, cursor_pos: usize) -> Option<(usize, usize)> {
    let char_at_cursor = query.chars().nth(cursor_pos)?;

    let (open_delim, close_delim) = match char_at_cursor {
        '(' | ')' => ('(', ')'),
        '[' | ']' => ('[', ']'),
        '{' | '}' => ('{', '}'),
        _ => return None,
    };

    let bounds = find_bracket_bounds(
        query,
        cursor_pos,
        open_delim,
        close_delim,
        TextObjectScope::Around,
    )?;
    Some((bounds.0, bounds.1.saturating_sub(1)))
}

#[cfg(test)]
#[path = "bracket_matcher_tests.rs"]
mod bracket_matcher_tests;
