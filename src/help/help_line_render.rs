use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;

pub fn render_line(app: &App, frame: &mut Frame, area: Rect) {
    let help_text = if app.focus == Focus::InputField && app.input.editor_mode == EditorMode::Insert
    {
        let query_empty = app.query().is_empty();
        if query_empty {
            " F1: Help | Shift+Tab: Switch Pane | Ctrl+P/N: Cycle History | ↑/Ctrl+R: History"
        } else {
            " F1: Help | Shift+Tab: Switch Pane | ↑/Ctrl+R: History | Enter: Output Result | Ctrl+Q: Output Query"
        }
    } else {
        " F1/?: Help | Shift+Tab: Switch Pane | Enter: Output Result | Ctrl+Q: Output Query | q: Quit"
    };

    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, area);
}
