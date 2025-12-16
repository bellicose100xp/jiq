//! Query execution utilities for autocomplete insertion

use crate::query::QueryState;
use tui_textarea::TextArea;

/// Execute query and update results
pub fn execute_query_and_update(textarea: &TextArea<'_>, query_state: &mut QueryState) {
    let query_text = textarea.lines()[0].clone();
    query_state.execute(&query_text);
}
