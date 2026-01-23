//! Tests for LayoutRegions struct

use ratatui::layout::Rect;

use super::layout_regions::{LayoutRegions, Region};

#[test]
fn test_new_creates_empty_regions() {
    let regions = LayoutRegions::new();

    assert!(regions.results_pane.is_none());
    assert!(regions.input_field.is_none());
    assert!(regions.search_bar.is_none());
    assert!(regions.ai_window.is_none());
    assert!(regions.autocomplete.is_none());
    assert!(regions.history_popup.is_none());
    assert!(regions.tooltip.is_none());
    assert!(regions.error_overlay.is_none());
    assert!(regions.help_popup.is_none());
    assert!(regions.snippet_list.is_none());
    assert!(regions.snippet_preview.is_none());
}

#[test]
fn test_clear_resets_all_regions() {
    let mut regions = LayoutRegions::new();

    regions.results_pane = Some(Rect::new(0, 0, 100, 50));
    regions.input_field = Some(Rect::new(0, 50, 100, 3));
    regions.ai_window = Some(Rect::new(50, 10, 40, 30));

    regions.clear();

    assert!(regions.results_pane.is_none());
    assert!(regions.input_field.is_none());
    assert!(regions.ai_window.is_none());
}

#[test]
fn test_region_enum_variants() {
    let regions = vec![
        Region::ResultsPane,
        Region::InputField,
        Region::SearchBar,
        Region::AiWindow,
        Region::Autocomplete,
        Region::HistoryPopup,
        Region::Tooltip,
        Region::ErrorOverlay,
        Region::HelpPopup,
        Region::SnippetList,
        Region::SnippetPreview,
    ];

    assert_eq!(regions.len(), 11);
}

#[test]
fn test_region_derives_eq() {
    assert_eq!(Region::ResultsPane, Region::ResultsPane);
    assert_ne!(Region::ResultsPane, Region::InputField);
}

#[test]
fn test_region_derives_clone() {
    let region = Region::AiWindow;
    let cloned = region;
    assert_eq!(region, cloned);
}

#[test]
fn test_region_derives_copy() {
    let region = Region::SearchBar;
    let copied = region;
    assert_eq!(region, copied);
}
