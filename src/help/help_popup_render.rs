use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::help::{HelpSection, HelpTab, get_tab_content};
use crate::theme;
use crate::widgets::{popup, scrollbar};

const HORIZONTAL_PADDING: u16 = 1;
const VERTICAL_PADDING: u16 = 1;

/// Render the help popup
///
/// Returns the popup area for region tracking.
pub fn render_popup(app: &mut App, frame: &mut Frame) -> Option<Rect> {
    let frame_area = frame.area();

    if frame_area.width < 40 || frame_area.height < 15 {
        return None;
    }

    // Popup dimensions - use 80% of screen (min 70x20, max 90x30)
    let popup_width = ((frame_area.width as f32 * 0.8) as u16)
        .clamp(70, 90)
        .min(frame_area.width.saturating_sub(4));
    let popup_height = ((frame_area.height as f32 * 0.8) as u16)
        .clamp(20, 30)
        .min(frame_area.height.saturating_sub(2));

    let popup_area = popup::centered_popup(frame_area, popup_width, popup_height);
    popup::clear_area(frame, popup_area);

    // Outer block with title and border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Keyboard Shortcuts ")
        .title_bottom(
            theme::border_hints::build_hints(
                &[
                    ("1-7", "Jump"),
                    ("Tab", "Next"),
                    ("h/l", "Switch"),
                    ("j/k", "Scroll"),
                    ("q", "Close"),
                ],
                theme::help::BORDER,
            )
            .centered(),
        )
        .border_style(Style::default().fg(theme::help::BORDER))
        .style(Style::default().bg(theme::help::BACKGROUND));

    let inner_area = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    // Split inner area: tab bar, content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(1),    // Content
        ])
        .split(inner_area);

    // Render tab bar (centered)
    let tab_line = render_tab_bar(
        app.help.active_tab,
        app.help.get_hovered_tab(),
        chunks[0].width,
    );
    frame.render_widget(
        Paragraph::new(tab_line).alignment(Alignment::Center),
        chunks[0],
    );

    // Render separator line
    let separator = Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(theme::help::FOOTER),
    ));
    frame.render_widget(Paragraph::new(separator), chunks[1]);

    // Apply padding to content area
    let content_area = popup::inset_rect(chunks[2], HORIZONTAL_PADDING, VERTICAL_PADDING);

    // Render content for active tab
    let content = get_tab_content(app.help.active_tab);
    let lines = render_help_sections(content.sections, content_area.width);

    // Update scroll bounds for current tab
    let content_height = lines.len() as u32;
    let visible_height = content_area.height;
    app.help
        .current_scroll_mut()
        .update_bounds(content_height, visible_height);

    let paragraph = Paragraph::new(Text::from(lines)).scroll((app.help.current_scroll().offset, 0));
    frame.render_widget(paragraph, content_area);

    // Render scrollbar on outer border (excluding corners), matching border color
    let scrollbar_area = Rect {
        x: popup_area.x,
        y: popup_area.y.saturating_add(1),
        width: popup_area.width,
        height: popup_area.height.saturating_sub(2),
    };
    let scroll = app.help.current_scroll();
    let viewport = scroll.viewport_height as usize;
    let max_scroll = scroll.max_offset as usize;
    let clamped_offset = (scroll.offset as usize).min(max_scroll);
    scrollbar::render_vertical_scrollbar_styled(
        frame,
        scrollbar_area,
        content_height as usize,
        viewport,
        clamped_offset,
        theme::help::SCROLLBAR,
    );

    Some(popup_area)
}

/// Spacing between tabs in the tab bar (in characters)
pub const TAB_DIVIDER_WIDTH: u16 = 3;

fn render_tab_bar(active_tab: HelpTab, hovered_tab: Option<HelpTab>, _width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let divider = " ".repeat(TAB_DIVIDER_WIDTH as usize);

    for (i, tab) in HelpTab::all().iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(divider.clone()));
        }

        let number = tab.index() + 1;
        let label = format!("{}:{}", number, tab.name());
        let is_hovered = hovered_tab == Some(*tab) && *tab != active_tab;

        if *tab == active_tab {
            spans.push(Span::styled(
                format!("[{}]", label),
                theme::help::TAB_ACTIVE,
            ));
        } else if is_hovered {
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(theme::help::TAB_HOVER_FG)
                    .bg(theme::help::TAB_HOVER_BG)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(label, theme::help::TAB_INACTIVE));
        }
    }

    Line::from(spans)
}

fn render_help_sections(sections: &[HelpSection], width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Calculate centering: key(15) + spacing(2) + desc(~40) = ~57 chars typical
    // We want to center this content in the available width
    let content_width = 57u16;
    let left_padding = if width > content_width {
        (width.saturating_sub(content_width)) / 2
    } else {
        0
    };
    let padding = " ".repeat(left_padding as usize);

    for (section_idx, section) in sections.iter().enumerate() {
        // Add section header if present
        if let Some(title) = section.title {
            if section_idx > 0 {
                lines.push(Line::from(""));
            }
            // Left-align section header with same padding as content
            let header_text = format!("{}── {} ──", padding, title);
            lines.push(Line::from(Span::styled(
                header_text,
                theme::help::SECTION_HEADER,
            )));
        }

        // Add entries
        for (key, desc) in section.entries {
            let key_span = Span::styled(format!("{}{:<15}", padding, key), theme::help::KEY);
            let desc_span = Span::styled(*desc, Style::default().fg(theme::help::DESCRIPTION));
            lines.push(Line::from(vec![key_span, desc_span]));
        }
    }

    lines
}

#[cfg(test)]
#[path = "help_popup_render_tests.rs"]
mod help_popup_render_tests;
