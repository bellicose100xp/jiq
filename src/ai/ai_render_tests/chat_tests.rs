//! Chat input, transcript, and popup geometry tests for the AI popup

use super::*;
use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::ai_state::{Suggestion, SuggestionType};
use crate::ai::chat::ChatExchange;
use crate::scroll::Scrollable;
use crate::theme;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

const PLACEHOLDER: &str = "Ask about this query or data…";

fn configured_state() -> AiState {
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state
}

fn suggestion(kind: SuggestionType, query: &str, description: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: description.to_string(),
        suggestion_type: kind,
    }
}

fn exchange(question: &str, answer: Option<&str>, queries: &[&str]) -> ChatExchange {
    ChatExchange {
        question: question.to_string(),
        query: ".".to_string(),
        raw_response: String::new(),
        answer: answer.map(str::to_string),
        suggestions: queries
            .iter()
            .map(|q| suggestion(SuggestionType::Query, q, "desc"))
            .collect(),
    }
}

/// Render the popup above a 3-row input bar and return the buffer plus the
/// popup rectangle
fn render(state: &mut AiState, width: u16, height: u16, chat_focused: bool) -> (Buffer, Rect) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let mut popup = None;
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: height - 4,
                width,
                height: 3,
            };
            popup = render_popup(state, f, input_area, chat_focused);
        })
        .unwrap();
    (
        terminal.backend().buffer().clone(),
        popup.expect("popup should render"),
    )
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

