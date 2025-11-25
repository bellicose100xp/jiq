use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::editor::EditorMode;

/// Input field state
pub struct InputState {
    pub textarea: TextArea<'static>,
    pub editor_mode: EditorMode,
}

impl InputState {
    /// Create a new InputState
    pub fn new() -> Self {
        let mut textarea = TextArea::default();

        // Configure for single-line input
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        // Remove default underline from cursor line
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            editor_mode: EditorMode::default(),
        }
    }

    /// Get the current query text
    pub fn query(&self) -> &str {
        self.textarea.lines()[0].as_ref()
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input_state() {
        let state = InputState::new();
        assert_eq!(state.query(), "");
        assert_eq!(state.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_default() {
        let state = InputState::default();
        assert_eq!(state.query(), "");
    }

    #[test]
    fn test_query_after_insert() {
        let mut state = InputState::new();
        state.textarea.insert_str("test query");
        assert_eq!(state.query(), "test query");
    }
}
