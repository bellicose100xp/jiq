//! Mouse click handling
//!
//! Handles click events to change focus between UI components.

use super::app_state::{App, Focus};
use crate::editor::EditorMode;
use crate::layout::Region;

/// Handle left mouse button click for the given region
///
/// Routes click to change focus or activate components.
pub fn handle_click(app: &mut App, region: Option<Region>) {
    match region {
        Some(Region::ResultsPane) => click_results_pane(app),
        Some(Region::InputField) => click_input_field(app),
        Some(Region::SearchBar) => click_search_bar(app),
        // Other regions: no focus change behavior yet
        _ => {}
    }
}

fn click_results_pane(app: &mut App) {
    if app.focus != Focus::ResultsPane {
        app.focus = Focus::ResultsPane;
    }
}

fn click_input_field(app: &mut App) {
    if app.focus != Focus::InputField {
        app.focus = Focus::InputField;
        app.input.editor_mode = EditorMode::Insert;
    }
}

fn click_search_bar(app: &mut App) {
    if app.search.is_visible() && app.search.is_confirmed() {
        app.search.unconfirm();
    }
}

#[cfg(test)]
#[path = "mouse_click_tests.rs"]
mod mouse_click_tests;
