use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::snippet_state::{SnippetMode, SnippetState};
use crate::ai::render::text::wrap_text;
use crate::widgets::popup;

const MIN_LIST_HEIGHT: u16 = 3;
const SEARCH_HEIGHT: u16 = 3;
const NAME_INPUT_HEIGHT: u16 = 3;
const DESCRIPTION_INPUT_HEIGHT: u16 = 3;
const QUERY_INPUT_HEIGHT: u16 = 3;
const HINTS_HEIGHT: u16 = 3;
const BROWSE_HINTS_HEIGHT: u16 = 1;

pub fn render_popup(state: &mut SnippetState, frame: &mut Frame, results_area: Rect) {
    popup::clear_area(frame, results_area);

    match state.mode() {
        SnippetMode::Browse => render_browse_mode(state, frame, results_area),
        SnippetMode::CreateName | SnippetMode::CreateQuery | SnippetMode::CreateDescription => {
            render_create_mode(state, frame, results_area)
        }
        SnippetMode::EditName { .. }
        | SnippetMode::EditQuery { .. }
        | SnippetMode::EditDescription { .. } => render_edit_mode(state, frame, results_area),
        SnippetMode::ConfirmDelete { .. } => render_confirm_delete_mode(state, frame, results_area),
    }
}

fn render_browse_mode(state: &mut SnippetState, frame: &mut Frame, results_area: Rect) {
    let selected_snippet = state.selected_snippet().cloned();
    let total_count = state.snippets().len();
    let filtered_count = state.filtered_count();

    let inner_width = results_area.width.saturating_sub(4) as usize;
    let preview_content_height = calculate_preview_height(selected_snippet.as_ref(), inner_width);
    let preview_height = (preview_content_height as u16 + 2).min(results_area.height / 2);

    let min_required = SEARCH_HEIGHT + MIN_LIST_HEIGHT + preview_height + BROWSE_HINTS_HEIGHT;
    if results_area.height < min_required {
        let visible_count = results_area.height.saturating_sub(SEARCH_HEIGHT + 2) as usize;
        state.set_visible_count(visible_count.max(1));
        render_minimal(state, filtered_count, total_count, frame, results_area);
        return;
    }

    let layout = Layout::vertical([
        Constraint::Length(SEARCH_HEIGHT),
        Constraint::Min(MIN_LIST_HEIGHT),
        Constraint::Length(preview_height),
        Constraint::Length(BROWSE_HINTS_HEIGHT),
    ])
    .split(results_area);

    let search_area = layout[0];
    let list_area = layout[1];
    let preview_area = layout[2];
    let hints_area = layout[3];

    let visible_count = list_area.height.saturating_sub(2) as usize;
    state.set_visible_count(visible_count);

    render_search(state, frame, search_area);
    render_list(state, filtered_count, total_count, frame, list_area);
    render_preview(selected_snippet.as_ref(), inner_width, frame, preview_area);
    render_browse_hints(frame, hints_area);
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
) {
    if area.height < SEARCH_HEIGHT + MIN_LIST_HEIGHT {
        let content = build_list_content_from_visible(state, area.width);
        let title = build_list_title(filtered_count, total_count);
        let popup = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(popup, area);
        return;
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
}

fn render_search(state: &mut SnippetState, frame: &mut Frame, area: Rect) {
    let search_textarea = state.search_textarea_mut();
    search_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Search ")
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );
    search_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
    frame.render_widget(&*search_textarea, area);
}

fn render_list(
    state: &SnippetState,
    filtered_count: usize,
    total_count: usize,
    frame: &mut Frame,
    area: Rect,
) {
    let content = build_list_content_from_visible(state, area.width);
    let title = build_list_title(filtered_count, total_count);

    let list = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(list, area);
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
            Style::default().fg(Color::DarkGray),
        ))],
    };

    let preview = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Snippet Preview ")
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(preview, area);
}

fn render_browse_hints(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled(" [↑/↓]", Style::default().fg(Color::Yellow)),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Apply  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Ctrl+N]", Style::default().fg(Color::Yellow)),
        Span::styled(" New  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Ctrl+E]", Style::default().fg(Color::Yellow)),
        Span::styled(" Edit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Ctrl+D]", Style::default().fg(Color::Yellow)),
        Span::styled(" Delete  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Close", Style::default().fg(Color::DarkGray)),
    ]);

    let hints_widget =
        Paragraph::new(hints).style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    frame.render_widget(hints_widget, area);
}

