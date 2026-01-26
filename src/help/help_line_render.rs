use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

use crate::app::{App, Focus};
use crate::editor::EditorMode;
use crate::theme;

pub fn render_line(app: &App, frame: &mut Frame, area: Rect) {
    let help_text = if app.search.is_visible() {
        if app.search.is_confirmed() {
            " F1/?: Help | Esc: Close | n/N: Next/Prev | /: Edit Search"
        } else {
            " F1/?: Help | Esc: Close | Enter: Confirm Search"
        }
    } else if app.snippets.is_visible() {
        " F1/?: Help | Esc: Close"
    } else if app.focus == Focus::InputField && app.input.editor_mode == EditorMode::Insert {
        let query_empty = app.query().is_empty();
        if query_empty {
            " F1: Help | Shift+Tab: Switch Pane | Ctrl+S: Snippets | Ctrl+P/N: Cycle History | ↑/Ctrl+R: History"
        } else {
            " F1: Help | Shift+Tab: Switch Pane | Ctrl+S: Snippets | ↑/Ctrl+R: History | Enter: Output Result | Ctrl+Q: Output Query"
        }
    } else {
        " F1/?: Help | Shift+Tab: Switch Pane | Ctrl+S: Snippets | Enter: Output Result | Ctrl+Q: Output Query | q: Quit"
    };

    let help = Paragraph::new(help_text).style(Style::default().fg(theme::help_line::TEXT));

    frame.render_widget(help, area);
}

#[cfg(test)]
#[path = "help_line_render_tests.rs"]
mod help_line_render_tests;
