//! Mouse scroll handling
//!
//! Routes scroll events to the appropriate component based on cursor position.

use super::app_state::App;
use crate::layout::Region;
use crate::scroll::Scrollable;

/// Scroll direction for mouse wheel and trackpad swipe events
///
/// Up/Down come from the vertical wheel; Left/Right come from a horizontal
/// two-finger trackpad swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    fn is_horizontal(self) -> bool {
        matches!(self, ScrollDirection::Left | ScrollDirection::Right)
    }
}

/// Handle scroll event for the given region
///
/// Routes scroll to the component under the cursor.
/// Falls back to results pane when cursor is outside all regions.
pub fn handle_scroll(app: &mut App, region: Option<Region>, direction: ScrollDirection) {
    app.double_click.reset();
    match region {
        Some(Region::ResultsPane) | Some(Region::BackButton) | None => {
            if direction.is_horizontal() {
                scroll_results_horizontal(app, direction)
            } else {
                scroll_results(app, direction)
            }
        }
        Some(Region::InputField) => scroll_input(app, direction),
        // List popups scroll vertically only; a horizontal swipe over them is a no-op.
        Some(Region::HelpPopup) if !direction.is_horizontal() => scroll_help(app, direction),
        Some(Region::AiWindow) if !direction.is_horizontal() => scroll_ai(app, direction),
        Some(Region::SnippetList) if !direction.is_horizontal() => scroll_snippets(app, direction),
        Some(Region::HistoryPopup) if !direction.is_horizontal() => scroll_history(app, direction),
        Some(Region::Autocomplete) if !direction.is_horizontal() => {
            scroll_autocomplete(app, direction)
        }
        // Non-scrollable regions and horizontal swipes over vertical lists: do nothing
        Some(Region::HelpPopup)
        | Some(Region::AiWindow)
        | Some(Region::SnippetList)
        | Some(Region::HistoryPopup)
        | Some(Region::Autocomplete)
        | Some(Region::SearchBar)
        | Some(Region::Tooltip)
        | Some(Region::ErrorOverlay)
        | Some(Region::SnippetPreview) => {}
    }
}

const RESULTS_SCROLL_LINES: u16 = 3;
const RESULTS_H_SCROLL_COLS: u16 = 3;
const HELP_SCROLL_LINES: u16 = 3;
const LIST_SCROLL_ITEMS: usize = 1;

fn scroll_results(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.results_scroll.scroll_up(RESULTS_SCROLL_LINES),
        ScrollDirection::Down => app.results_scroll.scroll_down(RESULTS_SCROLL_LINES),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

fn scroll_results_horizontal(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Left => app.results_scroll.scroll_left(RESULTS_H_SCROLL_COLS),
        ScrollDirection::Right => app.results_scroll.scroll_right(RESULTS_H_SCROLL_COLS),
        ScrollDirection::Up | ScrollDirection::Down => {}
    }
}

fn scroll_help(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.help.current_scroll_mut().scroll_up(HELP_SCROLL_LINES),
        ScrollDirection::Down => app.help.current_scroll_mut().scroll_down(HELP_SCROLL_LINES),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

fn scroll_ai(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.ai.selection.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.ai.selection.scroll_view_down(LIST_SCROLL_ITEMS),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

fn scroll_snippets(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.snippets.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.snippets.scroll_view_down(LIST_SCROLL_ITEMS),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

fn scroll_history(app: &mut App, direction: ScrollDirection) {
    // History entries are displayed in reverse order (newest first at top)
    // so we invert the scroll direction to match visual expectation
    match direction {
        ScrollDirection::Up => app.history.scroll_view_down(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.history.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

fn scroll_autocomplete(app: &mut App, direction: ScrollDirection) {
    match direction {
        ScrollDirection::Up => app.autocomplete.scroll_view_up(LIST_SCROLL_ITEMS),
        ScrollDirection::Down => app.autocomplete.scroll_view_down(LIST_SCROLL_ITEMS),
        ScrollDirection::Left | ScrollDirection::Right => {}
    }
}

const INPUT_SCROLL_CHARS: isize = 3;

fn scroll_input(app: &mut App, direction: ScrollDirection) {
    // The input field has only one axis to scroll (horizontal text pan), so both the
    // vertical wheel (Up/Down) and a horizontal swipe (Left/Right) map onto it. Up and
    // Left pan toward earlier characters; Down and Right pan toward later characters.
    let text_length = app.input.query().chars().count();
    match direction {
        ScrollDirection::Up | ScrollDirection::Left => app
            .input
            .scroll_horizontal(-INPUT_SCROLL_CHARS, text_length),
        ScrollDirection::Down | ScrollDirection::Right => {
            app.input.scroll_horizontal(INPUT_SCROLL_CHARS, text_length)
        }
    }
}

#[cfg(test)]
#[path = "mouse_scroll_tests.rs"]
mod mouse_scroll_tests;
