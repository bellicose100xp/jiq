//! Tests for help_content

use super::*;

#[test]
#[allow(clippy::const_is_empty)]
fn test_help_categories_not_empty() {
    assert!(!HELP_CATEGORIES.is_empty());
    assert_eq!(HELP_CATEGORIES.len(), 7);
}

#[test]
fn test_all_tabs_have_categories() {
    for tab in HelpTab::all() {
        let category = get_tab_content(*tab);
        assert_eq!(category.tab, *tab);
        assert!(!category.sections.is_empty());
    }
}

#[test]
fn test_global_tab_contains_essential_shortcuts() {
    let global = get_tab_content(HelpTab::Global);

    let entries: Vec<_> = global
        .sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .collect();

    assert!(
        entries.iter().any(|(k, _)| k.contains("F1")),
        "Global should have F1 for help"
    );
    assert!(
        entries.iter().any(|(k, _)| k.contains("Ctrl+S")),
        "Global should have Ctrl+S for snippets"
    );
    assert!(
        entries.iter().any(|(k, _)| k.contains("Enter")),
        "Global should have Enter for output"
    );
}

#[test]
fn test_input_tab_has_insert_and_normal_sections() {
    let input = get_tab_content(HelpTab::Input);

    let section_titles: Vec<_> = input.sections.iter().filter_map(|s| s.title).collect();

    assert!(
        section_titles.iter().any(|t| t.contains("INSERT")),
        "Input tab should have INSERT MODE section"
    );
    assert!(
        section_titles.iter().any(|t| t.contains("NORMAL")),
        "Input tab should have NORMAL MODE section"
    );
    assert!(
        section_titles.iter().any(|t| t.contains("AUTOCOMPLETE")),
        "Input tab should have AUTOCOMPLETE section"
    );
}

#[test]
fn test_result_tab_contains_navigation() {
    let results = get_tab_content(HelpTab::Result);

    let entries: Vec<_> = results
        .sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .collect();

    assert!(
        entries
            .iter()
            .any(|(k, _)| k.contains("j") || k.contains("k")),
        "Result should have j/k for scrolling"
    );
    assert!(
        entries
            .iter()
            .any(|(k, _)| k.contains("g") || k.contains("G")),
        "Result should have g/G for jump to top/bottom"
    );
}

#[test]
fn test_history_tab_contains_history_shortcuts() {
    let history = get_tab_content(HelpTab::History);

    let entries: Vec<_> = history
        .sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .collect();

    assert!(
        entries.iter().any(|(k, _)| k.contains("Ctrl+R")),
        "History should have Ctrl+R to open"
    );
    assert!(
        entries
            .iter()
            .any(|(_, d)| d.contains("Navigate") || d.contains("entries")),
        "History should have navigation"
    );
}

#[test]
fn test_search_tab_contains_search_shortcuts() {
    let search = get_tab_content(HelpTab::Search);

    let entries: Vec<_> = search
        .sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .collect();

    assert!(
        entries.iter().any(|(k, _)| k.contains("Ctrl+F")),
        "Search should have Ctrl+F"
    );
    assert!(
        entries
            .iter()
            .any(|(k, d)| k.contains("n") && d.contains("match")),
        "Search should have n for next match"
    );
}

#[test]
fn test_snippet_tab_contains_snippet_shortcuts() {
    let snippet = get_tab_content(HelpTab::Snippet);

    let entries: Vec<_> = snippet
        .sections
        .iter()
        .flat_map(|s| s.entries.iter())
        .collect();

    assert!(
        entries.iter().any(|(k, _)| k.contains("Ctrl+S")),
        "Snippet should have Ctrl+S to open"
    );
    assert!(
        entries.iter().any(|(_, d)| d.contains("Create")),
        "Snippet should have create functionality"
    );
    assert!(
        entries.iter().any(|(_, d)| d.contains("Delete")),
        "Snippet should have delete functionality"
    );
}

#[test]
fn test_ai_tab_contains_ai_shortcuts() {
    let ai = get_tab_content(HelpTab::AI);

    let entries: Vec<_> = ai.sections.iter().flat_map(|s| s.entries.iter()).collect();

    assert!(
        entries.iter().any(|(k, _)| k.contains("Ctrl+A")),
        "AI should have Ctrl+A to toggle"
    );
    assert!(
        entries.iter().any(|(k, _)| k.contains("Alt+")),
        "AI should have Alt+number shortcuts"
    );
}

#[test]
fn test_all_entries_have_descriptions() {
    for category in HELP_CATEGORIES {
        for section in category.sections {
            for (key, desc) in section.entries {
                assert!(!key.is_empty(), "Key should not be empty");
                assert!(
                    !desc.is_empty(),
                    "Description should not be empty for key: {}",
                    key
                );
            }
        }
    }
}

#[test]
#[allow(clippy::const_is_empty)]
fn test_help_footer_not_empty() {
    assert!(!HELP_FOOTER.is_empty());
}

#[test]
fn test_help_footer_contains_navigation_hints() {
    assert!(
        HELP_FOOTER.contains("Tab") || HELP_FOOTER.contains("tab"),
        "Footer should mention tab navigation"
    );
    assert!(
        HELP_FOOTER.contains("scroll"),
        "Footer should mention scrolling"
    );
    assert!(
        HELP_FOOTER.contains("close"),
        "Footer should mention how to close"
    );
}

#[test]
fn test_get_tab_content_returns_correct_tab() {
    for tab in HelpTab::all() {
        let content = get_tab_content(*tab);
        assert_eq!(content.tab, *tab);
    }
}
