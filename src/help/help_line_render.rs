//! Help line rendering
//!
//! This module handles rendering of the help line at the bottom of the screen.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;

/// Render the help line (bottom of screen)
pub fn render_line(app: &App, frame: &mut Frame, area: Rect) {
    // Mode-aware help text: in Insert mode 'q' and '?' type characters
    let help_text = if app.focus == Focus::InputField && app.input.editor_mode == EditorMode::Insert {
        let query_empty = app.query().is_empty();
        if query_empty {
            // Empty input: show history navigation
            " F1: Help | Shift+Tab: Switch Pane | Ctrl+P/N: Cycle History | ↑/Ctrl+R: History"
        } else {
            // Has content: show exit shortcuts + history
            " F1: Help | Shift+Tab: Switch Pane | ↑/Ctrl+R: History | Enter: Output Result | Ctrl+Q: Output Query"
        }
    } else {
        " F1/?: Help | Shift+Tab: Switch Pane | Enter: Output Result | Ctrl+Q: Output Query | q: Quit"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, area);
}