fn build_list_content_from_visible(state: &SnippetState, area_width: u16) -> Vec<Line<'static>> {
    if state.filtered_count() == 0 {
        let message = if state.snippets().is_empty() {
            "   No snippets yet. Press Ctrl+N to create one."
        } else {
            "   No matches"
        };
        vec![Line::from(vec![Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        let selected_index = state.selected_index();
        let max_width = area_width.saturating_sub(4) as usize;

        state
            .visible_snippets()
            .map(|(i, s)| {
                let is_selected = i == selected_index;
                let prefix = if is_selected { " ► " } else { "   " };

                let (name_style, desc_style) = if is_selected {
                    (
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    )
                } else {
                    (
                        Style::default().fg(Color::White),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                let mut spans = vec![Span::styled(format!("{}{}", prefix, s.name), name_style)];

                if let Some(desc) = &s.description {
                    let name_len = prefix.len() + s.name.len();
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

                if is_selected {
                    let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
                    let padding_len = max_width.saturating_sub(current_len);
                    if padding_len > 0 {
                        spans.push(Span::styled(
                            " ".repeat(padding_len),
                            Style::default().bg(Color::Cyan),
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
            Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(Color::White),
            ))
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
                    .title(" New Snippet - Name ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            name_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
            frame.render_widget(&*name_textarea, area);
        }
        SnippetMode::CreateQuery => {
            let query_textarea = state.query_textarea_mut();
            query_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" New Snippet - Query ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            query_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
            frame.render_widget(&*query_textarea, area);
        }
        SnippetMode::CreateDescription => {
            let desc_textarea = state.description_textarea_mut();
            desc_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" New Snippet - Description ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            desc_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let name_textarea = state.name_textarea_mut();
    name_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Name ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    name_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Name ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let query_textarea = state.query_textarea_mut();
    query_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Query ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    query_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Query ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let desc_textarea = state.description_textarea_mut();
    desc_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Description (optional) ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    desc_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Description (optional) ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(display, area);
    }
}

fn render_create_hints(mode: &SnippetMode, frame: &mut Frame, area: Rect) {
    let hints = match mode {
        SnippetMode::CreateName | SnippetMode::CreateQuery | SnippetMode::CreateDescription => {
            Line::from(vec![
                Span::styled(" [Enter]", Style::default().fg(Color::Yellow)),
                Span::styled(" Create  ", Style::default().fg(Color::White)),
                Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
                Span::styled(" Next  ", Style::default().fg(Color::White)),
                Span::styled("[Shift+Tab]", Style::default().fg(Color::Yellow)),
                Span::styled(" Prev  ", Style::default().fg(Color::White)),
                Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
                Span::styled(" Cancel", Style::default().fg(Color::White)),
            ])
        }
        _ => Line::from(vec![]),
    };

    let hints_widget = Paragraph::new(vec![hints]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
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
                    .title(" Edit Snippet - Name ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            name_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
            frame.render_widget(&*name_textarea, area);
        }
        SnippetMode::EditQuery { .. } => {
            let query_textarea = state.query_textarea_mut();
            query_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Edit Snippet - Query ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            query_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
            frame.render_widget(&*query_textarea, area);
        }
        SnippetMode::EditDescription { .. } => {
            let desc_textarea = state.description_textarea_mut();
            desc_textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Edit Snippet - Description ")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );
            desc_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let name_textarea = state.name_textarea_mut();
    name_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Name ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    name_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Name ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let query_textarea = state.query_textarea_mut();
    query_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Query ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    query_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Query ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
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
        Color::Yellow
    } else {
        Color::Cyan
    };
    let desc_textarea = state.description_textarea_mut();
    desc_textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Description (optional) ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black)),
    );
    desc_textarea.set_style(Style::default().fg(Color::White).bg(Color::Black));
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
                .title(" Description (optional) ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(display, area);
    }
}

fn render_edit_hints(mode: &SnippetMode, frame: &mut Frame, area: Rect) {
    let hints = match mode {
        SnippetMode::EditName { .. }
        | SnippetMode::EditQuery { .. }
        | SnippetMode::EditDescription { .. } => Line::from(vec![
            Span::styled(" [Enter]", Style::default().fg(Color::Yellow)),
            Span::styled(" Update  ", Style::default().fg(Color::White)),
            Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
            Span::styled(" Next  ", Style::default().fg(Color::White)),
            Span::styled("[Shift+Tab]", Style::default().fg(Color::Yellow)),
            Span::styled(" Prev  ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]),
        _ => Line::from(vec![]),
    };

    let hints_widget = Paragraph::new(vec![hints]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
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
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Enter]", Style::default().fg(Color::Yellow)),
            Span::styled(" Confirm    ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let dialog = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Delete ")
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().bg(Color::Black)),
    );

    popup::clear_area(frame, dialog_area);
    frame.render_widget(dialog, dialog_area);
}

#[cfg(test)]
#[path = "snippet_render_tests.rs"]
mod snippet_render_tests;
