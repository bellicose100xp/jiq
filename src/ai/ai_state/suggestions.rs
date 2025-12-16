//! AI suggestion management
//!
//! Handles suggestion selection, navigation, and state updates.
//! Currently minimal as most suggestion logic is in lifecycle methods.

use crate::ai::ai_state::AiState;

impl AiState {
    // Note: Suggestion management methods are currently minimal.
    // Most suggestion handling is done in lifecycle methods:
    // - start_request() clears suggestions
    // - complete_request() parses suggestions
    //
    // Future expansion could include:
    // - select_suggestion()
    // - navigate_suggestions()
    // - filter_suggestions()
    // - etc.
}
