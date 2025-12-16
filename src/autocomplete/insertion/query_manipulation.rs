//! Query string manipulation utilities for autocomplete insertion

#[cfg(debug_assertions)]
use log::debug;

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

    #[cfg(debug_assertions)]
    debug!(
        "extract_middle_query: current_query='{}' before_cursor='{}' partial='{}' trigger_pos={} base_len={}",
        current_query,
        before_cursor,
        partial,
        trigger_pos_in_before_cursor,
        base_query.len()
    );

    // Middle is everything from end of base to (but not including) trigger
    let base_len = base_query.len();
    if trigger_pos_in_before_cursor <= base_len {
        // Trigger at or before base ends - no middle
        return String::new();
    }

    // Extract middle - preserve all whitespace as it may be significant
    // (e.g., "then " needs the space before the field access)
    let middle = current_query[base_len..trigger_pos_in_before_cursor].to_string();

    #[cfg(debug_assertions)]
    debug!("extract_middle_query: extracted_middle='{}'", middle);

    middle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_middle_query_simple_path() {
        // Simple path: no middle
        let result = extract_middle_query(".services.ca", ".services", ".services.ca", "ca");
        assert_eq!(result, "", "Simple path should have empty middle");
    }

    #[test]
    fn test_extract_middle_query_after_pipe() {
        // After pipe with identity - preserves trailing space
        let result = extract_middle_query(".services | .ca", ".services", ".services | .ca", "ca");
        assert_eq!(
            result, " | ",
            "Middle: pipe with trailing space (before dot)"
        );
    }

    #[test]
    fn test_extract_middle_query_with_if_then() {
        // Complex: if/then between base and current field - preserves trailing space
        let query = ".services | if has(\"x\") then .ca";
        let before_cursor = query;
        let result = extract_middle_query(query, ".services", before_cursor, "ca");
        assert_eq!(
            result, " | if has(\"x\") then ",
            "Middle with trailing space (important for 'then ')"
        );
    }

    #[test]
    fn test_extract_middle_query_with_select() {
        // With select function - preserves trailing space
        let query = ".items | select(.active) | .na";
        let result = extract_middle_query(query, ".items", query, "na");
        assert_eq!(
            result, " | select(.active) | ",
            "Middle: includes pipe with trailing space"
        );
    }

    #[test]
    fn test_extract_middle_query_no_partial() {
        // Just typed dot, no partial yet - preserves trailing space
        let result = extract_middle_query(".services | .", ".services", ".services | .", "");
        assert_eq!(
            result, " | ",
            "Middle with trailing space before trigger dot"
        );
    }

    #[test]
    fn test_extract_middle_query_base_not_prefix() {
        // Edge case: base is not prefix of current query (shouldn't happen)
        let result = extract_middle_query(".items.ca", ".services", ".items.ca", "ca");
        assert_eq!(result, "", "Should return empty if base not a prefix");
    }

    #[test]
    fn test_extract_middle_query_nested_pipes() {
        // Multiple pipes and functions - preserves trailing space
        let query = ".a | .b | map(.c) | .d";
        let result = extract_middle_query(query, ".a", query, "d");
        assert_eq!(result, " | .b | map(.c) | ", "Middle with trailing space");
    }
}
