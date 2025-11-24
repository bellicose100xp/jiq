use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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

        // Render error overlay (if visible and error exists) - render last to overlay results
        if self.error_overlay_visible && self.query_result.is_err() {
            self.render_error_overlay(frame, results_area);
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
        let mut title_spans = match self.editor_mode {
            EditorMode::Normal => {
                vec![
                    Span::raw(" Query ["),
                    Span::styled(mode_text, Style::default().fg(mode_color)),
                    Span::raw("] (press 'i' to edit) "),
                ]
            }
            _ => {
                vec![
                    Span::raw(" Query ["),
                    Span::styled(mode_text, Style::default().fg(mode_color)),
                    Span::raw("] "),
                ]
            }
        };

        // Add error indicator if there's an error
        if self.query_result.is_err() {
            title_spans.push(Span::styled(
                "⚠ Syntax Error (Ctrl+E to view)",
                Style::default().fg(Color::Yellow),
            ));
        }

        let title = Line::from(title_spans);

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
            Err(_error) => {
                // When there's an error, show last successful result in full area (no splitting)
                // The error overlay will be rendered separately if user requests it with Ctrl+E
                self.results_viewport_height = area.height.saturating_sub(2);

                if let Some(last_result) = &self.last_successful_result {
                    // Render last successful result
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

                    frame.render_widget(results_widget, area);
                } else {
                    // No cached result, show empty results pane
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Results ")
                        .border_style(Style::default().fg(border_color));

                    let empty_text = Text::from("");
                    let content = Paragraph::new(empty_text).block(block);

                    frame.render_widget(content, area);
                }
            }
        }
    }

    /// Render the error overlay (floating at the bottom of results pane)
    fn render_error_overlay(&self, frame: &mut Frame, results_area: Rect) {
        // Only render if there's an error
        if let Err(error) = &self.query_result {
            // Truncate error to max 5 lines of content
            let error_lines: Vec<&str> = error.lines().collect();
            let max_content_lines = 5;
            let (display_error, truncated) = if error_lines.len() > max_content_lines {
                let truncated_lines = &error_lines[..max_content_lines];
                let mut display = truncated_lines.join("\n");
                display.push_str("\n... (error truncated)");
                (display, true)
            } else {
                (error.clone(), false)
            };

            // Calculate overlay height (content lines + borders)
            let content_lines = if truncated { max_content_lines + 1 } else { error_lines.len() };
            let overlay_height = (content_lines as u16 + 2).min(7).max(3); // Min 3, max 7

            // Position overlay at bottom of results pane, with 1 line gap from bottom border
            let overlay_y = results_area.bottom().saturating_sub(overlay_height + 1);

            // Create overlay area with margins
            let overlay_area = Rect {
                x: results_area.x + 2,  // 2-char left margin
                y: overlay_y,
                width: results_area.width.saturating_sub(4),  // 2-char margins on both sides
                height: overlay_height,
            };

            // Clear the background to make it truly floating
            frame.render_widget(Clear, overlay_area);

            // Render error overlay with distinct styling
            let error_block = Block::default()
                .borders(Borders::ALL)
                .title(" Syntax Error (Ctrl+E to close) ")
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black));

            let error_widget = Paragraph::new(display_error.as_str())
                .block(error_block)
                .style(Style::default().fg(Color::Red));

            frame.render_widget(error_widget, overlay_area);
        }
    }

    /// Render the help line (bottom)
    fn render_help_line(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = " Tab: Autocomplete | Shift+Tab: Switch Focus | Enter: Exit with Results | Ctrl+Q: Exit with Query | q: Quit";

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
                // Calculate actual type label length including field type if present
                let type_label_len = match &s.suggestion_type {
                    SuggestionType::Field => {
                        if let Some(field_type) = &s.field_type {
                            // Format: "[field: TypeName]" = "[field: " (8) + TypeName + "]" (1)
                            9 + field_type.to_string().len()
                        } else {
                            7 // "[field]"
                        }
                    }
                    _ => {
                        // Other types: "[fn]", "[op]", "[pat]"
                        s.suggestion_type.to_string().len() + 2 // "[]" wrapping
                    }
                };
                s.text.len() + type_label_len + TYPE_LABEL_SPACING
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

        // Calculate max field text width for alignment
        let max_field_width = suggestions
            .iter()
            .take(MAX_VISIBLE_SUGGESTIONS)
            .map(|s| s.text.len())
            .max()
            .unwrap_or(0);

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

                let type_label = match &suggestion.suggestion_type {
                    SuggestionType::Field => {
                        if let Some(field_type) = &suggestion.field_type {
                            format!("[field: {}]", field_type)
                        } else {
                            format!("[{}]", suggestion.suggestion_type)
                        }
                    }
                    _ => format!("[{}]", suggestion.suggestion_type),
                };

                // Calculate padding to align type labels
                let padding_needed = max_field_width.saturating_sub(suggestion.text.len());
                let padding = " ".repeat(padding_needed);

                let line = if i == self.autocomplete.selected_index() {
                    // Highlight selected item with high contrast colors
                    Line::from(vec![
                        Span::styled(
                            format!("► {} {}", suggestion.text, padding),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(" {}", type_label),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!("  {} {}", suggestion.text, padding),
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Black),
                        ),
                        Span::styled(
                            format!(" {}", type_label),
                            Style::default()
                                .fg(type_color)
                                .bg(Color::Black),
                        ),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        // Clear the background area to prevent transparency
        frame.render_widget(Clear, popup_area);

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
