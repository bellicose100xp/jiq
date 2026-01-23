//! Tests for region hit testing

use ratatui::layout::Rect;

use super::layout_hit_test::region_at;
use super::layout_regions::{LayoutRegions, Region};

fn create_test_regions() -> LayoutRegions {
    let mut regions = LayoutRegions::new();
    regions.results_pane = Some(Rect::new(0, 0, 100, 40));
    regions.input_field = Some(Rect::new(0, 40, 100, 3));
    regions
}

#[test]
fn test_hit_results_pane() {
    let regions = create_test_regions();

    assert_eq!(region_at(&regions, 50, 20), Some(Region::ResultsPane));
    assert_eq!(region_at(&regions, 0, 0), Some(Region::ResultsPane));
    assert_eq!(region_at(&regions, 99, 39), Some(Region::ResultsPane));
}

#[test]
fn test_hit_input_field() {
    let regions = create_test_regions();

    assert_eq!(region_at(&regions, 50, 40), Some(Region::InputField));
    assert_eq!(region_at(&regions, 0, 42), Some(Region::InputField));
}

#[test]
fn test_hit_outside_all_regions() {
    let regions = create_test_regions();

    assert_eq!(region_at(&regions, 100, 50), None);
    assert_eq!(region_at(&regions, 0, 50), None);
}

#[test]
fn test_overlay_priority_help_popup() {
    let mut regions = create_test_regions();
    regions.help_popup = Some(Rect::new(10, 10, 80, 25));

    assert_eq!(region_at(&regions, 50, 20), Some(Region::HelpPopup));
    assert_eq!(region_at(&regions, 5, 5), Some(Region::ResultsPane));
}

#[test]
fn test_overlay_priority_error_overlay() {
    let mut regions = create_test_regions();
    regions.error_overlay = Some(Rect::new(10, 30, 80, 5));

    assert_eq!(region_at(&regions, 50, 32), Some(Region::ErrorOverlay));
    assert_eq!(region_at(&regions, 50, 20), Some(Region::ResultsPane));
}

#[test]
fn test_overlay_priority_ai_window() {
    let mut regions = create_test_regions();
    regions.ai_window = Some(Rect::new(50, 10, 40, 25));

    assert_eq!(region_at(&regions, 70, 20), Some(Region::AiWindow));
    assert_eq!(region_at(&regions, 20, 20), Some(Region::ResultsPane));
}

#[test]
fn test_overlay_priority_autocomplete() {
    let mut regions = create_test_regions();
    regions.autocomplete = Some(Rect::new(5, 30, 30, 10));

    assert_eq!(region_at(&regions, 15, 35), Some(Region::Autocomplete));
}

#[test]
fn test_overlay_priority_history_popup() {
    let mut regions = create_test_regions();
    regions.history_popup = Some(Rect::new(0, 20, 100, 20));

    assert_eq!(region_at(&regions, 50, 30), Some(Region::HistoryPopup));
}

#[test]
fn test_overlay_priority_tooltip() {
    let mut regions = create_test_regions();
    regions.tooltip = Some(Rect::new(60, 20, 30, 10));

    assert_eq!(region_at(&regions, 75, 25), Some(Region::Tooltip));
}

#[test]
fn test_search_bar_region() {
    let mut regions = create_test_regions();
    regions.results_pane = Some(Rect::new(0, 0, 100, 37));
    regions.search_bar = Some(Rect::new(0, 37, 100, 3));

    assert_eq!(region_at(&regions, 50, 38), Some(Region::SearchBar));
    assert_eq!(region_at(&regions, 50, 20), Some(Region::ResultsPane));
}

#[test]
fn test_snippet_regions() {
    let mut regions = LayoutRegions::new();
    regions.snippet_list = Some(Rect::new(0, 5, 100, 20));
    regions.snippet_preview = Some(Rect::new(0, 25, 100, 15));

    assert_eq!(region_at(&regions, 50, 10), Some(Region::SnippetList));
    assert_eq!(region_at(&regions, 50, 30), Some(Region::SnippetPreview));
}

#[test]
fn test_help_popup_takes_priority_over_all() {
    let mut regions = LayoutRegions::new();
    regions.results_pane = Some(Rect::new(0, 0, 100, 50));
    regions.ai_window = Some(Rect::new(50, 10, 40, 30));
    regions.error_overlay = Some(Rect::new(10, 35, 80, 5));
    regions.snippet_list = Some(Rect::new(0, 10, 45, 25));
    regions.help_popup = Some(Rect::new(5, 5, 90, 40));

    // Help popup should be returned for any point inside it
    assert_eq!(region_at(&regions, 50, 20), Some(Region::HelpPopup));
    assert_eq!(region_at(&regions, 80, 35), Some(Region::HelpPopup));
}

#[test]
fn test_empty_regions_returns_none() {
    let regions = LayoutRegions::new();

    assert_eq!(region_at(&regions, 0, 0), None);
    assert_eq!(region_at(&regions, 50, 25), None);
    assert_eq!(region_at(&regions, 100, 100), None);
}

#[test]
fn test_boundary_conditions() {
    let mut regions = LayoutRegions::new();
    regions.results_pane = Some(Rect::new(10, 10, 50, 30));

    // Inside boundaries (inclusive start)
    assert_eq!(region_at(&regions, 10, 10), Some(Region::ResultsPane));
    assert_eq!(region_at(&regions, 59, 39), Some(Region::ResultsPane));

    // Outside boundaries (exclusive end)
    assert_eq!(region_at(&regions, 60, 39), None);
    assert_eq!(region_at(&regions, 59, 40), None);
    assert_eq!(region_at(&regions, 9, 10), None);
    assert_eq!(region_at(&regions, 10, 9), None);
}
