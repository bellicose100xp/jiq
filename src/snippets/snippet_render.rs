use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::snippet_state::{SnippetMode, SnippetState};
use crate::ai::render::text::wrap_text;
use crate::syntax_highlight::JqHighlighter;
use crate::theme;
use crate::widgets::{popup, scrollbar};

const MIN_LIST_HEIGHT: u16 = 3;
const SEARCH_HEIGHT: u16 = 3;
const NAME_INPUT_HEIGHT: u16 = 3;
const DESCRIPTION_INPUT_HEIGHT: u16 = 3;
const QUERY_INPUT_HEIGHT: u16 = 3;
const HINTS_HEIGHT: u16 = 3;

fn build_browse_hints() -> Line<'static> {
    theme::border_hints::build_hints(
        &[
            ("↑/↓", "Navigate"),
            ("Enter", "Apply"),
            ("Ctrl+N", "New"),
            ("Ctrl+E", "Edit"),
            ("Ctrl+R", "Replace"),
            ("Ctrl+D", "Delete"),
            ("Esc", "Close"),
        ],
        theme::snippets::BORDER,
    )
}

fn build_form_hints(action: &'static str) -> Line<'static> {
    theme::border_hints::build_hints(
        &[
            ("Enter", action),
            ("Tab", "Next"),
            ("Shift+Tab", "Prev"),
            ("Esc", "Cancel"),
        ],
        theme::snippets::FIELD_ACTIVE_BORDER,
    )
}

fn build_confirm_hints() -> Line<'static> {
    theme::border_hints::build_hints(
        &[("Enter", "Confirm"), ("Esc", "Cancel")],
        theme::snippets::FIELD_ACTIVE_BORDER,
    )
}

/// Render the snippet manager popup
///
/// Returns (list_area, preview_area) for region tracking.
/// Only returns areas when in browse mode.
pub fn render_popup(
    state: &mut SnippetState,
    frame: &mut Frame,
    results_area: Rect,
) -> (Option<Rect>, Option<Rect>) {
    popup::clear_area(frame, results_area);

    match state.mode() {
        SnippetMode::Browse => render_browse_mode(state, frame, results_area),
        SnippetMode::CreateName | SnippetMode::CreateQuery | SnippetMode::CreateDescription => {
            render_create_mode(state, frame, results_area);
            (None, None)
        }
        SnippetMode::EditName { .. }
        | SnippetMode::EditQuery { .. }
        | SnippetMode::EditDescription { .. } => {
            render_edit_mode(state, frame, results_area);
            (None, None)
        }
        SnippetMode::ConfirmDelete { .. } => {
            render_confirm_delete_mode(state, frame, results_area);
            (None, None)
        }
        SnippetMode::ConfirmUpdate { .. } => {
            render_confirm_update_mode(state, frame, results_area);
            (None, None)
        }
    }
}

fn render_browse_mode(
    state: &mut SnippetState,
    frame: &mut Frame,
    results_area: Rect,
) -> (Option<Rect>, Option<Rect>) {
    let selected_snippet = state.selected_snippet().cloned();
    let total_count = state.snippets().len();
    let filtered_count = state.filtered_count();

    let inner_width = results_area.width.saturating_sub(4) as usize;
    let preview_content_height = calculate_preview_height(selected_snippet.as_ref(), inner_width);
    let preview_height = (preview_content_height as u16 + 2).min(results_area.height / 2);

    let min_required = SEARCH_HEIGHT + MIN_LIST_HEIGHT + preview_height;
    if results_area.height < min_required {
        let visible_count = results_area.height.saturating_sub(SEARCH_HEIGHT + 2) as usize;
        state.set_visible_count(visible_count.max(1));
        return render_minimal(state, filtered_count, total_count, frame, results_area);
    }

    let layout = Layout::vertical([
        Constraint::Length(SEARCH_HEIGHT),
        Constraint::Min(MIN_LIST_HEIGHT),
        Constraint::Length(preview_height),
    ])
    .split(results_area);

    let search_area = layout[0];
    let list_area = layout[1];
    let preview_area = layout[2];

    let visible_count = list_area.height.saturating_sub(2) as usize;
    state.set_visible_count(visible_count);

    render_search(state, frame, search_area);
    render_list(state, filtered_count, total_count, frame, list_area);
    render_preview(selected_snippet.as_ref(), inner_width, frame, preview_area);

    (Some(list_area), Some(preview_area))
}

