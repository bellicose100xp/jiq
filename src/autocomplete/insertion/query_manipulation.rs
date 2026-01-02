//! Query string manipulation utilities for autocomplete insertion

/// Extract middle query: everything between base and current field being typed
///
/// Examples:
/// - Query: ".services | if has(...) then .ca", base: ".services"
///   → middle: " | if has(...) then "
/// - Query: ".services | .ca", base: ".services"
///   → middle: " | "
/// - Query: ".services.ca", base: ".services"
///   → middle: ""
pub fn extract_middle_query(
    current_query: &str,
    base_query: &str,
    before_cursor: &str,
    partial: &str,
) -> String {
    // Find where base ends in current query
    if !current_query.starts_with(base_query) {
        // Base is not a prefix of current query (shouldn't happen, but handle gracefully)
        return String::new();
    }

    // Find where the trigger char is in before_cursor
    // Middle should be: everything after base, up to but not including trigger char
    // Examples:
    //   Query: ".services | .ca", partial: "ca", base: ".services"
    //   → trigger is the dot at position 11
    //   → middle = query[9..11] = " | " (with trailing space, no dot)
    let trigger_pos_in_before_cursor = if partial.is_empty() {
        // Just typed trigger char - it's the last char
        before_cursor.len().saturating_sub(1)
    } else {
        // Partial being typed - trigger is one char before partial
        before_cursor.len().saturating_sub(partial.len() + 1)
    };

    // Middle is everything from end of base to (but not including) trigger
    let base_len = base_query.len();
    if trigger_pos_in_before_cursor <= base_len {
        // Trigger at or before base ends - no middle
        return String::new();
    }

    // Extract middle - preserve all whitespace as it may be significant
    // (e.g., "then " needs the space before the field access)
    let middle = current_query[base_len..trigger_pos_in_before_cursor].to_string();

    middle
}

#[cfg(test)]
#[path = "query_manipulation_tests.rs"]
mod query_manipulation_tests;
