use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Application state
pub struct App {
    should_quit: bool,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Render the UI
    pub fn render(&self, frame: &mut Frame) {
        // Split the terminal into two panes: results (top) and input (bottom)
        let layout = Layout::vertical([
            Constraint::Min(3),      // Results pane takes most of the space
            Constraint::Length(3),   // Input field is fixed 3 lines
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];

        // Render results pane with hardcoded content
        self.render_results_pane(frame, results_area);

        // Render input field with hardcoded content
        self.render_input_field(frame, input_area);
    }

    /// Render the results pane (top)
    fn render_results_pane(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Results ")
            .border_style(Style::default().fg(Color::Cyan));

        let content = Paragraph::new("Hello World!\n\nThis is the results pane.\nFiltered JSON will appear here.")
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(content, area);
    }

    /// Render the input field (bottom)
    fn render_input_field(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Query ")
            .border_style(Style::default().fg(Color::DarkGray));

        let content = Paragraph::new("(query input will go here)")
            .block(block)
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(content, area);
    }
}