fn calculate_preview_height(
    snippet: Option<&super::snippet_state::Snippet>,
    max_width: usize,
) -> usize {
    match snippet {
        Some(s) => wrap_text(&s.query, max_width).len(),
        None => 1,
    }
}

fn render_minimal(
    state: &mut SnippetState,
    filtered_count: usize,
    total_count: usize,
    frame: &mut Frame,
    area: Rect,
) -> (Option<Rect>, Option<Rect>) {
    if area.height < SEARCH_HEIGHT + MIN_LIST_HEIGHT {
        let content = build_list_content_from_visible(state, area.width, state.get_hovered());
        let title = build_list_title(filtered_count, total_count);

        let hints = build_browse_hints();

        let popup = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title)
                .title_bottom(hints.alignment(ratatui::layout::Alignment::Center))
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(popup, area);
        return (Some(area), None);
    }

    let layout = Layout::vertical([
        Constraint::Length(SEARCH_HEIGHT),
        Constraint::Min(MIN_LIST_HEIGHT),
    ])
    .split(area);

    let search_area = layout[0];
    let list_area = layout[1];

    let visible_count = list_area.height.saturating_sub(2) as usize;
    state.set_visible_count(visible_count);

    render_search(state, frame, search_area);
    render_list(state, filtered_count, total_count, frame, list_area);

    (Some(list_area), None)
}

