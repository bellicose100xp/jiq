//! Hit testing for layout regions
//!
//! Determines which UI component is at a given screen position.

use ratatui::layout::Rect;

use super::layout_regions::{LayoutRegions, Region};

/// Check if a point is within a rectangle
#[allow(dead_code)]
fn contains(rect: &Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Returns the topmost region containing the given point
///
/// Checks overlays first (in render order, topmost last), then base regions.
/// Returns `None` if the point is outside all tracked regions.
#[allow(dead_code)]
pub fn region_at(regions: &LayoutRegions, x: u16, y: u16) -> Option<Region> {
    // Check overlays first (in reverse render order - topmost first)
    // Help popup is rendered last, so it's topmost
    if let Some(rect) = &regions.help_popup
        && contains(rect, x, y)
    {
        return Some(Region::HelpPopup);
    }

    // Error overlay
    if let Some(rect) = &regions.error_overlay
        && contains(rect, x, y)
    {
        return Some(Region::ErrorOverlay);
    }

    // Snippet manager regions (rendered over results area)
    if let Some(rect) = &regions.snippet_preview
        && contains(rect, x, y)
    {
        return Some(Region::SnippetPreview);
    }
    if let Some(rect) = &regions.snippet_list
        && contains(rect, x, y)
    {
        return Some(Region::SnippetList);
    }

    // History popup (above input)
    if let Some(rect) = &regions.history_popup
        && contains(rect, x, y)
    {
        return Some(Region::HistoryPopup);
    }

    // AI window (right side above input)
    if let Some(rect) = &regions.ai_window
        && contains(rect, x, y)
    {
        return Some(Region::AiWindow);
    }

    // Tooltip (right side above input)
    if let Some(rect) = &regions.tooltip
        && contains(rect, x, y)
    {
        return Some(Region::Tooltip);
    }

    // Autocomplete (left side above input)
    if let Some(rect) = &regions.autocomplete
        && contains(rect, x, y)
    {
        return Some(Region::Autocomplete);
    }

    // Base layout regions
    // Search bar (at bottom of results when visible)
    if let Some(rect) = &regions.search_bar
        && contains(rect, x, y)
    {
        return Some(Region::SearchBar);
    }

    // Input field
    if let Some(rect) = &regions.input_field
        && contains(rect, x, y)
    {
        return Some(Region::InputField);
    }

    // Results pane (checked last as it's the largest base area)
    if let Some(rect) = &regions.results_pane
        && contains(rect, x, y)
    {
        return Some(Region::ResultsPane);
    }

    None
}
