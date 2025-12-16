//! Autocomplete suggestion insertion logic
//!
//! This module handles inserting autocomplete suggestions into the query,
//! managing cursor positioning, and executing the updated query.

use tui_textarea::TextArea;

use crate::app::App;
use crate::autocomplete::autocomplete_state::Suggestion;
use crate::autocomplete::{SuggestionContext, analyze_context, find_char_before_field_access};
use crate::query::{CharType, QueryState};

#[cfg(debug_assertions)]
use log::debug;

// Re-export sub-module functions
pub use self::cursor::move_cursor_to_column;
pub use self::execution::execute_query_and_update;
pub use self::query_manipulation::extract_middle_query;

// Module declarations
#[path = "insertion/cursor.rs"]
mod cursor;
#[path = "insertion/execution.rs"]
mod execution;
#[path = "insertion/query_manipulation.rs"]
mod query_manipulation;

/// Insert an autocomplete suggestion from App context
///
/// This function delegates to the existing `insert_suggestion()` function and
/// updates related app state (hide autocomplete, reset scroll, clear error overlay).
/// This pattern allows the App to delegate feature-specific logic to the autocomplete module.
///
/// # Arguments
/// * `app` - Mutable reference to the App struct
/// * `suggestion` - The suggestion to insert
pub fn insert_suggestion_from_app(app: &mut App, suggestion: &Suggestion) {
    // Delegate to existing insert_suggestion function
    insert_suggestion(&mut app.input.textarea, &mut app.query, suggestion);

    // Hide autocomplete and reset scroll/error state
    app.autocomplete.hide();
    app.results_scroll.reset();
    app.error_overlay_visible = false; // Auto-hide error overlay on query change

    // Handle AI state based on query result
    let cursor_pos = app.input.textarea.cursor().1;
    let query = app.input.textarea.lines()[0].as_ref();
    crate::ai::ai_events::handle_query_result(
        &mut app.ai,
        &app.query.result,
        query,
        cursor_pos,
        app.query.executor.json_input(),
    );
}