fn render_search(state: &mut SnippetState, frame: &mut Frame, area: Rect) {
    let search_textarea = state.search_textarea_mut();
    search_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Search ")
            .border_style(Style::default().fg(theme::snippets::BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    search_textarea.set_style(
        Style::default()
            .fg(theme::snippets::SEARCH_TEXT)
            .bg(theme::snippets::SEARCH_BG),
    );
    frame.render_widget(&*search_textarea, area);
}

fn render_list(
    state: &SnippetState,
    filtered_count: usize,
    total_count: usize,
    frame: &mut Frame,
    area: Rect,
) {
    let content = build_list_content_from_visible(state, area.width, state.get_hovered());
    let title = build_list_title(filtered_count, total_count);

    let hints = build_browse_hints();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_bottom(hints.alignment(ratatui::layout::Alignment::Center))
        .border_style(Style::default().fg(theme::snippets::BORDER))
        .style(Style::default().bg(theme::snippets::BACKGROUND));

    let list = Paragraph::new(content).block(block);
    frame.render_widget(list, area);

    // Render scrollbar on border (excluding corners), matching border color
    let scrollbar_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    // Use scrollbar_area height as both track and viewport for correct ratio
    let track_height = scrollbar_area.height as usize;
    let max_scroll = filtered_count.saturating_sub(track_height);
    let clamped_offset = state.scroll_offset().min(max_scroll);
    scrollbar::render_vertical_scrollbar_styled(
        frame,
        scrollbar_area,
        filtered_count,
        track_height,
        clamped_offset,
        theme::snippets::SCROLLBAR,
    );
}

fn render_preview(
    selected_snippet: Option<&super::snippet_state::Snippet>,
    inner_width: usize,
    frame: &mut Frame,
    area: Rect,
) {
    let content = match selected_snippet {
        Some(snippet) => build_preview_content(snippet, inner_width),
        None => vec![Line::from(Span::styled(
            " No snippet selected",
            Style::default().fg(theme::snippets::DESCRIPTION),
        ))],
    };

    let preview = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Snippet Preview ")
            .border_style(Style::default().fg(theme::snippets::BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );

    frame.render_widget(preview, area);
}

fn build_list_content_from_visible(
    state: &SnippetState,
    area_width: u16,
    hovered_index: Option<usize>,
) -> Vec<Line<'static>> {
    if state.filtered_count() == 0 {
        let message = if state.snippets().is_empty() {
            "   No snippets yet. Press Ctrl+N to create one."
        } else {
            "   No matches"
        };
        vec![Line::from(vec![Span::styled(
            message,
            Style::default().fg(theme::snippets::DESCRIPTION),
        )])]
    } else {
        let selected_index = state.selected_index();
        let max_width = area_width.saturating_sub(4) as usize;

        state
            .visible_snippets()
            .map(|(i, s)| {
                let is_selected = i == selected_index;
                let is_hovered = hovered_index == Some(i) && !is_selected;

                let (prefix, name_style, desc_style, bg_color) = if is_selected {
                    (
                        vec![Span::styled(
                            " ▌ ",
                            Style::default()
                                .fg(theme::snippets::ITEM_SELECTED_INDICATOR)
                                .bg(theme::snippets::ITEM_SELECTED_BG),
                        )],
                        Style::default()
                            .fg(theme::snippets::FIELD_TEXT)
                            .bg(theme::snippets::ITEM_SELECTED_BG)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(theme::snippets::DESCRIPTION)
                            .bg(theme::snippets::ITEM_SELECTED_BG),
                        Some(theme::snippets::ITEM_SELECTED_BG),
                    )
                } else if is_hovered {
                    (
                        vec![Span::styled(
                            "   ",
                            Style::default().bg(theme::snippets::ITEM_HOVERED_BG),
                        )],
                        Style::default()
                            .fg(theme::snippets::FIELD_TEXT)
                            .bg(theme::snippets::ITEM_HOVERED_BG),
                        Style::default()
                            .fg(theme::snippets::DESCRIPTION)
                            .bg(theme::snippets::ITEM_HOVERED_BG),
                        Some(theme::snippets::ITEM_HOVERED_BG),
                    )
                } else {
                    (
                        vec![Span::styled(
                            "   ",
                            Style::default().bg(theme::snippets::ITEM_NORMAL_BG),
                        )],
                        Style::default().fg(theme::snippets::FIELD_TEXT),
                        Style::default().fg(theme::snippets::DESCRIPTION),
                        None,
                    )
                };

                let mut spans = prefix;
                spans.push(Span::styled(s.name.clone(), name_style));

                if let Some(desc) = &s.description {
                    let name_len = 3 + s.name.len(); // 3 = width of prefix " ▌ " or "   "
                    let separator = " - ";
                    let available = max_width.saturating_sub(name_len + separator.len());

                    if available > 10 {
                        let truncated_desc = if desc.len() > available {
                            format!("{}…", &desc[..available.saturating_sub(1)])
                        } else {
                            desc.clone()
                        };
                        spans.push(Span::styled(
                            format!("{}{}", separator, truncated_desc),
                            desc_style,
                        ));
                    }
                }

                if let Some(bg) = bg_color {
                    let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
                    let padding_len = max_width.saturating_sub(current_len);
                    if padding_len > 0 {
                        spans.push(Span::styled(
                            " ".repeat(padding_len),
                            Style::default().bg(bg),
                        ));
                    }
                }

                Line::from(spans)
            })
            .collect()
    }
}

fn build_list_title(filtered_count: usize, total_count: usize) -> String {
    if total_count == 0 {
        " Snippets ".to_string()
    } else if filtered_count == total_count {
        format!(" Snippets ({}) ", total_count)
    } else {
        format!(" Snippets ({}/{}) ", filtered_count, total_count)
    }
}

fn build_preview_content(
    snippet: &super::snippet_state::Snippet,
    max_width: usize,
) -> Vec<Line<'static>> {
    let wrapped_query = wrap_text(&snippet.query, max_width);
    wrapped_query
        .into_iter()
        .map(|line| {
            let mut spans = vec![Span::raw(" ")];
            spans.extend(JqHighlighter::highlight(&line));
            Line::from(spans)
        })
        .collect()
}

fn render_create_mode(state: &mut SnippetState, frame: &mut Frame, area: Rect) {
    let mode = state.mode().clone();

    let min_required =
        NAME_INPUT_HEIGHT + QUERY_INPUT_HEIGHT + DESCRIPTION_INPUT_HEIGHT + HINTS_HEIGHT;
    if area.height < min_required {
        render_create_minimal(state, &mode, frame, area);
        return;
    }

    let layout = Layout::vertical([
        Constraint::Length(NAME_INPUT_HEIGHT),
        Constraint::Length(QUERY_INPUT_HEIGHT),
        Constraint::Length(DESCRIPTION_INPUT_HEIGHT),
        Constraint::Min(1),
        Constraint::Length(HINTS_HEIGHT),
    ])
    .split(area);

    let name_area = layout[0];
    let query_area = layout[1];
    let description_area = layout[2];
    let hints_area = layout[4];

    let is_name_active = mode == SnippetMode::CreateName;
    let is_query_active = mode == SnippetMode::CreateQuery;
    let is_desc_active = mode == SnippetMode::CreateDescription;

    render_create_name_input(state, is_name_active, frame, name_area);
    render_create_query_input(state, is_query_active, frame, query_area);
    render_create_description_input(state, is_desc_active, frame, description_area);
    render_create_hints(&mode, frame, hints_area);
}

fn render_create_minimal(
    state: &mut SnippetState,
    mode: &SnippetMode,
    frame: &mut Frame,
    area: Rect,
) {
    match mode {
        SnippetMode::CreateName => {
            let name_textarea = state.name_textarea_mut();
            name_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" New Snippet - Name ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            name_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*name_textarea, area);
        }
        SnippetMode::CreateQuery => {
            let query_textarea = state.query_textarea_mut();
            query_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" New Snippet - Query ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            query_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*query_textarea, area);
        }
        SnippetMode::CreateDescription => {
            let desc_textarea = state.description_textarea_mut();
            desc_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" New Snippet - Description ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            desc_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*desc_textarea, area);
        }
        _ => {}
    }
}

fn render_create_name_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let name_textarea = state.name_textarea_mut();
    name_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Name ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    name_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*name_textarea, area);
    } else {
        let content = name_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Name ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_create_query_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let query_textarea = state.query_textarea_mut();
    query_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Query ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    query_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*query_textarea, area);
    } else {
        let content = query_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Query ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_create_description_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let desc_textarea = state.description_textarea_mut();
    desc_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Description (optional) ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    desc_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*desc_textarea, area);
    } else {
        let content = desc_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Description (optional) ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_create_hints(mode: &SnippetMode, frame: &mut Frame, area: Rect) {
    let hints = match mode {
        SnippetMode::CreateName | SnippetMode::CreateQuery | SnippetMode::CreateDescription => {
            build_form_hints("Create")
        }
        _ => Line::from(vec![]),
    };

    let hints_widget = Paragraph::new(vec![hints]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::snippets::BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );

    frame.render_widget(hints_widget, area);
}

fn render_edit_mode(state: &mut SnippetState, frame: &mut Frame, area: Rect) {
    let mode = state.mode().clone();

    let min_required =
        NAME_INPUT_HEIGHT + QUERY_INPUT_HEIGHT + DESCRIPTION_INPUT_HEIGHT + HINTS_HEIGHT;
    if area.height < min_required {
        render_edit_minimal(state, &mode, frame, area);
        return;
    }

    let layout = Layout::vertical([
        Constraint::Length(NAME_INPUT_HEIGHT),
        Constraint::Length(QUERY_INPUT_HEIGHT),
        Constraint::Length(DESCRIPTION_INPUT_HEIGHT),
        Constraint::Min(1),
        Constraint::Length(HINTS_HEIGHT),
    ])
    .split(area);

    let name_area = layout[0];
    let query_area = layout[1];
    let description_area = layout[2];
    let hints_area = layout[4];

    let is_name_active = matches!(mode, SnippetMode::EditName { .. });
    let is_query_active = matches!(mode, SnippetMode::EditQuery { .. });
    let is_desc_active = matches!(mode, SnippetMode::EditDescription { .. });

    render_edit_name_input(state, is_name_active, frame, name_area);
    render_edit_query_input(state, is_query_active, frame, query_area);
    render_edit_description_input(state, is_desc_active, frame, description_area);
    render_edit_hints(&mode, frame, hints_area);
}

fn render_edit_minimal(
    state: &mut SnippetState,
    mode: &SnippetMode,
    frame: &mut Frame,
    area: Rect,
) {
    match mode {
        SnippetMode::EditName { .. } => {
            let name_textarea = state.name_textarea_mut();
            name_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Edit Snippet - Name ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            name_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*name_textarea, area);
        }
        SnippetMode::EditQuery { .. } => {
            let query_textarea = state.query_textarea_mut();
            query_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Edit Snippet - Query ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            query_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*query_textarea, area);
        }
        SnippetMode::EditDescription { .. } => {
            let desc_textarea = state.description_textarea_mut();
            desc_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Edit Snippet - Description ")
                    .border_style(Style::default().fg(theme::snippets::FIELD_ACTIVE_BORDER))
                    .style(Style::default().bg(theme::snippets::BACKGROUND)),
            );
            desc_textarea.set_style(
                Style::default()
                    .fg(theme::snippets::FIELD_TEXT)
                    .bg(theme::snippets::BACKGROUND),
            );
            frame.render_widget(&*desc_textarea, area);
        }
        _ => {}
    }
}