fn line_texts(lines: &[ratatui::text::Line<'_>]) -> Vec<String> {
    lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect()
}

/// The chat input sits on the last padded row with its separator directly
/// above, and the empty input shows the prompt and placeholder
fn assert_chat_input_at_bottom(buffer: &Buffer, popup: Rect) {
    let regions = popup_regions(popup);
    let bottom_border = popup.y + popup.height - 1;
    assert_eq!(
        regions.chat_input.y + 2,
        bottom_border,
        "input row sits above the padding row and the bottom border"
    );
    assert_eq!(regions.chat_input.height, 1);

    let input = row_text(buffer, regions.chat_input.y);
    // The text area draws its cursor cell ahead of the placeholder, so allow
    // whitespace between the prompt and the placeholder text
    let after_prompt = input
        .split_once("› ")
        .map(|(_, rest)| rest.trim_start())
        .unwrap_or("");
    assert!(
        after_prompt.starts_with(PLACEHOLDER),
        "input row should show prompt and placeholder, got: {input:?}"
    );

    let separator_y = regions.chat_input.y - 1;
    for x in regions.content.x..regions.content.x + regions.content.width {
        assert_eq!(
            buffer[(x, separator_y)].symbol(),
            "─",
            "separator row {separator_y} col {x}"
        );
    }
    assert_eq!(
        buffer[(regions.content.x, separator_y)].fg,
        theme::ai::chat_separator()
    );
}

// =========================================================================
// (a) Chat input renders at the bottom in every state
// =========================================================================

#[test]
fn test_chat_input_with_suggestions() {
    let mut state = configured_state();
    state.suggestions = vec![
        suggestion(SuggestionType::Fix, ".a", "First"),
        suggestion(SuggestionType::Query, ".b", "Second"),
    ];
    let (buffer, popup) = render(&mut state, 100, 30, false);
    assert_chat_input_at_bottom(&buffer, popup);
    assert!(row_text(&buffer, popup.y + 2).contains("1. [Fix] .a"));
}

#[test]
fn test_chat_input_while_loading() {
    let mut state = configured_state();
    state.loading = true;
    let (buffer, popup) = render(&mut state, 100, 30, false);
    assert_chat_input_at_bottom(&buffer, popup);
    assert!(row_text(&buffer, popup.y + 2).contains("Thinking..."));
}

#[test]
fn test_chat_input_with_error() {
    let mut state = configured_state();
    state.error = Some("Rate limited".to_string());
    let (buffer, popup) = render(&mut state, 100, 30, false);
    assert_chat_input_at_bottom(&buffer, popup);
    assert!(row_text(&buffer, popup.y + 4).contains("Rate limited"));
}

#[test]
fn test_chat_input_with_no_suggestions() {
    let mut state = configured_state();
    state.no_suggestions = true;
    let (buffer, popup) = render(&mut state, 100, 30, false);
    assert_chat_input_at_bottom(&buffer, popup);
    assert!(row_text(&buffer, popup.y + 2).contains("No suggestions"));
}

#[test]
fn test_chat_input_in_empty_state_uses_minimum_height() {
    let mut state = configured_state();
    let (buffer, popup) = render(&mut state, 100, 30, false);
    // No header, no suggestions: 2 chat rows + 4 = 6, the minimum height
    assert_eq!(popup.height, 6);
    assert_chat_input_at_bottom(&buffer, popup);
}

#[test]
fn test_chat_input_shows_typed_text_instead_of_placeholder() {
    let mut state = configured_state();
    state.chat_input.insert_str("why is this empty?");
    let (buffer, popup) = render(&mut state, 100, 30, false);
    let input = row_text(&buffer, popup_regions(popup).chat_input.y);
    assert!(input.contains("› why is this empty?"), "got: {input:?}");
    assert!(!input.contains(PLACEHOLDER));
}

#[test]
fn test_chat_prompt_is_bold_in_prompt_color() {
    let mut state = configured_state();
    let (buffer, popup) = render(&mut state, 100, 30, false);
    let regions = popup_regions(popup);
    let cell = &buffer[(regions.chat_input.x, regions.chat_input.y)];
    assert_eq!(cell.symbol(), "›");
    assert_eq!(cell.fg, theme::ai::chat_prompt());
    assert!(cell.modifier.contains(Modifier::BOLD));
}

// =========================================================================
// (b) Bottom hints follow chat focus
// =========================================================================

fn bottom_border(buffer: &Buffer, popup: Rect) -> String {
    row_text(buffer, popup.y + popup.height - 1)
}

#[test]
fn test_hints_when_chat_focused_with_suggestions() {
    let mut state = configured_state();
    state.suggestions = vec![suggestion(SuggestionType::Fix, ".a", "First")];
    // Wide terminal so the hint line is not clipped
    let (buffer, popup) = render(&mut state, 200, 30, true);
    let hints = bottom_border(&buffer, popup);
    for expected in [
        "Alt+1-5 Apply",
        "Alt+↑↓ Select",
        "Enter Ask",
        "Esc Back",
        "Ctrl+L Clear",
        "Ctrl+A Close",
    ] {
        assert!(
            hints.contains(expected),
            "missing {expected:?} in {hints:?}"
        );
    }
    assert!(!hints.contains("Apply Selection"));
}

#[test]
fn test_hints_when_chat_unfocused_with_suggestions() {
    let mut state = configured_state();
    state.suggestions = vec![suggestion(SuggestionType::Fix, ".a", "First")];
    let (buffer, popup) = render(&mut state, 200, 30, false);
    let hints = bottom_border(&buffer, popup);
    for expected in [
        "Alt+1-5 Apply",
        "Alt+↑↓ Select",
        "Enter Apply Selection",
        "Ctrl+A Close",
    ] {
        assert!(
            hints.contains(expected),
            "missing {expected:?} in {hints:?}"
        );
    }
    assert!(!hints.contains("Enter Ask"));
    assert!(!hints.contains("Esc"));
    assert!(!hints.contains("Ctrl+L"));
}

#[test]
fn test_hints_when_chat_focused_without_suggestions() {
    let mut state = configured_state();
    let (buffer, popup) = render(&mut state, 200, 30, true);
    let hints = bottom_border(&buffer, popup);
    for expected in ["Enter Ask", "Esc Back", "Ctrl+L Clear", "Ctrl+A Close"] {
        assert!(
            hints.contains(expected),
            "missing {expected:?} in {hints:?}"
        );
    }
    assert!(!hints.contains("Alt+"));
}

#[test]
fn test_hints_when_chat_unfocused_without_suggestions() {
    let mut state = configured_state();
    let (buffer, popup) = render(&mut state, 200, 30, false);
    let hints = bottom_border(&buffer, popup);
    assert!(hints.contains("Ctrl+A Close"));
    assert!(!hints.contains("Enter"));
    assert!(!hints.contains("Alt+"));
}

// =========================================================================
// (c) Transcript rendering
// =========================================================================

fn conversation_state() -> AiState {
    let mut state = configured_state();
    state.history = vec![
        exchange("What is this?", Some("It is a list."), &[".[]"]),
        exchange("Count them", Some("Use length."), &[".[] | length"]),
    ];
    state.current_question = Some("Filter active".to_string());
    state.answer = Some("Use select.".to_string());
    state.suggestions = vec![
        suggestion(SuggestionType::Fix, ".a", "Fix it"),
        suggestion(SuggestionType::Query, ".b", "Query it"),
    ];
    state
}

#[test]
fn test_transcript_header_lines_in_order() {
    let state = conversation_state();
    let lines = build_header_lines(&state, 60);
    assert_eq!(
        line_texts(&lines),
        vec![
            "❯ What is this?",
            "It is a list.",
            "  [Query] .[]",
            "",
            "❯ Count them",
            "Use length.",
            "  [Query] .[] | length",
            "",
            "❯ Filter active",
            "Use select.",
            "",
        ]
    );
}

#[test]
fn test_transcript_styles_old_and_current_questions_differently() {
    let state = conversation_state();
    let lines = build_header_lines(&state, 60);

    let old_question = lines[0].spans[0].style;
    assert_eq!(old_question.fg, Some(theme::ai::chat_transcript()));
    assert!(old_question.add_modifier.contains(Modifier::BOLD));
    assert_eq!(
        lines[1].spans[0].style.fg,
        Some(theme::ai::chat_transcript())
    );

    assert_eq!(lines[8].spans[0].style, theme::ai::chat_question());
    assert_eq!(lines[9].spans[0].style.fg, Some(theme::ai::chat_answer()));
}

#[test]
fn test_transcript_exchange_without_answer_skips_answer_line() {
    let mut state = configured_state();
    state.history = vec![exchange("Only queries", None, &[".x", ".y"])];
    let lines = build_header_lines(&state, 60);
    assert_eq!(
        line_texts(&lines),
        vec!["❯ Only queries", "  [Query] .x", "  [Query] .y", ""]
    );
}

#[test]
fn test_transcript_wraps_long_question_with_indent() {
    let mut state = configured_state();
    state.current_question = Some("one two three four five six".to_string());
    let lines = build_header_lines(&state, 16);
    assert_eq!(
        line_texts(&lines),
        vec!["❯ one two three", "  four five six"]
    );
}

#[test]
fn test_suggestions_render_below_transcript() {
    let mut state = conversation_state();
    // Tall terminal: nothing scrolls, so content rows map straight to screen rows
    let (buffer, popup) = render(&mut state, 100, 60, false);
    let content = popup_regions(popup).content;

    let header_height = state.selection.header_height();
    assert_eq!(header_height, 11);
    assert_eq!(state.selection.scroll_offset(), 0);

    assert!(row_text(&buffer, content.y).contains("❯ What is this?"));
    assert!(row_text(&buffer, content.y + 8).contains("❯ Filter active"));
    assert!(row_text(&buffer, content.y + header_height).contains("1. [Fix] .a"));
    assert!(row_text(&buffer, content.y + header_height + 3).contains("2. [Query] .b"));

    assert_eq!(state.selection.suggestion_at_y(header_height - 1), None);
    assert_eq!(state.selection.suggestion_at_y(header_height), Some(0));
    assert_eq!(state.selection.suggestion_at_y(header_height + 2), Some(0));
    assert_eq!(state.selection.suggestion_at_y(header_height + 3), Some(1));
    assert_eq!(state.selection.suggestion_at_y(header_height + 5), None);
}

#[test]
fn test_popup_height_is_content_driven_with_transcript() {
    let mut state = conversation_state();
    let (_, popup) = render(&mut state, 100, 60, false);
    // 11 header + 5 suggestion rows + 2 chat rows + 4 for borders and titles
    assert_eq!(popup.height, 22);
}

// =========================================================================
// (d) Region geometry
// =========================================================================

#[test]
fn test_popup_regions_geometry() {
    let popup = Rect {
        x: 10,
        y: 5,
        width: 50,
        height: 14,
    };
    let regions = popup_regions(popup);
    // border (1) + padding (1) on each side; 2 rows kept for the chat input
    assert_eq!(
        regions.content,
        Rect {
            x: 12,
            y: 7,
            width: 46,
            height: 8
        }
    );
    assert_eq!(
        regions.chat_input,
        Rect {
            x: 12,
            y: 16,
            width: 46,
            height: 1
        }
    );
}

#[test]
fn test_popup_regions_too_short_for_chat_input() {
    let popup = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 4,
    };
    let regions = popup_regions(popup);
    assert_eq!(regions.content.height, 0);
    assert_eq!(regions.chat_input.height, 0);
}

