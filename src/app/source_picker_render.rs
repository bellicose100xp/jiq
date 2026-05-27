//! Source-picker view.
//!
//! Two regions stacked vertically:
//!
//! 1. **Option banner** at the top — fixed-height box listing Clipboard
//!    and Paste. The bottom border carries the keyboard-hint strip,
//!    whose Enter label adapts to the highlighted option (`Load` for
//!    Clipboard, `Open paste editor` for Paste).
//! 2. **Preview** below — fills the rest of the viewport. When
//!    Clipboard is highlighted the cached payload's first lines render
//!    inside a bordered box. When Paste is highlighted the area is
//!    deliberately left empty: the user already knows what Enter will
//!    do from the hint strip, so a "Paste" placeholder box would just
//!    add visual noise.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};

use crate::input::loader::ClipboardPeek;
use crate::input::{SourceChoice, SourcePickerState};
use crate::theme;

/// Hard cap on bytes scanned when building the preview head. The
/// cached payload may be megabytes; scanning ~4 KB is enough to show
/// the JSON shape without copying the whole buffer for every render.
const PREVIEW_MAX_BYTES: usize = 4 * 1024;

/// Render the picker view (option banner + optional preview pane) and
/// return the area it consumed.
pub fn render(state: &SourcePickerState, frame: &mut Frame, area: Rect) -> Rect {
    if area.height < 6 || area.width < 40 {
        render_too_small(frame, area);
        return area;
    }

    // Option banner is fixed-height: 2 borders + 2 option rows = 4
    // rows. The hint strip rides the banner's bottom border so it stays
    // visible regardless of which option is highlighted.
    const BANNER_HEIGHT: u16 = 4;
    let chunks =
        Layout::vertical([Constraint::Length(BANNER_HEIGHT), Constraint::Min(0)]).split(area);
    let banner_area = chunks[0];
    let preview_area = chunks[1];

    render_banner(state, frame, banner_area);
    if matches!(state.selection, SourceChoice::Clipboard) {
        render_preview(state, frame, preview_area);
    }

    area
}

fn render_banner(state: &SourcePickerState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Choose JSON input source ")
        .border_style(Style::default().fg(theme::input::MODE_INSERT))
        .padding(Padding::horizontal(1))
        .title_bottom(bottom_hints(state.selection).alignment(Alignment::Center));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line<'static>> = vec![
        option_line(state, SourceChoice::Clipboard),
        option_line(state, SourceChoice::Paste),
    ];
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn render_preview(state: &SourcePickerState, frame: &mut Frame, area: Rect) {
    // Only Usable peeks reach this branch — main.rs gates the picker
    // off entirely when the clipboard isn't usable, so the preview
    // pane never has to handle "no JSON" itself.
    let bytes = match &state.peek {
        ClipboardPeek::Usable(bytes) => bytes.as_str(),
        _ => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Clipboard preview ")
        .border_style(Style::default().fg(theme::input::BORDER_UNFOCUSED))
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Use whatever vertical room the preview pane has; the cached
    // payload may be many lines, but we never need more than the inner
    // height to fill the screen.
    let max_lines = inner.height.saturating_sub(1) as usize; // reserve 1 row for tail
    let lines = clipboard_preview_lines(bytes, max_lines.max(1));
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

/// Build the lines shown in the preview pane. Splits on newlines,
/// keeps at most `max_lines` of head content, and appends a "… (N
/// more lines, M.M KB more)" tail when content is elided.
fn clipboard_preview_lines(bytes: &str, max_lines: usize) -> Vec<Line<'static>> {
    let total_lines = bytes.lines().count();
    let total_bytes = bytes.len();

    // Cap the byte slice we scan so render time stays O(viewport)
    // even on a 50 MB clipboard.
    let head = if bytes.len() <= PREVIEW_MAX_BYTES {
        bytes
    } else {
        let mut end = PREVIEW_MAX_BYTES;
        while end > 0 && !bytes.is_char_boundary(end) {
            end -= 1;
        }
        &bytes[..end]
    };

    let visible: Vec<&str> = head.lines().take(max_lines).collect();
    let visible_byte_count: usize = visible.iter().map(|l| l.len() + 1).sum();
    let truncated = visible.len() < total_lines || visible_byte_count < total_bytes;

    let mut out: Vec<Line<'static>> = visible
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme::palette::TEXT),
            ))
        })
        .collect();

    if truncated {
        let elided_lines = total_lines.saturating_sub(visible.len());
        let elided_bytes = total_bytes.saturating_sub(visible_byte_count);
        let tail = match (elided_lines, elided_bytes) {
            (0, b) if b > 0 => format!("… ({} more)", fmt_bytes(b)),
            (l, 0) => format!("… ({l} more line{})", if l == 1 { "" } else { "s" }),
            (l, b) => format!(
                "… ({} more line{}, {} more)",
                l,
                if l == 1 { "" } else { "s" },
                fmt_bytes(b)
            ),
        };
        out.push(Line::from(Span::styled(
            tail,
            Style::default()
                .fg(theme::palette::TEXT_MUTED)
                .add_modifier(Modifier::ITALIC),
        )));
    }
    out
}

/// Smaller-than-min terminal: replace the picker with a single-line
/// hint pointing at the explicit flags so the user has an escape hatch
/// without resizing.
fn render_too_small(frame: &mut Frame, area: Rect) {
    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            "Terminal too small for the source picker.",
            Style::default()
                .fg(theme::input::BORDER_ERROR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Resize, or relaunch with --clipboard / --paste.",
            Style::default().fg(theme::palette::TEXT),
        )),
    ])
    .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn option_line(state: &SourcePickerState, choice: SourceChoice) -> Line<'static> {
    let selected = state.selection == choice;
    let marker = if selected { "▶" } else { " " };
    let label = match choice {
        SourceChoice::Clipboard => "Clipboard",
        SourceChoice::Paste => "Paste",
    };
    let style = if selected {
        Style::default()
            .fg(theme::input::MODE_INSERT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::palette::TEXT)
    };
    Line::from(vec![
        Span::styled(format!(" {} ", marker), style),
        Span::styled(label.to_string(), style),
    ])
}

/// Build the keyboard-hint strip rendered on the option banner's
/// bottom border. The Enter label is contextual: it tells the user
/// what's about to happen if they confirm the current selection.
fn bottom_hints(selection: SourceChoice) -> Line<'static> {
    let enter_label = match selection {
        SourceChoice::Clipboard => "Load",
        SourceChoice::Paste => "Open paste editor",
    };
    let entries: [(&'static str, &'static str); 3] =
        [("Enter", enter_label), ("↑/↓", "Switch"), ("Esc", "Quit")];
    theme::border_hints::build_hints(&entries, theme::input::MODE_INSERT)
}

/// Format a byte count using SI-style suffixes (B / KB / MB).
fn fmt_bytes(n: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * 1024;
    if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{} B", n)
    }
}
