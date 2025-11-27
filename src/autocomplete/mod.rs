mod context;
mod jq_functions;
mod result_analyzer;
mod state;

pub use context::{analyze_context, find_char_before_field_access, get_suggestions, SuggestionContext};
// JsonFieldType is part of public API for Suggestion struct
#[allow(unused_imports)]
pub use state::{AutocompleteState, JsonFieldType, SuggestionType};

#[cfg(test)]
pub use state::Suggestion;