#[test]
fn test_content_row_at_geometry() {
    let popup = Rect {
        x: 10,
        y: 5,
        width: 50,
        height: 14,
    };
    assert_eq!(content_row_at(popup, 5), None, "top border");
    assert_eq!(content_row_at(popup, 6), None, "top padding");
    assert_eq!(content_row_at(popup, 7), Some(0));
    assert_eq!(content_row_at(popup, 14), Some(7));
    assert_eq!(content_row_at(popup, 15), None, "separator");
    assert_eq!(content_row_at(popup, 16), None, "chat input");
    assert_eq!(content_row_at(popup, 18), None, "bottom border");
}

#[test]
fn test_is_chat_input_row_geometry() {
    let popup = Rect {
        x: 10,
        y: 5,
        width: 50,
        height: 14,
    };
    assert!(!is_chat_input_row(popup, 14), "last content row");
    assert!(is_chat_input_row(popup, 15), "separator");
    assert!(is_chat_input_row(popup, 16), "input row");
    assert!(!is_chat_input_row(popup, 17), "bottom padding");
    assert!(!is_chat_input_row(popup, 18), "bottom border");
}

#[test]
fn test_regions_match_rendered_popup() {
    let mut state = configured_state();
    state.suggestions = vec![suggestion(SuggestionType::Fix, ".a", "First")];
    let (buffer, popup) = render(&mut state, 100, 30, false);
    let regions = popup_regions(popup);

    assert_eq!(content_row_at(popup, regions.content.y), Some(0));
    assert!(row_text(&buffer, regions.content.y).contains("1. [Fix] .a"));
    assert!(is_chat_input_row(popup, regions.chat_input.y));
    assert!(row_text(&buffer, regions.chat_input.y).contains("› "));
}

