use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::state::{App, Focus};

impl App {
    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Split the terminal into two panes: results (top) and input (bottom)
        let layout = Layout::vertical([
            Constraint::Min(3),      // Results pane takes most of the space
            Constraint::Length(3),   // Input field is fixed 3 lines
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];

        // Render results pane
        self.render_results_pane(frame, results_area);

        // Render input field
        self.render_input_field(frame, input_area);
    }

    /// Render the input field (bottom)
    fn render_input_field(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::InputField {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        // Update textarea block with focus-aware styling
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(border_color)),
        );

        // Render the textarea widget
        frame.render_widget(&self.textarea, area);
    }

    /// Render the results pane (top)
    fn render_results_pane(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::ResultsPane {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Results ")
            .border_style(Style::default().fg(border_color));

        // Display query results or error message
        let (text, style) = match &self.query_result {
            Ok(result) => {
                // Use default style to preserve jq's ANSI color codes
                (result.as_str(), Style::default())
            }
            Err(error) => {
                // Use red color for error messages
                (error.as_str(), Style::default().fg(Color::Red))
            }
        };

        let content = Paragraph::new(text)
            .block(block)
            .style(style)
            .scroll((self.results_scroll, 0)); // Apply vertical scroll

        frame.render_widget(content, area);
    }
}
