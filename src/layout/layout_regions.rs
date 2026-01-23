//! Layout regions tracking for UI components
//!
//! Tracks where UI components are rendered for position-aware mouse interactions.

use ratatui::layout::Rect;

/// Identifies a UI component region
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    // Base layout
    ResultsPane,
    InputField,
    SearchBar,

    // Popups
    AiWindow,
    Autocomplete,
    HistoryPopup,
    Tooltip,
    ErrorOverlay,
    HelpPopup,

    // Snippet manager sub-regions
    SnippetList,
    SnippetPreview,
}

/// Tracks rendered areas of UI components
///
/// Updated during each render pass. Regions are `None` when the component is not visible.
/// Used by mouse event handlers to determine which component is under the cursor.
#[derive(Default, Clone, Debug)]
pub struct LayoutRegions {
    // Base layout
    pub results_pane: Option<Rect>,
    pub input_field: Option<Rect>,
    pub search_bar: Option<Rect>,

    // Popups (only populated when visible)
    pub ai_window: Option<Rect>,
    pub autocomplete: Option<Rect>,
    pub history_popup: Option<Rect>,
    pub tooltip: Option<Rect>,
    pub error_overlay: Option<Rect>,
    pub help_popup: Option<Rect>,

    // Snippet manager sub-regions
    pub snippet_list: Option<Rect>,
    pub snippet_preview: Option<Rect>,
}

impl LayoutRegions {
    /// Create a new empty LayoutRegions
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all regions before a new render pass
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
