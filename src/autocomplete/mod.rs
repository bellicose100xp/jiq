mod context;
mod jq_functions;
pub mod json_analyzer;
mod state;

pub use context::get_suggestions;
// JsonFieldType is part of public API for Suggestion struct
#[allow(unused_imports)]
pub use state::{AutocompleteState, JsonFieldType, SuggestionType};

#[cfg(test)]
pub use state::Suggestion;