/// Insert an autocomplete suggestion at the current cursor position
/// Uses explicit state-based formulas for each context type
///
/// Returns the new query string after insertion
pub fn insert_suggestion(
    textarea: &mut TextArea<'_>,
    query_state: &mut QueryState,
    suggestion: &Suggestion,
) {
    let suggestion_text = &suggestion.text;
    // Get base query that produced these suggestions
    let base_query = match &query_state.base_query_for_suggestions {
        Some(b) => b.clone(),
        None => {
            // Fallback to current query if no base (shouldn't happen)
            textarea.lines()[0].clone()
        }
    };

    let query = textarea.lines()[0].clone();
    let cursor_pos = textarea.cursor().1;
    let before_cursor = &query[..cursor_pos.min(query.len())];

    #[cfg(debug_assertions)]
    debug!(
        "insert_suggestion: current_query='{}' base_query='{}' suggestion='{}' cursor_pos={}",
        query, base_query, suggestion_text, cursor_pos
    );

    // Determine the trigger context
    // Note: For insertion, we create a temporary BraceTracker since we only need
    // to distinguish FunctionContext from FieldContext here. ObjectKeyContext
    // handling will be added in task 9.
    let mut temp_tracker = crate::autocomplete::BraceTracker::new();
    temp_tracker.rebuild(before_cursor);
    let (context, partial) = analyze_context(before_cursor, &temp_tracker);

    #[cfg(debug_assertions)]
    debug!(
        "context_analysis: context={:?} partial='{}'",
        context, partial
    );

    // For function/operator context (jq keywords like then, else, end, etc.),
    // we should do simple replacement without adding dots or complex formulas
    if context == SuggestionContext::FunctionContext {
        // Simple replacement: remove the partial and insert the suggestion
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        // Append opening parenthesis if the function requires arguments
        let insert_text = if suggestion.needs_parens {
            format!("{}(", suggestion_text)
        } else {
            suggestion_text.to_string()
        };

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            insert_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "function_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        // Replace the entire line with new query
        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion (including parenthesis if added)
        let target_pos = replacement_start + insert_text.len();
        move_cursor_to_column(textarea, target_pos);

        // Execute query
        execute_query_and_update(textarea, query_state);
        return;
    }

    // For ObjectKeyContext (object key names inside `{}`),
    // we do simple replacement similar to FunctionContext
    // This handles cases like `{na` -> `{name` or `{name: .name, ag` -> `{name: .name, age`
    if context == SuggestionContext::ObjectKeyContext {
        // Simple replacement: remove the partial and insert the suggestion
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            suggestion_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "object_key_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        // Replace the entire line with new query
        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion
        let target_pos = replacement_start + suggestion_text.len();
        move_cursor_to_column(textarea, target_pos);

        // Execute query
        execute_query_and_update(textarea, query_state);
        return;
    }

    // For field context, continue with the existing complex logic
    let char_before = find_char_before_field_access(before_cursor, &partial);
    let trigger_type = QueryState::classify_char(char_before);

    // Extract middle_query: everything between base and current field being typed
    // This preserves complex expressions like if/then/else, functions, etc.
    let mut middle_query = extract_middle_query(&query, &base_query, before_cursor, &partial);
    let mut adjusted_base = base_query.clone();
    let mut adjusted_suggestion = suggestion_text.to_string();

    #[cfg(debug_assertions)]
    debug!(
        "field_context: partial='{}' char_before={:?} trigger_type={:?} middle_query='{}'",
        partial, char_before, trigger_type, middle_query
    );

    // Special handling for CloseBracket trigger with [] in middle_query
    // This handles nested arrays like: .services[].capacityProviderStrategy[].field
    // When user types [], it becomes part of middle_query, but should be part of base
    if trigger_type == CharType::CloseBracket && middle_query == "[]" {
        #[cfg(debug_assertions)]
        debug!("nested_array_adjustment: detected [] in middle_query, moving to base");

        // Move [] from middle to base
        adjusted_base = format!("{}{}", base_query, middle_query);
        middle_query = String::new();

        // Strip [] prefix from suggestion if present (it's already in the query)
        // Also strip the leading dot since CloseBracket formula will add it
        if let Some(stripped) = adjusted_suggestion.strip_prefix("[]") {
            // Strip leading dot if present (e.g., "[].base" -> "base")
            adjusted_suggestion = stripped.strip_prefix('.').unwrap_or(stripped).to_string();

            #[cfg(debug_assertions)]
            debug!("nested_array_adjustment: stripped [] and leading dot from suggestion");
        }

        #[cfg(debug_assertions)]
        debug!(
            "nested_array_adjustment: adjusted_base='{}' adjusted_suggestion='{}' middle_query='{}'",
            adjusted_base, adjusted_suggestion, middle_query
        );
    }

    // Special case: if base is root "." and suggestion starts with ".",
    // replace the base entirely instead of appending
    // This prevents: "." + ".services" = "..services"
    let new_query = if adjusted_base == "."
        && adjusted_suggestion.starts_with('.')
        && middle_query.is_empty()
    {
        #[cfg(debug_assertions)]
        debug!("formula: root_replacement (special case for root '.')");

        adjusted_suggestion.to_string()
    } else {
        // Apply insertion formula: base + middle + (operator) + suggestion
        // The middle preserves complex expressions between base and current field
        let formula_result = match trigger_type {
            CharType::NoOp => {
                // NoOp means continuing a path, but we need to check if suggestion needs a dot
                // - If suggestion starts with special char like [, {, etc., don't add dot
                // - If base is root ".", don't add another dot
                // - Otherwise, add dot for path continuation (like .user.name)
                let needs_dot = !adjusted_suggestion.starts_with('[')
                    && !adjusted_suggestion.starts_with('{')
                    && !adjusted_suggestion.starts_with('.')
                    && adjusted_base != ".";

                if needs_dot {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + '.' + suggestion");

                    format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
                } else {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + suggestion (no dot added)");

                    format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
                }
            }
            CharType::CloseBracket => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseBracket -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::PipeOperator | CharType::Semicolon | CharType::Comma | CharType::Colon => {
                #[cfg(debug_assertions)]
                debug!("formula: Separator -> base + middle + ' ' + suggestion");

                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!(
                    "{}{} {}",
                    adjusted_base, trimmed_middle, adjusted_suggestion
                )
            }
            CharType::OpenParen => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenParen -> base + middle + suggestion (paren already in middle)"
                );

                // Formula: base + middle + suggestion
                // The ( is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBracket => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBracket -> base + middle + suggestion (bracket already in middle)"
                );

                // Formula: base + middle + suggestion
                // The [ is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBrace => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBrace -> base + middle + suggestion (brace already in middle)"
                );

                // Formula: base + middle + suggestion
                // The { is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::QuestionMark => {
                #[cfg(debug_assertions)]
                debug!("formula: QuestionMark -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::Dot => {
                #[cfg(debug_assertions)]
                debug!("formula: Dot -> base + middle + suggestion");

                // Formula: base + middle + suggestion
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::CloseParen | CharType::CloseBrace => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseParen/CloseBrace -> base + middle + '.' + suggestion");

                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
        };

        #[cfg(debug_assertions)]
        debug!(
            "formula_components: base='{}' middle='{}' suggestion='{}'",
            adjusted_base, middle_query, adjusted_suggestion
        );

        formula_result
    };

    #[cfg(debug_assertions)]
    debug!("new_query_constructed: '{}'", new_query);

    // Replace the entire line with new query
    textarea.delete_line_by_head();
    textarea.insert_str(&new_query);

    #[cfg(debug_assertions)]
    debug!("query_after_insertion: '{}'", textarea.lines()[0]);

    // Move cursor to end of query
    let target_pos = new_query.len();
    move_cursor_to_column(textarea, target_pos);

    // Execute query
    execute_query_and_update(textarea, query_state);
}
