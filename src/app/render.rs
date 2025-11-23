use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::autocomplete::SuggestionType;
use crate::editor::EditorMode;
use crate::syntax::JqHighlighter;
use super::state::{App, Focus};

// Autocomplete popup display constants
const MAX_VISIBLE_SUGGESTIONS: usize = 10;
const MAX_POPUP_WIDTH: usize = 60;
const POPUP_BORDER_HEIGHT: u16 = 2;
const POPUP_PADDING: u16 = 4;
const POPUP_OFFSET_X: u16 = 2;
const TYPE_LABEL_SPACING: usize = 3;

impl App {
    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Split the terminal into three areas: results, input, and help
        let layout = Layout::vertical([
            Constraint::Min(3),      // Results pane takes most of the space
            Constraint::Length(3),   // Input field is fixed 3 lines
            Constraint::Length(1),   // Help line at bottom
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];
        let help_area = layout[2];

        // Render results pane
        self.render_results_pane(frame, results_area);

        // Render input field
        self.render_input_field(frame, input_area);

        // Render help line
        self.render_help_line(frame, help_area);

        // Render autocomplete popup (if visible) - render last so it overlays other widgets
        if self.autocomplete.is_visible() {
            self.render_autocomplete_popup(frame, input_area);
        }
    }

    /// Render the input field (bottom)
    fn render_input_field(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Choose color based on mode
        let mode_color = match self.editor_mode {
            EditorMode::Insert => Color::Cyan,        // Cyan for Insert
            EditorMode::Normal => Color::Yellow,      // Yellow for Normal
            EditorMode::Operator(_) => Color::Green,  // Green for Operator
        };

        // Set border color - mode color when focused, gray when unfocused
        let border_color = if self.focus == Focus::InputField {
            mode_color
        } else {
            Color::DarkGray
        };

        // Build title with colored mode indicator and hint
        let mode_text = self.editor_mode.display();
        let title = match self.editor_mode {
            EditorMode::Normal => {
                Line::from(vec![
                    Span::raw(" Query ["),
                    Span::styled(mode_text, Style::default().fg(mode_color)),
                    Span::raw("] (press 'i' to edit) "),
                ])
            }
            _ => {
                Line::from(vec![
                    Span::raw(" Query ["),
                    Span::styled(mode_text, Style::default().fg(mode_color)),
                    Span::raw("] "),
                ])
            }
        };

        // Set cursor color based on mode
        let cursor_style = match self.editor_mode {
            EditorMode::Insert => Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED),
            EditorMode::Normal => Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED),
            EditorMode::Operator(_) => Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED),
        };
        self.textarea.set_cursor_style(cursor_style);

        // Update textarea block with mode-aware styling
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)),
        );

        // Render the textarea widget
        frame.render_widget(&self.textarea, area);

        // Render syntax highlighting overlay
        self.render_syntax_highlighting(frame, area);
    }

    /// Render syntax highlighting overlay on top of the textarea
    fn render_syntax_highlighting(&self, frame: &mut Frame, area: Rect) {
        // Get the query text
        let query = self.query();

        // Skip if empty
        if query.is_empty() {
            return;
        }

        // Highlight the query
        let highlighted_spans = JqHighlighter::highlight(query);

        // Create a line with highlighted spans
        let highlighted_line = Line::from(highlighted_spans);

        // Calculate the inner area (inside the border)
        // The border takes 1 character on each side
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Render the highlighted text without a block (transparent overlay)
        let paragraph = Paragraph::new(highlighted_line);
        frame.render_widget(paragraph, inner_area);
    }

    /// Render the results pane (top)
    fn render_results_pane(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::ResultsPane {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        match &self.query_result {
            Ok(result) => {
                // Store viewport height for page scrolling calculations (subtract borders)
                self.results_viewport_height = area.height.saturating_sub(2);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Results ")
                    .border_style(Style::default().fg(border_color));

                // Parse jq's ANSI color codes into Ratatui Text
                let colored_text = result
                    .as_bytes()
                    .to_vec()
                    .into_text()
                    .unwrap_or_else(|_| Text::raw(result)); // Fallback to plain text on parse error

                let content = Paragraph::new(colored_text)
                    .block(block)
                    .scroll((self.results_scroll, 0));

                frame.render_widget(content, area);
            }
            Err(error) => {
                // Split the area: error at top, last successful result below
                if let Some(last_result) = &self.last_successful_result {
                    // Calculate error height (number of lines + borders)
                    let error_lines = error.lines().count() as u16;
                    let error_height = (error_lines + 2).min(area.height / 3); // Max 1/3 of space

                    let split_layout = Layout::vertical([
                        Constraint::Length(error_height),
                        Constraint::Min(0),
                    ])
                    .split(area);

                    let error_area = split_layout[0];
                    let results_area = split_layout[1];

                    // Store viewport height for the results section (subtract borders)
                    self.results_viewport_height = results_area.height.saturating_sub(2);

                    // Render error section
                    let error_block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Error ")
                        .border_style(Style::default().fg(Color::Red));

                    let error_widget = Paragraph::new(error.as_str())
                        .block(error_block)
                        .style(Style::default().fg(Color::Red));

                    frame.render_widget(error_widget, error_area);

                    // Render last successful result section
                    let results_block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Results (last valid query) ")
                        .border_style(Style::default().fg(border_color));

                    // Parse cached result with colors
                    let colored_text = last_result
                        .as_bytes()
                        .to_vec()
                        .into_text()
                        .unwrap_or_else(|_| Text::raw(last_result));

                    let results_widget = Paragraph::new(colored_text)
                        .block(results_block)
                        .scroll((self.results_scroll, 0));

                    frame.render_widget(results_widget, results_area);
                } else {
                    // No cached result, just show error (fallback to original behavior)
                    self.results_viewport_height = area.height.saturating_sub(2);

                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Error ")
                        .border_style(Style::default().fg(Color::Red));

                    let content = Paragraph::new(error.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Red))
                        .scroll((self.results_scroll, 0));

                    frame.render_widget(content, area);
                }
            }
        }
    }

    /// Render the help line (bottom)
    fn render_help_line(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = " Tab: Autocomplete | Shift+Tab: Switch Focus | Enter: Exit with Results | Shift+Enter: Exit with Query | q: Quit";

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(help, area);
    }

    /// Render the autocomplete popup above the input field
    fn render_autocomplete_popup(&self, frame: &mut Frame, input_area: Rect) {
        let suggestions = self.autocomplete.suggestions();
        if suggestions.is_empty() {
            return;
        }

        // Calculate popup dimensions
        let visible_count = suggestions.len().min(MAX_VISIBLE_SUGGESTIONS);
        let popup_height = (visible_count as u16) + POPUP_BORDER_HEIGHT;

        // Calculate max width needed for suggestions
        let max_text_width = suggestions
            .iter()
            .map(|s| {
                let type_label = format!("[{}]", s.suggestion_type);
                s.text.len() + type_label.len() + TYPE_LABEL_SPACING
            })
            .max()
            .unwrap_or(20)
            .min(MAX_POPUP_WIDTH);
        let popup_width = (max_text_width as u16) + POPUP_PADDING;

        // Position popup just above the input field
        let popup_x = input_area.x + POPUP_OFFSET_X;
        let popup_y = input_area.y.saturating_sub(popup_height);

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width.min(input_area.width.saturating_sub(POPUP_PADDING)),
            height: popup_height.min(input_area.y), // Don't overflow above input
        };

        // Create list items with styling
        let items: Vec<ListItem> = suggestions
            .iter()
            .take(MAX_VISIBLE_SUGGESTIONS)
            .enumerate()
            .map(|(i, suggestion)| {
                let type_color = match suggestion.suggestion_type {
                    SuggestionType::Function => Color::Yellow,
                    SuggestionType::Field => Color::Cyan,
                    SuggestionType::Operator => Color::Magenta,
                    SuggestionType::Pattern => Color::Green,
                };

                let type_label = format!("[{}]", suggestion.suggestion_type);

                let line = if i == self.autocomplete.selected_index() {
                    // Highlight selected item
                    Line::from(vec![
                        Span::styled(
                            format!("► {} ", suggestion.text),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                                .add_modifier(Modifier::REVERSED),
                        ),
                        Span::styled(
                            type_label,
                            Style::default()
                                .fg(type_color)
                                .add_modifier(Modifier::REVERSED),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!("  {} ", suggestion.text),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(type_label, Style::default().fg(type_color)),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        // Create the list widget
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Suggestions ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

        frame.render_widget(list, popup_area);
    }
}
