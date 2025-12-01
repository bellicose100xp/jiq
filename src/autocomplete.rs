mod context;
pub mod insertion;
pub mod jq_functions;
pub mod autocomplete_render;
mod result_analyzer;
pub mod autocomplete_state;

pub use context::{analyze_context, find_char_before_field_access, get_suggestions, SuggestionContext};
// JsonFieldType is part of public API for Suggestion struct
#[allow(unused_imports)]
pub use autocomplete_state::{AutocompleteState, JsonFieldType, Suggestion, SuggestionType};

use crate::query::ResultType;

/// Minimum characters required before showing autocomplete suggestions
/// Performance optimization to avoid showing suggestions for very short queries
pub const MIN_CHARS_FOR_AUTOCOMPLETE: usize = 1;

/// Update autocomplete suggestions based on query context
///
/// This function analyzes the current query and cursor position to generate
/// relevant autocomplete suggestions. It uses the cached result from the
/// last successful query execution to provide field suggestions.
///
/// # Arguments
/// * `autocomplete` - The autocomplete state to update
/// * `query` - The current query text
/// * `cursor_pos` - The cursor column position
/// * `result` - Optional unformatted result from last successful query
/// * `result_type` - Optional type of the result (Object, Array, etc.)
pub fn update_suggestions(
    autocomplete: &mut AutocompleteState,
    query: &str,
    cursor_pos: usize,
    result: Option<&str>,
    result_type: Option<ResultType>,
) {
    // Performance optimization: only show autocomplete for non-empty queries
    if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
        autocomplete.hide();
        return;
    }

    // Get suggestions based on unformatted query result (no ANSI codes)
    let suggestions = get_suggestions(query, cursor_pos, result, result_type);

    // Update autocomplete state
    autocomplete.update_suggestions(suggestions);
}
