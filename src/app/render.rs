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
use crate::history::MAX_VISIBLE_HISTORY;
use crate::notification::render_notification;
use crate::widgets::popup;
use crate::help::{HELP_ENTRIES, HELP_FOOTER};
use super::state::{App, Focus};

// Autocomplete popup display constants
const MAX_VISIBLE_SUGGESTIONS: usize = 10;
const MAX_POPUP_WIDTH: usize = 60;
const POPUP_BORDER_HEIGHT: u16 = 2;
const POPUP_PADDING: u16 = 4;
const POPUP_OFFSET_X: u16 = 2;
const TYPE_LABEL_SPACING: usize = 3;

// History popup display constants
const HISTORY_SEARCH_HEIGHT: u16 = 3;

// Help popup display constants
const HELP_POPUP_WIDTH: u16 = 70;
const HELP_POPUP_PADDING: u16 = 4; // borders (2) + footer (2)

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

        // Render history popup (if visible) - overlays autocomplete
        if self.history.is_visible() {
            self.render_history_popup(frame, input_area);
        }

        // Render error overlay (if visible and error exists) - render last to overlay results
        if self.error_overlay_visible && self.query.result.is_err() {
            self.render_error_overlay(frame, results_area);
        }

        // Render help popup (if visible) - render last to overlay everything
        if self.help.visible {
            self.render_help_popup(frame);
        }

        // Render notification overlay (if active) - render last to overlay everything
        render_notification(frame, &mut self.notification);
    }

    /// Render the input field (bottom)
    fn render_input_field(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Calculate viewport width (inside borders) and update scroll offset
        let viewport_width = area.width.saturating_sub(2) as usize;
        self.input.calculate_scroll_offset(viewport_width);

        // Choose color based on mode
        let mode_color = match self.input.editor_mode {
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
        let mode_text = self.input.editor_mode.display();
        let mut title_spans = match self.input.editor_mode {
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
        if self.query.result.is_err() {
            title_spans.push(Span::styled(
                "⚠ Syntax Error (Ctrl+E to view)",
                Style::default().fg(Color::Yellow),
            ));
        }

        let title = Line::from(title_spans);

        // Create block with mode-aware styling
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));

        // Get query text and render with syntax highlighting + cursor
        let query = self.query();
        let cursor_col = self.input.textarea.cursor().1;
        let scroll_offset = self.input.scroll_offset;

        if query.is_empty() {
            // Empty query - just show cursor
            use crate::syntax_highlight::overlay::insert_cursor_into_spans;
            let cursor_spans = insert_cursor_into_spans(vec![], 0);
            let paragraph = Paragraph::new(Line::from(cursor_spans)).block(block);
            frame.render_widget(paragraph, area);
        } else {
            // Render styled text with cursor
            use crate::syntax_highlight::JqHighlighter;
            use crate::syntax_highlight::overlay::{insert_cursor_into_spans, extract_visible_spans};

            let highlighted_spans = JqHighlighter::highlight(query);

            // Extract visible portion based on scroll offset
            let visible_spans = extract_visible_spans(
                &highlighted_spans,
                scroll_offset,
                viewport_width,
            );

            // Insert cursor at the correct position (relative to visible area)
            let cursor_in_viewport = cursor_col.saturating_sub(scroll_offset);
            let spans_with_cursor = insert_cursor_into_spans(visible_spans, cursor_in_viewport);

            let paragraph = Paragraph::new(Line::from(spans_with_cursor)).block(block);
            frame.render_widget(paragraph, area);
        }
    }

    /// Render the results pane (top)
    fn render_results_pane(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::ResultsPane {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        // Determine title text and style based on error state
        let (title_text, title_style) = if self.query.result.is_err() {
            (
                " ⚠ Syntax Error (last successful query result) ",
                Style::default().fg(Color::Yellow)
            )
        } else {
            (
                " Results ",
                Style::default().fg(border_color)
            )
        };

        match &self.query.result {
            Ok(result) => {
                // Update scroll bounds based on content and viewport
                let viewport_height = area.height.saturating_sub(2);
                let viewport_width = area.width.saturating_sub(2);
                let line_count = self.results_line_count_u32();
                self.results_scroll.update_bounds(line_count, viewport_height);
                self.results_scroll
                    .update_h_bounds(self.query.max_line_width(), viewport_width);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(title_text, title_style))
                    .border_style(Style::default().fg(border_color));

                // Parse jq's ANSI color codes into Ratatui Text
                let colored_text = result
                    .as_bytes()
                    .to_vec()
                    .into_text()
                    .unwrap_or_else(|_| Text::raw(result)); // Fallback to plain text on parse error

                let content = Paragraph::new(colored_text)
                    .block(block)
                    .scroll((self.results_scroll.offset, self.results_scroll.h_offset));

                frame.render_widget(content, area);
            }
            Err(_error) => {
                // When there's an error, show last successful result in full area (no splitting)
                // The error overlay will be rendered separately if user requests it with Ctrl+E
                let viewport_height = area.height.saturating_sub(2);
                let viewport_width = area.width.saturating_sub(2);
                let line_count = self.results_line_count_u32();
                self.results_scroll.update_bounds(line_count, viewport_height);
                self.results_scroll
                    .update_h_bounds(self.query.max_line_width(), viewport_width);

                if let Some(last_result) = &self.query.last_successful_result {
                    // Render last successful result with error title
                    let results_block = Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled(title_text, title_style))
                        .border_style(Style::default().fg(border_color));

                    // Parse cached result with colors
                    let colored_text = last_result
                        .as_bytes()
                        .to_vec()
                        .into_text()
                        .unwrap_or_else(|_| Text::raw(last_result));

                    let results_widget = Paragraph::new(colored_text)
                        .block(results_block)
                        .scroll((self.results_scroll.offset, self.results_scroll.h_offset));

                    frame.render_widget(results_widget, area);
                } else {
                    // No cached result, show empty results pane with error title
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled(title_text, title_style))
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
        if let Err(error) = &self.query.result {
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
            let overlay_height = (content_lines as u16 + 2).clamp(3, 7); // Min 3, max 7

            // Position overlay at bottom of results pane, with 1 line gap from bottom border
            let overlay_y = results_area.bottom().saturating_sub(overlay_height + 1);

            // Create overlay area with margins and position at bottom
            let overlay_with_margins = popup::inset_rect(results_area, 2, 0);
            let overlay_area = Rect {
                x: overlay_with_margins.x,
                y: overlay_y,
                width: overlay_with_margins.width,
                height: overlay_height,
            };

            // Clear the background to make it truly floating
            popup::clear_area(frame, overlay_area);

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
        // Mode-aware help text: in Insert mode 'q' and '?' type characters
        let help_text = if self.focus == Focus::InputField && self.input.editor_mode == EditorMode::Insert {
            let query_empty = self.query().is_empty();
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
        // Use signature for functions if available, otherwise use text
        let max_text_width = suggestions
            .iter()
            .map(|s| {
                // Get display text: signature for functions, text for others
                let display_text_len = match s.suggestion_type {
                    SuggestionType::Function => {
                        s.signature.as_ref().map(|sig| sig.len()).unwrap_or(s.text.len())
                    }
                    _ => s.text.len(),
                };

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
                display_text_len + type_label_len + TYPE_LABEL_SPACING
            })
            .max()
            .unwrap_or(20)
            .min(MAX_POPUP_WIDTH);
        let popup_width = (max_text_width as u16) + POPUP_PADDING;

        // Position popup just above the input field
        let popup_area = popup::popup_above_anchor(input_area, popup_width, popup_height, POPUP_OFFSET_X);

        // Calculate max display text width for alignment
        // Use signature for functions if available, otherwise use text
        let max_display_width = suggestions
            .iter()
            .take(MAX_VISIBLE_SUGGESTIONS)
            .map(|s| match s.suggestion_type {
                SuggestionType::Function => {
                    s.signature.as_ref().map(|sig| sig.len()).unwrap_or(s.text.len())
                }
                _ => s.text.len(),
            })
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

                // Get display text: signature for functions, text for others
                let display_text = match suggestion.suggestion_type {
                    SuggestionType::Function => {
                        suggestion.signature.as_deref().unwrap_or(&suggestion.text)
                    }
                    _ => &suggestion.text,
                };

                // Calculate padding to align type labels
                let padding_needed = max_display_width.saturating_sub(display_text.len());
                let padding = " ".repeat(padding_needed);

                let line = if i == self.autocomplete.selected_index() {
                    // Highlight selected item with high contrast colors
                    Line::from(vec![
                        Span::styled(
                            format!("► {} {}", display_text, padding),
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
                            format!("  {} {}", display_text, padding),
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
        popup::clear_area(frame, popup_area);

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

    /// Render the history popup above the input field
    fn render_history_popup(&mut self, frame: &mut Frame, input_area: Rect) {
        // Calculate dimensions - ensure minimum 1 row for "No matches" message
        let visible_count = self.history.filtered_count().min(MAX_VISIBLE_HISTORY);
        let list_height = (visible_count as u16).max(1) + 2; // +2 for borders, min 1 row
        let total_height = list_height + HISTORY_SEARCH_HEIGHT;

        // Position popup above input (full width)
        let popup_y = input_area.y.saturating_sub(total_height);

        let popup_area = Rect {
            x: input_area.x,
            y: popup_y,
            width: input_area.width,
            height: total_height.min(input_area.y),
        };

        // Clear background
        popup::clear_area(frame, popup_area);

        // Split into list area and search area
        let layout = Layout::vertical([
            Constraint::Min(3),           // History list
            Constraint::Length(HISTORY_SEARCH_HEIGHT), // Search box
        ])
        .split(popup_area);

        let list_area = layout[0];
        let search_area = layout[1];

        // Build title with match count
        let title = format!(
            " History ({}/{}) ",
            self.history.filtered_count(),
            self.history.total_count()
        );

        // Calculate max text length based on available width
        // Format: " ► text " with borders -> overhead = 6 chars (borders + padding + arrow)
        let max_text_len = (list_area.width as usize).saturating_sub(6);

        // Create list items
        let items: Vec<ListItem> = if self.history.filtered_count() == 0 {
            // Show "No matches" when search has no results
            vec![ListItem::new(Line::from(Span::styled(
                "   No matches",
                Style::default().fg(Color::DarkGray),
            )))]
        } else {
            self.history
                .visible_entries()
                .map(|(display_idx, entry)| {
                    // Truncate long entries (char-safe for UTF-8)
                    let display_text = if entry.chars().count() > max_text_len {
                        let truncated: String = entry.chars().take(max_text_len).collect();
                        format!("{}…", truncated)
                    } else {
                        entry.to_string()
                    };

                    let line = if display_idx == self.history.selected_index() {
                        // Selected item
                        Line::from(vec![Span::styled(
                            format!(" ► {} ", display_text),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else {
                        // Unselected item
                        Line::from(vec![Span::styled(
                            format!("   {} ", display_text),
                            Style::default().fg(Color::White).bg(Color::Black),
                        )])
                    };

                    ListItem::new(line)
                })
                .collect()
        };

        // Render list
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(list, list_area);

        // Render search box using TextArea
        let search_textarea = self.history.search_textarea_mut();
        search_textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );
        search_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
        frame.render_widget(&*search_textarea, search_area);
    }

    /// Render the help popup (centered modal with keyboard shortcuts)
    fn render_help_popup(&mut self, frame: &mut Frame) {
        // Calculate popup dimensions
        let content_height = HELP_ENTRIES.len() as u16;
        let ideal_popup_height = content_height + HELP_POPUP_PADDING;
        let ideal_popup_width = HELP_POPUP_WIDTH;

        // Clamp dimensions to fit within the frame
        let frame_area = frame.area();
        let popup_width = ideal_popup_width.min(frame_area.width);
        let popup_height = ideal_popup_height.min(frame_area.height);

        // Don't render if terminal is too small
        if frame_area.width < 20 || frame_area.height < 10 {
            return;
        }

        // Center the popup
        let popup_area = popup::centered_popup(frame_area, popup_width, popup_height);

        // Clear the background for floating effect
        popup::clear_area(frame, popup_area);

        // Create help text with proper formatting
        let mut lines: Vec<Line> = Vec::new();

        for (key, desc) in HELP_ENTRIES {
            if key.is_empty() && desc.is_empty() {
                // Empty line for spacing
                lines.push(Line::from(""));
            } else if key.is_empty() {
                // Category header (bold, cyan)
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(*desc, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                // Key-description pair
                let key_span = Span::styled(
                    format!("  {:<15}", key),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                );
                let desc_span = Span::styled(*desc, Style::default().fg(Color::White));
                lines.push(Line::from(vec![key_span, desc_span]));
            }
        }

        // Add footer
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("           {}          ", HELP_FOOTER),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        let help_text = Text::from(lines.clone());

        // Update scroll bounds based on content and viewport
        let content_height = lines.len() as u32;
        let visible_height = popup_height.saturating_sub(2); // -2 for borders
        self.help.scroll.update_bounds(content_height, visible_height);

        // Create the popup widget with scroll
        let popup = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts ")
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Black)),
            )
            .scroll((self.help.scroll.offset, 0));

        frame.render_widget(popup, popup_area);
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Create a test terminal with specified dimensions
    pub fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    /// Render an App to a test terminal and return the buffer as a string
    pub fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal.draw(|f| app.render(f)).unwrap();
        terminal.backend().to_string()
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use super::test_helpers::render_to_string;
    use crate::app::state::App;
    use crate::config::ClipboardBackend;
    use crate::editor::EditorMode;
    use crate::history::HistoryState;
    use insta::assert_snapshot;

    const TEST_WIDTH: u16 = 80;
    const TEST_HEIGHT: u16 = 24;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), ClipboardBackend::Auto)
    }

    // === Basic UI Layout Tests ===

    #[test]
    fn snapshot_initial_ui_empty_query() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_with_query() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_with_array_data() {
        let json = r#"[{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".[].name");
        app.query.execute(".[].name");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Focus State Tests ===

    #[test]
    fn snapshot_ui_input_focused() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_results_focused() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Editor Mode Tests ===

    #[test]
    fn snapshot_ui_insert_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Insert;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_normal_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Normal;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_operator_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Operator('d');

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Error State Tests ===

    #[test]
    fn snapshot_ui_with_error() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_error_overlay_visible() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");
        app.error_overlay_visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Terminal Size Tests ===

    #[test]
    fn snapshot_ui_small_terminal() {
        let json = r#"{"name": "Alice"}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, 40, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_wide_terminal() {
        let json = r#"{"name": "Alice"}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, 120, 30);
        assert_snapshot!(output);
    }

    // === Popup/Overlay Tests ===

    #[test]
    fn snapshot_history_popup() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Add some history entries (using test helper)
        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.add_entry_in_memory(".age");
        app.history.add_entry_in_memory(".users[]");
        app.history.open(None);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_history_popup_with_search() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.add_entry_in_memory(".age");
        app.history.add_entry_in_memory(".users[]");
        app.history.open(Some("na"));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_history_popup_no_matches() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.open(Some("xyz"));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_help_popup() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.help.visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_error_overlay() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Create an error state
        app.query.result = Err("jq: compile error: syntax error at line 1".to_string());
        app.error_overlay_visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Error State Title Tests ===

    #[test]
    fn snapshot_results_pane_with_syntax_error_unfocused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Execute a successful query first to populate last_successful_result
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");
        
        // Now create an error state
        app.input.textarea.delete_line_by_head();
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");
        
        // Ensure results pane is unfocused
        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_syntax_error_focused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Execute a successful query first to populate last_successful_result
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");
        
        // Now create an error state
        app.input.textarea.delete_line_by_head();
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");
        
        // Focus the results pane to verify cyan border with yellow title
        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_success_unfocused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Execute a successful query
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");
        
        // Ensure results pane is unfocused
        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_success_focused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Execute a successful query
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");
        
        // Focus the results pane to verify cyan border and title
        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    // === Autocomplete Popup Tests ===

    #[test]
    fn snapshot_autocomplete_popup_with_function_signatures() {
        use crate::autocomplete::{Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Set up autocomplete with function suggestions that have signatures
        let suggestions = vec![
            Suggestion::new("select", SuggestionType::Function)
                .with_description("Filter elements by condition")
                .with_signature("select(expr)")
                .with_needs_parens(true),
            Suggestion::new("sort", SuggestionType::Function)
                .with_description("Sort array")
                .with_signature("sort"),
            Suggestion::new("sort_by", SuggestionType::Function)
                .with_description("Sort array by expression")
                .with_signature("sort_by(expr)")
                .with_needs_parens(true),
        ];
        app.autocomplete.update_suggestions(suggestions);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_autocomplete_popup_selected_item_with_signature() {
        use crate::autocomplete::{Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Set up autocomplete with function suggestions
        let suggestions = vec![
            Suggestion::new("map", SuggestionType::Function)
                .with_description("Apply expression to each element")
                .with_signature("map(expr)")
                .with_needs_parens(true),
            Suggestion::new("max", SuggestionType::Function)
                .with_description("Maximum value")
                .with_signature("max"),
            Suggestion::new("max_by", SuggestionType::Function)
                .with_description("Maximum by expression")
                .with_signature("max_by(expr)")
                .with_needs_parens(true),
        ];
        app.autocomplete.update_suggestions(suggestions);
        
        // Select the second item (max)
        app.autocomplete.select_next();

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_autocomplete_popup_mixed_types() {
        use crate::autocomplete::{Suggestion, SuggestionType, JsonFieldType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Set up autocomplete with mixed suggestion types
        let suggestions = vec![
            Suggestion::new("keys", SuggestionType::Function)
                .with_description("Get object keys or array indices")
                .with_signature("keys"),
            Suggestion::new_with_type("name", SuggestionType::Field, Some(JsonFieldType::String))
                .with_description("String field"),
            Suggestion::new(".[]", SuggestionType::Pattern)
                .with_description("Iterate over array/object values"),
        ];
        app.autocomplete.update_suggestions(suggestions);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }
}

