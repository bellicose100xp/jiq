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

#[allow(unused_imports)]
pub use autocomplete_state::{
    AutocompleteState, JsonFieldType, Suggestion, SuggestionType, update_suggestions_from_app,
};
pub use context::{
    SuggestionContext, analyze_context, find_char_before_field_access, get_suggestions,
};
pub use insertion::insert_suggestion_from_app;

use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

pub const MIN_CHARS_FOR_AUTOCOMPLETE: usize = 1;

pub fn update_suggestions(
    autocomplete: &mut AutocompleteState,
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    brace_tracker: &BraceTracker,
) {
    if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
        autocomplete.hide();
        return;
    }

    let suggestions = get_suggestions(query, cursor_pos, result_parsed, result_type, brace_tracker);
    autocomplete.update_suggestions(suggestions);
}
