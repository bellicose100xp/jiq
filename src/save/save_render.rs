use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use super::save_state::{PathPreview, SaveMode, SaveState};
use crate::theme;

const POPUP_WIDTH: u16 = 64;
const POPUP_HEIGHT: u16 = 10;
const INNER_PAD_X: u16 = 2;
const INNER_PAD_Y: u16 = 1;

pub fn render_save_popup(frame: &mut Frame<'_>, area: Rect, state: &mut SaveState) {
    match state.mode().clone() {
        SaveMode::Closed => {}
        SaveMode::EnterFilename => render_enter_filename(frame, area, state),
    }
}

fn render_enter_filename(frame: &mut Frame<'_>, area: Rect, state: &mut SaveState) {
    let preview = state.compute_preview();
    let popup_area = centered_rect(area, POPUP_WIDTH, POPUP_HEIGHT);
    frame.render_widget(Clear, popup_area);

    let border_style = Style::default().fg(border_color(&preview));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(
            " Save Result to file ",
            Style::default()
                .fg(theme::save::title())
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(build_bottom_hints(&preview).alignment(Alignment::Center));

    let outer_inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let padded = outer_inner.inner(Margin {
        horizontal: INNER_PAD_X,
        vertical: INNER_PAD_Y,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(padded);

    let label = Paragraph::new(Line::from(Span::styled(
        "Path:",
        Style::default().fg(theme::save::hint_text()),
    )));
    frame.render_widget(label, chunks[0]);

    let textarea = state.filename_mut();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::save::input_border())),
    );
    textarea.set_style(
        Style::default()
            .fg(theme::save::input_fg())
            .bg(theme::save::input_bg()),
    );
    frame.render_widget(&*textarea, chunks[1]);

    let preview_line = build_preview_line(&preview, chunks[2].width);
    frame.render_widget(Paragraph::new(preview_line), chunks[2]);
}

fn build_preview_line(preview: &PathPreview, max_width: u16) -> Line<'static> {
    match preview {
        PathPreview::Ready {
            resolved,
            exists: false,
        } => {
            let prefix = "\u{2192} ";
            let path = truncate_front(&resolved.display().to_string(), max_width, prefix.len());
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme::save::preview_ok())),
                Span::styled(path, Style::default().fg(theme::save::preview_ok())),
            ])
        }
        PathPreview::Ready {
            resolved,
            exists: true,
        } => {
            let prefix = "\u{26A0} File exists: ";
            let path = truncate_front(&resolved.display().to_string(), max_width, prefix.len());
            Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(theme::save::preview_warn())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(path, Style::default().fg(theme::save::preview_warn())),
            ])
        }
        PathPreview::Error(msg) => Line::from(vec![
            Span::styled(
                "\u{2715} ",
                Style::default()
                    .fg(theme::save::error())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg.clone(), Style::default().fg(theme::save::error())),
        ]),
    }
}

/// Truncate a path from the front so the filename always shows.
/// Returns "…/parent/file.ext" sized to fit `max_width - prefix_width`.
fn truncate_front(path: &str, max_width: u16, prefix_chars: usize) -> String {
    let prefix_width = prefix_chars as u16;
    if max_width <= prefix_width {
        return String::new();
    }
    let budget = (max_width - prefix_width) as usize;
    let path_width = path.width();
    if path_width <= budget {
        return path.to_string();
    }
    // Walk back from the end of the path until the remaining suffix fits in
    // (budget - 1) display cells (the 1 leaves room for the leading ellipsis).
    let mut chars: Vec<char> = path.chars().collect();
    while chars.iter().collect::<String>().as_str().width() + 1 > budget && !chars.is_empty() {
        chars.remove(0);
    }
    let mut out = String::with_capacity(budget);
    out.push('\u{2026}');
    out.extend(chars);
    out
}

fn build_bottom_hints(preview: &PathPreview) -> Line<'static> {
    let action = if preview.would_overwrite() {
        "Overwrite"
    } else {
        "Save"
    };
    theme::border_hints::build_hints(&[("Enter", action), ("Esc", "Cancel")], hint_color(preview))
}

fn hint_color(preview: &PathPreview) -> ratatui::style::Color {
    match preview {
        PathPreview::Ready { exists: true, .. } => theme::save::preview_warn(),
        PathPreview::Error(_) => theme::save::error(),
        _ => theme::save::border(),
    }
}

fn border_color(_preview: &PathPreview) -> ratatui::style::Color {
    // Border stays the popup's brand color regardless of preview state.
    // Collision/error signaling lives on the preview line and the bottom-
    // border hints, where the user is already looking.
    theme::save::border()
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
#[path = "save_render_tests.rs"]
mod save_render_tests;
