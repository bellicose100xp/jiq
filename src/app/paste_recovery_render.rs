use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
use tui_textarea::TextArea;

use crate::editor::EditorMode;
use crate::input::PasteRecoveryState;
use crate::input::paste_recovery::PasteRecoveryMode;
use crate::theme;

/// Render the paste-recovery view as a full replacement for the normal
/// app layout. The textarea is the live `app.input.textarea`, so all of
/// jiq's existing VIM bindings (operators, char-search, text objects,
/// etc.) apply during recovery for free.
///
/// Returns the `Rect` covering the whole area so callers can suppress
/// the standard layout above it.
pub fn render(
    state: &PasteRecoveryState,
    textarea: &mut TextArea<'static>,
    editor_mode: EditorMode,
    frame: &mut Frame,
    area: Rect,
) -> Rect {
    let layout = Layout::vertical([Constraint::Length(7), Constraint::Min(3)]).split(area);

    render_error_block(state, frame, layout[0]);
    render_textarea(textarea, editor_mode, frame, layout[1]);

    area
}

fn render_error_block(state: &PasteRecoveryState, frame: &mut Frame, area: Rect) {
    // Recovery mode (clipboard failure): red border, "No JSON loaded"
    // title, message styled as an error.
    // Explicit mode (--paste / picker→Paste): cyan border, neutral
    // "Paste JSON" title, message styled as plain instructions.
    let (title, border_color, message_color, show_secondary_hint) = match state.mode {
        PasteRecoveryMode::Recovery => (
            " No JSON loaded ",
            theme::input::BORDER_ERROR,
            theme::input::BORDER_ERROR,
            true,
        ),
        PasteRecoveryMode::Explicit => (
            " Paste JSON ",
            theme::input::MODE_INSERT,
            theme::palette::TEXT,
            false,
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::horizontal(1));

    let mut lines: Vec<Line<'_>> = vec![Line::from(Span::styled(
        state.error_message.clone(),
        Style::default()
            .fg(message_color)
            .add_modifier(Modifier::BOLD),
    ))];
    // Recovery mode echoes the failure diagnosis on the first line and
    // needs a secondary hint telling the user what to do next. Explicit
    // mode's first-line message ("Paste JSON below and press Enter to
    // load.") is already that hint; a second copy is just clutter.
    if show_secondary_hint {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "Paste JSON below and press Enter to load.",
            Style::default().fg(theme::palette::TEXT),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_textarea(
    textarea: &mut TextArea<'static>,
    editor_mode: EditorMode,
    frame: &mut Frame,
    area: Rect,
) {
    let mode_color = mode_color(editor_mode);
    let mode_text = editor_mode.display();

    let title_spans = match editor_mode {
        EditorMode::Normal => vec![
            Span::raw(" Paste JSON ["),
            Span::styled(mode_text, Style::default().fg(mode_color)),
            Span::raw("] (press 'i' to edit) "),
        ],
        _ => vec![
            Span::raw(" Paste JSON ["),
            Span::styled(mode_text, Style::default().fg(mode_color)),
            Span::raw("] "),
        ],
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Line::from(title_spans))
        .border_style(Style::default().fg(mode_color))
        .title_bottom(bottom_hints(editor_mode, mode_color).alignment(Alignment::Center));

    textarea.set_block(block);
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(cursor_style(editor_mode));
    textarea.set_placeholder_text("Paste JSON, then press Enter to load");

    frame.render_widget(&*textarea, area);
}

fn mode_color(mode: EditorMode) -> Color {
    match mode {
        EditorMode::Insert => theme::input::MODE_INSERT,
        EditorMode::Normal => theme::input::MODE_NORMAL,
        EditorMode::Operator(_) => theme::input::MODE_OPERATOR,
        EditorMode::CharSearch(_, _) => theme::input::MODE_CHAR_SEARCH,
        EditorMode::OperatorCharSearch(_, _, _, _) => theme::input::MODE_OPERATOR,
        EditorMode::TextObject(_, _) => theme::input::MODE_OPERATOR,
    }
}

fn cursor_style(mode: EditorMode) -> Style {
    match mode {
        EditorMode::Insert => Style::default().add_modifier(Modifier::REVERSED),
        _ => Style::default()
            .fg(mode_color(mode))
            .add_modifier(Modifier::REVERSED),
    }
}

/// Bottom-border hints. Mode-specific so we only surface the *opposite*
/// mode toggle (i.e. "Esc Normal" while in Insert, "i Insert" while in
/// Normal) — a user already in Normal mode doesn't need to be told how
/// to enter Normal mode. While inside an in-progress operator (`d…`,
/// `c…`, etc.) the toggle hint is dropped entirely; the user is mid
/// command and the existing handler will return them to Normal on its
/// own.
fn bottom_hints(mode: EditorMode, color: Color) -> Line<'static> {
    let toggle_hint: Option<(&'static str, &'static str)> = match mode {
        EditorMode::Insert => Some(("Esc", "Normal")),
        EditorMode::Normal => Some(("i", "Insert")),
        _ => None,
    };

    let mut entries: Vec<(&'static str, &'static str)> = vec![("Enter", "Load JSON")];
    if let Some(hint) = toggle_hint {
        entries.push(hint);
    }
    entries.push(("Ctrl+X", "Clear"));
    entries.push(("Ctrl+C", "Quit"));

    theme::border_hints::build_hints(&entries, color)
}