fn render_edit_name_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let name_textarea = state.name_textarea_mut();
    name_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Name ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    name_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*name_textarea, area);
    } else {
        let content = name_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Name ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_edit_query_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let query_textarea = state.query_textarea_mut();
    query_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Query ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    query_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*query_textarea, area);
    } else {
        let content = query_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Query ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_edit_description_input(
    state: &mut SnippetState,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let border_color = if is_active {
        theme::snippets::FIELD_ACTIVE_BORDER
    } else {
        theme::snippets::BORDER
    };
    let desc_textarea = state.description_textarea_mut();
    desc_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Description (optional) ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );
    desc_textarea.set_style(
        Style::default()
            .fg(theme::snippets::FIELD_TEXT)
            .bg(theme::snippets::BACKGROUND),
    );
    if is_active {
        frame.render_widget(&*desc_textarea, area);
    } else {
        let content = desc_textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let display = Paragraph::new(format!(" {}", content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Description (optional) ")
                .border_style(Style::default().fg(theme::snippets::BORDER))
                .style(Style::default().bg(theme::snippets::BACKGROUND)),
        );
        frame.render_widget(display, area);
    }
}

fn render_edit_hints(mode: &SnippetMode, frame: &mut Frame, area: Rect) {
    let hints = match mode {
        SnippetMode::EditName { .. }
        | SnippetMode::EditQuery { .. }
        | SnippetMode::EditDescription { .. } => build_form_hints("Update"),
        _ => Line::from(vec![]),
    };

    let hints_widget = Paragraph::new(vec![hints]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::snippets::BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );

    frame.render_widget(hints_widget, area);
}

fn render_confirm_delete_mode(state: &SnippetState, frame: &mut Frame, area: Rect) {
    let snippet_name = match state.mode() {
        SnippetMode::ConfirmDelete { snippet_name } => snippet_name.clone(),
        _ => String::new(),
    };

    let dialog_height: u16 = 7;
    let dialog_width = (area.width.saturating_sub(4)).min(50);
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    let truncated_name = if snippet_name.len() > 30 {
        format!("{}…", &snippet_name[..29])
    } else {
        snippet_name
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" Delete \"{}\"?", truncated_name),
            Style::default().fg(theme::snippets::FIELD_TEXT),
        )),
        Line::from(""),
        build_confirm_hints(),
    ];

    let dialog = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(theme::snippets::DELETE_BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );

    popup::clear_area(frame, dialog_area);
    frame.render_widget(dialog, dialog_area);
}

