//! Mouse scroll handling
//!
//! Routes scroll events to the appropriate component based on cursor position.

use super::app_state::App;
use crate::layout::Region;
use crate::scroll::Scrollable;

/// Scroll direction for mouse wheel events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

/// Handle scroll event for the given region
///
/// Routes scroll to the component under the cursor.
/// Falls back to results pane when cursor is outside all regions.
pub fn handle_scroll(app: &mut App, region: Option<Region>, direction: ScrollDirection) {
    match region {
        Some(Region::ResultsPane) | None => scroll_results(app, direction),
        Some(Region::HelpPopup) => scroll_help(app, direction),
        Some(Region::AiWindow) => scroll_ai(app, direction),
        Some(Region::SnippetList) => scroll_snippets(app, direction),
        Some(Region::HistoryPopup) => scroll_history(app, direction),
        Some(Region::Autocomplete) => scroll_autocomplete(app, direction),
        // Non-scrollable regions: do nothing
        Some(Region::InputField)
        | Some(Region::SearchBar)
        | Some(Region::Tooltip)
        | Some(Region::ErrorOverlay)
        | Some(Region::SnippetPreview) => {}
    }
}

const RESULTS_SCROLL_LINES: u16 = 3;
const HELP_SCROLL_LINES: u16 = 3;
const LIST_SCROLL_ITEMS: usize = 1;

fn scroll_results(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.results_scroll.scroll_up(RESULTS_SCROLL_LINES),
        ScrollDirection::Down => app.results_scroll.scroll_down(RESULTS_SCROLL_LINES),
    }
}

fn scroll_help(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.help.current_scroll_mut().scroll_up(HELP_SCROLL_LINES),
        ScrollDirection::Down => app.help.current_scroll_mut().scroll_down(HELP_SCROLL_LINES),
    }
}

fn scroll_ai(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.ai.selection.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.ai.selection.scroll_view_down(LIST_SCROLL_ITEMS),
    }
}

fn scroll_snippets(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.snippets.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.snippets.scroll_view_down(LIST_SCROLL_ITEMS),
    }
}

fn scroll_history(app: &mut App, direction: ScrollDirection) {
    // History entries are displayed in reverse order (newest first at top)
    // so we invert the scroll direction to match visual expectation
    match direction {
        ScrollDirection::Up => app.history.scroll_view_down(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.history.scroll_view_up(LIST_SCROLL_ITEMS),
    }
}

fn scroll_autocomplete(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.autocomplete.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.autocomplete.scroll_view_down(LIST_SCROLL_ITEMS),
    }
}

#[cfg(test)]
#[path = "mouse_scroll_tests.rs"]
mod mouse_scroll_tests;
