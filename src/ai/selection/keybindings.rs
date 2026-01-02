//! Keybinding handlers for AI suggestion selection
//!
//! Handles Alt+1-5 for direct selection, Alt+Up/Down/j/k for navigation,
//! and Enter for applying navigated selection.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::SelectionState;

/// Handle direct selection keybindings (Alt+1-5)
///
/// Parses Alt+1 through Alt+5 keybindings and validates the selection
/// index against the available suggestion count.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `suggestion_count` - Number of available suggestions
///
/// # Returns
/// * `Some(index)` - The 0-based index of the selected suggestion if valid
/// * `None` - If the key is not a selection key or the index is invalid
///
/// # Requirements
/// - 1.1-1.5: Alt+1-5 selects corresponding suggestion
/// - 2.1-2.4: Invalid selections are ignored
pub fn handle_direct_selection(key: KeyEvent, suggestion_count: usize) -> Option<usize> {
    // Only handle Alt+digit keys
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }

    // Parse digit from key code
    let digit = match key.code {
        KeyCode::Char('1') => 1,
        KeyCode::Char('2') => 2,
        KeyCode::Char('3') => 3,
        KeyCode::Char('4') => 4,
        KeyCode::Char('5') => 5,
        _ => return None,
    };

    // Convert to 0-based index
    let index = digit - 1;

    // Validate against suggestion count
    // Requirements 2.3, 2.4: Ignore if index >= suggestion_count
    if index < suggestion_count {
        Some(index)
    } else {
        None
    }
}

/// Handle navigation keybindings (Alt+Up/Down and Alt+j/k)
///
/// Parses Alt+Up/Down and Alt+j/k keybindings and updates the selection state
/// with wrapping behavior at boundaries.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `selection_state` - The selection state to update
/// * `suggestion_count` - Number of available suggestions
///
/// # Returns
/// * `true` - If the key was handled (Alt+Up/Down or Alt+j/k)
/// * `false` - If the key was not a navigation key
///
/// # Requirements
/// - 8.1: Alt+Down/j moves selection to next suggestion
/// - 8.2: Alt+Up/k moves selection to previous suggestion
/// - 8.3: Wraps to first suggestion when at the end
/// - 8.4: Wraps to last suggestion when at the beginning
pub fn handle_navigation(
    key: KeyEvent,
    selection_state: &mut SelectionState,
    suggestion_count: usize,
) -> bool {
    // Only handle Alt+arrow keys or Alt+j/k
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return false;
    }

    // No suggestions to navigate
    if suggestion_count == 0 {
        return false;
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            selection_state.navigate_next(suggestion_count);
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            selection_state.navigate_previous(suggestion_count);
            true
        }
        _ => false,
    }
}

/// Handle Enter key for applying navigated selection
///
/// Checks if navigation mode is active (user has used Alt+Up/Down/j/k) and
/// returns the selected index if so. This allows Enter to apply the
/// currently highlighted suggestion.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `selection_state` - The selection state to check
///
/// # Returns
/// * `Some(index)` - The 0-based index of the selected suggestion if navigation is active
/// * `None` - If Enter was not pressed or no suggestion is selected via navigation
///
/// # Requirements
/// - 9.1: Enter applies the highlighted suggestion when navigation is active
/// - 9.2: Does not interfere with normal Enter behavior when no navigation
/// - 9.3: Clears selection highlight after application (caller responsibility)
/// - 9.4: Does not interfere when popup has no suggestions
pub fn handle_apply_selection(key: KeyEvent, selection_state: &SelectionState) -> Option<usize> {
    // Only handle Enter key
    if key.code != KeyCode::Enter {
        return None;
    }

    // Only apply if navigation mode is active (user has used Alt+Up/Down/j/k)
    // Requirements 9.2, 9.4: Don't interfere with normal Enter behavior
    if !selection_state.is_navigation_active() {
        return None;
    }

    // Return the selected index
    selection_state.get_selected()
}

#[cfg(test)]
#[path = "keybindings_tests.rs"]
mod keybindings_tests;
