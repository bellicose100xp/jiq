use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::app::App;
use crate::help::{HELP_FOOTER, HelpSection, HelpTab, get_tab_content};
use crate::widgets::{popup, scrollbar};

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
        .title(" Keyboard Shortcuts ")
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    // Split inner area: tab bar, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Render tab bar
    let tabs = render_tab_bar(app.help.active_tab);
    frame.render_widget(tabs, chunks[0]);

    // Render separator line
    let separator = Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(separator), chunks[1]);

    // Render content for active tab
    let content = get_tab_content(app.help.active_tab);
    let lines = render_help_sections(content.sections);

    // Update scroll bounds for current tab
    let content_height = lines.len() as u32;
    let visible_height = chunks[2].height;
    app.help
        .current_scroll_mut()
        .update_bounds(content_height, visible_height);

    let paragraph = Paragraph::new(Text::from(lines)).scroll((app.help.current_scroll().offset, 0));
    frame.render_widget(paragraph, chunks[2]);

    // Render footer
    let footer = Line::from(Span::styled(
        HELP_FOOTER,
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(footer).centered(), chunks[3]);

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
        Color::Cyan,
    );

    Some(popup_area)
}

fn render_tab_bar(active_tab: HelpTab) -> Tabs<'static> {
    let titles: Vec<Line> = HelpTab::all()
        .iter()
        .map(|tab| {
            let number = tab.index() + 1;
            let label = format!("{}:{}", number, tab.name());
            if *tab == active_tab {
                Line::styled(
                    format!("[{}]", label),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::styled(label, Style::default().fg(Color::DarkGray))
            }
        })
        .collect();

    Tabs::new(titles)
        .divider(Span::raw(" "))
        .highlight_style(Style::default())
}

fn render_help_sections(sections: &[HelpSection]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (section_idx, section) in sections.iter().enumerate() {
        // Add section header if present
        if let Some(title) = section.title {
            if section_idx > 0 {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("── {} ──", title),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Add entries
        for (key, desc) in section.entries {
            let key_span = Span::styled(
                format!("  {:<15}", key),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            let desc_span = Span::styled(*desc, Style::default().fg(Color::White));
            lines.push(Line::from(vec![key_span, desc_span]));
        }
    }

    lines
}

#[cfg(test)]
#[path = "help_popup_render_tests.rs"]
mod help_popup_render_tests;
