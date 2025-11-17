use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io;

/// Application state
pub struct App {
    json_input: String,
    should_quit: bool,
}

impl App {
    /// Create a new App instance with JSON input
    pub fn new(json_input: String) -> Self {
        Self {
            json_input,
            should_quit: false,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Handle events and update application state
    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // Check that it's a key press event to avoid duplicates
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key press events
    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {
                // Other keys will be handled in future versions
            }
        }
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

        let content = Paragraph::new(self.json_input.as_str())
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