fn render_confirm_update_mode(state: &SnippetState, frame: &mut Frame, area: Rect) {
    let (snippet_name, old_query, new_query) = match state.mode() {
        SnippetMode::ConfirmUpdate {
            snippet_name,
            old_query,
            new_query,
        } => (snippet_name.clone(), old_query.clone(), new_query.clone()),
        _ => (String::new(), String::new(), String::new()),
    };

    let inner_width = area.width.saturating_sub(6) as usize;
    let old_wrapped = wrap_text(&old_query, inner_width);
    let new_wrapped = wrap_text(&new_query, inner_width);

    let old_lines = old_wrapped.len().max(1) as u16;
    let new_lines = new_wrapped.len().max(1) as u16;
    // Content: 4 lines before old query + old_lines + 2 lines between + new_lines + 2 lines after
    let content_height = 4 + old_lines + 2 + new_lines + 2;
    let dialog_height = (content_height + 2).min(area.height.saturating_sub(2));
    let dialog_width = (area.width.saturating_sub(4)).min(70);
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    let truncated_name = if snippet_name.len() > 40 {
        format!("{}…", &snippet_name[..39])
    } else {
        snippet_name
    };

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" Replace query for \"{}\"?", truncated_name),
            Style::default().fg(theme::snippets::FIELD_TEXT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Old query:",
            Style::default()
                .fg(theme::snippets::FIELD_ACTIVE_BORDER)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    for line in old_wrapped {
        let mut spans = vec![Span::raw("   ")];
        spans.extend(JqHighlighter::highlight(&line));
        content.push(Line::from(spans));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        " New query:",
        Style::default()
            .fg(theme::palette::SUCCESS)
            .add_modifier(Modifier::BOLD),
    )));

    for line in new_wrapped {
        let mut spans = vec![Span::raw("   ")];
        spans.extend(JqHighlighter::highlight(&line));
        content.push(Line::from(spans));
    }

    content.push(Line::from(""));
    content.push(build_confirm_hints());

    let dialog = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Replace Snippet Query ")
            .border_style(Style::default().fg(theme::snippets::BORDER))
            .style(Style::default().bg(theme::snippets::BACKGROUND)),
    );

    popup::clear_area(frame, dialog_area);
    frame.render_widget(dialog, dialog_area);
}

#[cfg(test)]
#[path = "snippet_render_tests.rs"]
mod snippet_render_tests;
