//! Tests for help_content

use super::*;

#[test]
fn test_help_entries_not_empty() {
    assert!(!HELP_ENTRIES.is_empty());
}

#[test]
fn test_help_entries_contains_global_section() {
    let global_section = HELP_ENTRIES
        .iter()
        .find(|(_, desc)| desc.contains("GLOBAL"));
    assert!(global_section.is_some());
}

#[test]
fn test_help_entries_contains_snippets_shortcut() {
    let snippets_entry = HELP_ENTRIES
        .iter()
        .find(|(key, desc)| key.contains("Ctrl+S") && desc.contains("snippets"));
    assert!(
        snippets_entry.is_some(),
        "Help entries should contain Ctrl+S for snippets manager"
    );
}

#[test]
fn test_help_entries_snippets_in_global_section() {
    let mut found_global = false;
    let mut found_snippets_after_global = false;
    let mut found_next_section = false;

    for (key, desc) in HELP_ENTRIES.iter() {
        if desc.contains("GLOBAL") {
            found_global = true;
        } else if found_global && !found_next_section {
            if desc.starts_with("──") && !desc.contains("GLOBAL") {
                found_next_section = true;
            } else if key.contains("Ctrl+S") && desc.contains("snippets") {
                found_snippets_after_global = true;
            }
        }
    }

    assert!(found_global, "Should have GLOBAL section");
    assert!(
        found_snippets_after_global,
        "Ctrl+S snippets entry should be in GLOBAL section"
    );
}

#[test]
fn test_help_entries_all_valid_format() {
    for (key, desc) in HELP_ENTRIES.iter() {
        if !key.is_empty() {
            assert!(
                !desc.is_empty(),
                "If key is provided, description should not be empty"
            );
        }
    }
}

#[test]
fn test_help_entries_contains_expected_sections() {
    let sections = [
        "GLOBAL",
        "INPUT: INSERT MODE",
        "INPUT: NORMAL MODE",
        "AUTOCOMPLETE",
        "RESULTS PANE",
        "SEARCH IN RESULTS",
        "HISTORY POPUP",
        "ERROR OVERLAY",
        "AI ASSISTANT",
    ];

    for section in sections {
        let found = HELP_ENTRIES.iter().any(|(_, desc)| desc.contains(section));
        assert!(found, "Help should contain section: {}", section);
    }
}

#[test]
fn test_help_footer_not_empty() {
    assert!(!HELP_FOOTER.is_empty());
}

#[test]
fn test_help_footer_contains_navigation_hints() {
    assert!(HELP_FOOTER.contains("scroll"));
    assert!(HELP_FOOTER.contains("close"));
}

#[test]
fn test_snippets_entry_location() {
    let mut global_index = None;
    let mut snippets_index = None;
    let mut next_section_index = None;

    for (i, (key, desc)) in HELP_ENTRIES.iter().enumerate() {
        if desc.contains("── GLOBAL ──") {
            global_index = Some(i);
        } else if key.contains("Ctrl+S") && desc.contains("snippets") {
            snippets_index = Some(i);
        } else if global_index.is_some()
            && snippets_index.is_none()
            && desc.starts_with("──")
            && !desc.contains("GLOBAL")
        {
            next_section_index = Some(i);
        }
    }

    assert!(global_index.is_some(), "Should have GLOBAL section");
    assert!(snippets_index.is_some(), "Should have snippets entry");

    if let (Some(global), Some(snippets), Some(next)) =
        (global_index, snippets_index, next_section_index)
    {
        assert!(
            snippets > global && snippets < next,
            "Snippets entry should be between GLOBAL section header and next section"
        );
    }
}
