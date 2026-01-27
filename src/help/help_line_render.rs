use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, Focus};
use crate::editor::EditorMode;
use crate::theme;

macro_rules! hints {
    ($($key:literal => $desc:literal),+ $(,)?) => {
        vec![$(($key, $desc)),+]
    };
}

fn get_context_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    if app.search.is_visible() {
        if app.search.is_confirmed() {
            hints!["F1/?" => "Help", "Esc" => "Close", "n/N" => "Next/Prev", "Ctrl+F" => "Edit Search", "/" => "Edit Search"]
        } else {
            hints!["F1/?" => "Help", "Esc" => "Close", "Enter" => "Confirm Search"]
        }
    } else if app.snippets.is_visible() {
        hints!["F1/?" => "Help", "Esc" => "Close"]
    } else if app.focus == Focus::InputField && app.input.editor_mode == EditorMode::Insert {
        hints!["F1" => "Help", "Shift+Tab" => "Navigate Results", "Ctrl+S" => "Snippets", "Ctrl+F" => "Search", "Ctrl+P/N" => "Cycle History", "Ctrl+R" => "History", "Ctrl+C" => "Quit"]
    } else if app.focus == Focus::ResultsPane {
        hints!["F1/?" => "Help", "Shift+Tab" => "Edit Query", "Ctrl+S" => "Snippets", "Ctrl+F" => "Search", "Ctrl+C" => "Quit"]
    } else {
        hints!["F1/?" => "Help", "Shift+Tab" => "Navigate Results", "Ctrl+S" => "Snippets", "Ctrl+F" => "Search", "Ctrl+C" => "Quit"]
    }
}

fn build_styled_spans(hints: &[(&'static str, &'static str)]) -> Vec<Span<'static>> {
    let key_style = Style::default().fg(theme::help_line::KEY);
    let desc_style = Style::default().fg(theme::help_line::DESCRIPTION);
    let sep_style = Style::default().fg(theme::help_line::SEPARATOR);

    let mut spans = Vec::with_capacity(hints.len() * 4 + 1);
    spans.push(Span::raw(" "));

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" \u{2022} ", sep_style));
        }
        spans.push(Span::styled(*key, key_style));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, desc_style));
    }

    spans
}

pub fn render_line(app: &App, frame: &mut Frame, area: Rect) {
    let hints = get_context_hints(app);
    let spans = build_styled_spans(&hints);
    let help = Paragraph::new(Line::from(spans));
    frame.render_widget(help, area);
}

#[cfg(test)]
#[path = "help_line_render_tests.rs"]
mod help_line_render_tests;