// =========================================================================
// (e) Scroll to bottom keeps the newest content visible
// =========================================================================

#[test]
fn test_scroll_to_bottom_shows_latest_response() {
    let mut state = configured_state();
    state.history = (0..6)
        .map(|i| exchange(&format!("Question {i}"), Some("Answer"), &[".q"]))
        .collect();
    state.current_question = Some("Latest question".to_string());
    state.answer = Some("Latest answer".to_string());
    state.suggestions = vec![suggestion(SuggestionType::Query, ".latest", "Newest")];
    state.selection.request_scroll_to_bottom();

    // 100x30: popup is capped at 13 rows, so the content viewport is 7 rows
    let (buffer, popup) = render(&mut state, 100, 30, false);
    let content = popup_regions(popup).content;
    assert_eq!(content.height, 7);

    let max_scroll = state.selection.max_scroll();
    assert!(max_scroll > 0, "transcript must overflow the viewport");
    assert_eq!(state.selection.scroll_offset(), max_scroll);

    let rows: Vec<String> = (content.y..content.y + content.height)
        .map(|y| row_text(&buffer, y))
        .collect();
    assert!(rows.iter().any(|r| r.contains("1. [Query] .latest")));
    assert!(rows.iter().any(|r| r.contains("   Newest")));
    assert!(!rows.iter().any(|r| r.contains("Question 0")));
}

#[test]
fn test_scroll_position_survives_rerender() {
    let mut state = configured_state();
    state.history = (0..6)
        .map(|i| exchange(&format!("Question {i}"), Some("Answer"), &[".q"]))
        .collect();
    state.answer = Some("Latest answer".to_string());
    state.selection.request_scroll_to_bottom();

    // Header: 6 exchanges x 4 rows + the answer = 25 rows; viewport is 7
    render(&mut state, 100, 30, false);
    let max_scroll = state.selection.max_scroll();
    assert_eq!(max_scroll, 18);
    state.selection.scroll_view_up(3);

    let (buffer, popup) = render(&mut state, 100, 30, false);
    assert_eq!(state.selection.scroll_offset(), 15);
    let content = popup_regions(popup).content;
    let rows: Vec<String> = (content.y..content.y + content.height)
        .map(|y| row_text(&buffer, y))
        .collect();
    // Rows 15..22 of the transcript: the last exchange's question sits on
    // the second-to-last row and the newest answer is scrolled out of view
    assert!(rows[5].contains("❯ Question 5"), "rows: {rows:#?}");
    assert!(rows[6].contains("Answer"));
    assert!(!rows.iter().any(|r| r.contains("Latest answer")));
}

// =========================================================================
// (f) Snapshot
// =========================================================================

#[test]
fn snapshot_ai_popup_conversation_chat_focused() {
    let mut state = configured_state();
    state.history = vec![exchange(
        "What does this data contain?",
        Some("A list of users with an active flag."),
        &[".users[]"],
    )];
    state.current_question = Some("Only the active ones".to_string());
    state.answer = Some("Filter on the active flag.".to_string());
    state.suggestions = vec![
        suggestion(
            SuggestionType::Query,
            ".users[] | select(.active)",
            "Keeps active users",
        ),
        suggestion(
            SuggestionType::Query,
            ".users | map(select(.active)) | length",
            "Counts them",
        ),
    ];

    let mut terminal = Terminal::new(TestBackend::new(100, 44)).unwrap();
    terminal
        .draw(|f| {
            let input_area = Rect {
                x: 0,
                y: 40,
                width: 100,
                height: 3,
            };
            render_popup(&mut state, f, input_area, true);
        })
        .unwrap();
    assert_snapshot!(terminal.backend().to_string());
}
