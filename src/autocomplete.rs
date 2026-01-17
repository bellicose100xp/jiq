pub mod autocomplete_render;
pub mod autocomplete_state;
mod brace_tracker;
mod context;
pub mod insertion;
pub mod jq_functions;
pub mod json_navigator;
pub mod path_parser;
mod result_analyzer;
mod scan_state;
mod variable_extractor;

#[cfg(test)]
#[path = "autocomplete/insertion_tests.rs"]
mod insertion_tests;

#[cfg(test)]
#[path = "autocomplete/path_parser_tests.rs"]
mod path_parser_tests;

#[cfg(test)]
#[path = "autocomplete/json_navigator_tests.rs"]
mod json_navigator_tests;

pub use brace_tracker::BraceTracker;

#[allow(unused_imports)]
pub use autocomplete_state::{
    AutocompleteState, JsonFieldType, MAX_VISIBLE_SUGGESTIONS, Suggestion, SuggestionType,
    update_suggestions_from_app,
};
pub use context::{SuggestionContext, analyze_context, get_suggestions};
pub use insertion::insert_suggestion_from_app;

use crate::query::ResultType;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

pub const MIN_CHARS_FOR_AUTOCOMPLETE: usize = 1;

#[allow(clippy::too_many_arguments)]
pub fn update_suggestions(
    autocomplete: &mut AutocompleteState,
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    original_json: Option<Arc<Value>>,
    all_field_names: Arc<HashSet<String>>,
    brace_tracker: &BraceTracker,
) {
    if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
        autocomplete.hide();
        return;
    }

    let suggestions = get_suggestions(
        query,
        cursor_pos,
        result_parsed,
        result_type,
        original_json,
        all_field_names,
        brace_tracker,
    );
    autocomplete.update_suggestions(suggestions);
}
