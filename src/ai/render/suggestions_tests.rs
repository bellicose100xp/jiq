use super::*;

use crate::ai::ai_state::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::ai_state::{Suggestion, SuggestionType};
use crate::ai::render::text::wrap_text;

/// Build an `AiState` carrying the given suggestions, with `selected` (if any)
/// chosen via the same `navigate_next` path the UI uses for Alt+Up/Down.
fn state_with_suggestions(suggestions: Vec<Suggestion>, selected: Option<usize>) -> AiState {
    let count = suggestions.len();
    let mut state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    state.visible = true;
    state.response = "AI response".to_string();
    state.suggestions = suggestions;

    if let Some(target) = selected {
        // navigate_next(None -> 0), then advance to the target index.
        for _ in 0..=target {
            state.selection.navigate_next(count);
        }
    }

    state
}

fn suggestion(query: &str, description: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: description.to_string(),
        suggestion_type: SuggestionType::Fix,
    }
}

#[test]
fn test_render_suggestions_selected_applies_highlight_styling() {
    // A selected suggestion's query line gets the selected background (line 73)
    // and its leading "N. " span uses the selected text color (line 58); its
    // description line uses the muted desc color (93) with the selected bg (97).
    // The unselected sibling's lines carry none of that highlight.
    let state = state_with_suggestions(
        vec![
            suggestion(".users[] | select(.active)", "Filters to active users"),
            suggestion(".users[] | .email", "Extracts emails"),
        ],
        Some(0),
    );

    let lines = render_suggestions(&state, 80, wrap_text);

    let selected_bg = theme::ai::suggestion_selected_bg();

    // Layout: [0] selected query line, [1] selected description, [2] spacer,
    // [3] unselected query line, [4] unselected description.
    let selected_query_line = &lines[0];
    let selected_desc_line = &lines[1];
    let spacer_line = &lines[2];
    let unselected_query_line = &lines[3];
    let unselected_desc_line = &lines[4];

    // Line 73: selected query line carries the selected background.
    assert_eq!(
        selected_query_line.style.bg,
        Some(selected_bg),
        "selected suggestion's query line must carry the selected background"
    );

    // Line 58: the leading "1. " span uses the selected text color.
    let number_span = selected_query_line
        .spans
        .first()
        .expect("query line should have a leading number span");
    assert_eq!(
        number_span.content.as_ref(),
        "1. ",
        "first span should be the selection number"
    );
    assert_eq!(
        number_span.style.fg,
        Some(theme::ai::suggestion_text_selected()),
        "selected number span must use the selected text color"
    );

    // Lines 93 & 97: selected description uses muted fg + selected bg.
    assert_eq!(
        selected_desc_line.style.bg,
        Some(selected_bg),
        "selected description line must carry the selected background"
    );
    let desc_span = selected_desc_line
        .spans
        .first()
        .expect("description line should have a span");
    assert_eq!(
        desc_span.style.fg,
        Some(theme::ai::suggestion_desc_muted()),
        "selected description must use the muted description color"
    );

    // The spacer between suggestions is unstyled.
    assert_eq!(spacer_line.style.bg, None);

    // The unselected sibling must not inherit any selected styling.
    assert_eq!(
        unselected_query_line.style.bg, None,
        "unselected query line must not carry the selected background"
    );
    assert_eq!(
        unselected_desc_line.style.bg, None,
        "unselected description line must not carry the selected background"
    );
    let unselected_number_span = unselected_query_line
        .spans
        .first()
        .expect("unselected query line should have a leading number span");
    assert_eq!(unselected_number_span.content.as_ref(), "2. ");
    assert_eq!(
        unselected_number_span.style.fg,
        Some(theme::ai::suggestion_text_normal()),
        "unselected number span must use the normal text color"
    );
    let unselected_desc_span = unselected_desc_line
        .spans
        .first()
        .expect("unselected description line should have a span");
    assert_eq!(
        unselected_desc_span.style.fg,
        Some(theme::ai::suggestion_desc_normal()),
        "unselected description must use the normal description color"
    );
}

#[test]
fn test_render_suggestions_wrapped_query_emits_indented_continuation_lines() {
    // A query wider than the available width wraps into multiple lines. The
    // continuation lines (skip(1) loop, lines 79-85) are indented by prefix_len
    // and, when the suggestion is selected, carry the selected background.
    let long_query = ".users[] | select(.active == true and .age > 18 and .verified == true) | {name: .name, email: .email}";
    let state = state_with_suggestions(vec![suggestion(long_query, "Complex filter")], Some(0));

    let max_width = 70u16;
    let lines = render_suggestions(&state, max_width, wrap_text);

    // The prefix for the first (numbered) suggestion is "1. [Fix] ".
    let prefix = "1. [Fix] ";
    let prefix_len = prefix.len();

    // Sanity check the fixture actually wraps at this width.
    let query_max_width = (max_width as usize).saturating_sub(prefix_len);
    let wrapped = wrap_text(long_query, query_max_width);
    assert!(
        wrapped.len() > 1,
        "fixture query must wrap into multiple lines at width {max_width} (got {})",
        wrapped.len()
    );

    let selected_bg = theme::ai::suggestion_selected_bg();
    let indent = " ".repeat(prefix_len);

    // First line is the multi-span query head; subsequent query-continuation
    // lines are single-span, indented, and (since selected) carry the bg.
    // The continuation lines are lines[1..wrapped.len()].
    let continuation_count = wrapped.len() - 1;
    assert!(continuation_count >= 1);

    for cont in &lines[1..=continuation_count] {
        let span = cont
            .spans
            .first()
            .expect("continuation line should have a span");
        let text = span.content.as_ref();
        assert!(
            text.starts_with(&indent),
            "continuation line must be indented by prefix_len spaces, got {text:?}"
        );
        assert!(
            !text.trim_start().is_empty(),
            "continuation line must contain query text after the indent"
        );
        // Lines 82-83: selected continuation lines carry the selected bg.
        assert_eq!(
            cont.style.bg,
            Some(selected_bg),
            "selected wrapped query continuation must carry the selected background"
        );
        // Query text uses the query_text color (line 80).
        assert_eq!(span.style.fg, Some(theme::ai::query_text()));
    }

    // The head query line is also selected-highlighted (line 73) for contrast.
    assert_eq!(lines[0].style.bg, Some(selected_bg));
}
