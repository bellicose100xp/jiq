pub mod autocomplete_render;
pub mod autocomplete_state;
mod brace_tracker;
mod context;
pub mod insertion;
pub mod jq_functions;
mod result_analyzer;
mod scan_state;

#[cfg(test)]
#[path = "autocomplete/insertion_tests.rs"]
mod insertion_tests;

pub use brace_tracker::BraceTracker;

pub use context::{
    SuggestionContext, analyze_context, find_char_before_field_access, get_suggestions,
};
// JsonFieldType is part of public API for Suggestion struct
#[allow(unused_imports)]
pub use autocomplete_state::{
    AutocompleteState, JsonFieldType, Suggestion, SuggestionType, update_suggestions_from_app,
};
pub use insertion::insert_suggestion_from_app;

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
/// * `brace_tracker` - The brace tracker for context detection
pub fn update_suggestions(
    autocomplete: &mut AutocompleteState,
    query: &str,
    cursor_pos: usize,
    result: Option<&str>,
    result_type: Option<ResultType>,
    brace_tracker: &BraceTracker,
) {
    // Performance optimization: only show autocomplete for non-empty queries
    if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
        autocomplete.hide();
        return;
    }

    // Get suggestions based on unformatted query result (no ANSI codes)
    let suggestions = get_suggestions(query, cursor_pos, result, result_type, brace_tracker);

    // Update autocomplete state
    autocomplete.update_suggestions(suggestions);
}
