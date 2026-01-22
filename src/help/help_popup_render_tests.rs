//! Tests for help_popup_render

use super::*;
use crate::help::HelpTab;

#[test]
fn test_render_help_sections_global() {
    let content = get_tab_content(HelpTab::Global);
    let lines = render_help_sections(content.sections);

    assert!(!lines.is_empty(), "Should render some lines");

    // Check that entries are rendered
    let line_strings: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    assert!(
        line_strings.iter().any(|s| s.contains("F1")),
        "Should contain F1 key"
    );
}

#[test]
fn test_render_help_sections_with_subsections() {
    let content = get_tab_content(HelpTab::Input);
    let lines = render_help_sections(content.sections);

    let line_strings: Vec<String> = lines.iter().map(|l| l.to_string()).collect();

    // Should have section headers
    assert!(
        line_strings.iter().any(|s| s.contains("INSERT MODE")),
        "Should contain INSERT MODE header"
    );
    assert!(
        line_strings.iter().any(|s| s.contains("NORMAL MODE")),
        "Should contain NORMAL MODE header"
    );
}

#[test]
fn test_render_tab_bar_highlights_active() {
    // Just verify it doesn't panic - visual inspection needed for actual styling
    let _tabs = render_tab_bar(HelpTab::Global);
}

#[test]
fn test_render_tab_bar_all_tabs() {
    for tab in HelpTab::all() {
        // Just verify it doesn't panic for any tab
        let _tabs = render_tab_bar(*tab);
    }
}
