//! Tests for help content definitions

use super::*;

#[test]
fn test_help_entries_count() {
    // Verify we have a reasonable number of help entries
    assert!(HELP_ENTRIES.len() >= 50);
}

#[test]
fn test_help_entries_have_valid_structure() {
    for (key, value) in HELP_ENTRIES.iter() {
        // Both key and value should be valid UTF-8 strings (guaranteed by type)
        assert!(key.len() < 100, "Key too long: {}", key);
        assert!(value.len() < 200, "Value too long: {}", value);
    }
}

#[test]
fn test_help_entries_contain_essential_keys() {
    let keys: Vec<&str> = HELP_ENTRIES.iter().map(|(k, _)| *k).collect();

    assert!(keys.contains(&"F1 or ?"), "Missing help toggle key");
    assert!(keys.contains(&"Ctrl+C"), "Missing quit key");
    assert!(keys.contains(&"Enter"), "Missing enter key");
    assert!(keys.contains(&"Esc"), "Missing escape key");
}

#[test]
fn test_help_entries_section_headers() {
    let values: Vec<&str> = HELP_ENTRIES.iter().map(|(_, v)| *v).collect();

    assert!(
        values.iter().any(|v| v.contains("GLOBAL")),
        "Missing GLOBAL section"
    );
    assert!(
        values.iter().any(|v| v.contains("INSERT MODE")),
        "Missing INSERT MODE section"
    );
    assert!(
        values.iter().any(|v| v.contains("NORMAL MODE")),
        "Missing NORMAL MODE section"
    );
    assert!(
        values.iter().any(|v| v.contains("RESULTS")),
        "Missing RESULTS section"
    );
}

#[test]
fn test_help_footer_has_content() {
    // Verify footer has reasonable content
    assert!(HELP_FOOTER.len() >= 10);
}

#[test]
fn test_help_footer_contains_navigation_hints() {
    assert!(
        HELP_FOOTER.contains("scroll"),
        "Footer should mention scrolling"
    );
    assert!(
        HELP_FOOTER.contains("close"),
        "Footer should mention closing"
    );
}

#[test]
fn test_help_entries_have_separator_lines() {
    let empty_key_count = HELP_ENTRIES.iter().filter(|(k, _)| k.is_empty()).count();

    // Should have multiple separator lines (empty keys are used for spacing)
    assert!(
        empty_key_count >= 5,
        "Should have separator lines for sections"
    );
}

#[test]
fn test_help_entries_ai_section() {
    let values: Vec<&str> = HELP_ENTRIES.iter().map(|(_, v)| *v).collect();

    assert!(
        values.iter().any(|v| v.contains("AI")),
        "Missing AI ASSISTANT section"
    );
}

#[test]
fn test_help_entries_search_section() {
    let values: Vec<&str> = HELP_ENTRIES.iter().map(|(_, v)| *v).collect();

    assert!(
        values.iter().any(|v| v.contains("SEARCH")),
        "Missing SEARCH section"
    );
}

#[test]
fn test_help_entries_autocomplete_section() {
    let values: Vec<&str> = HELP_ENTRIES.iter().map(|(_, v)| *v).collect();

    assert!(
        values.iter().any(|v| v.contains("AUTOCOMPLETE")),
        "Missing AUTOCOMPLETE section"
    );
}
