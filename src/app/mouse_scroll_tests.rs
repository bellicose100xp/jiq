//! Tests for mouse scroll handling

use crate::autocomplete::Suggestion;
use crate::layout::Region;
use crate::scroll::Scrollable;
use crate::snippets::Snippet;
use crate::test_utils::test_helpers::test_app;

use super::{ScrollDirection, handle_scroll};

fn setup_app_for_scroll_tests() -> crate::app::App {
    let mut app = test_app(r#"{"test": "data"}"#);
    app.results_scroll.max_offset = 100;
    app.results_scroll.viewport_height = 20;
    app.results_scroll.offset = 10;
    app
}

#[test]
fn test_scroll_results_pane_down() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::ResultsPane), ScrollDirection::Down);

    assert_eq!(app.results_scroll.offset, initial_offset + 3);
}

#[test]
fn test_scroll_results_pane_up() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::ResultsPane), ScrollDirection::Up);

    assert_eq!(app.results_scroll.offset, initial_offset - 3);
}

#[test]
fn test_scroll_falls_back_to_results_when_none() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, None, ScrollDirection::Down);

    assert_eq!(
        app.results_scroll.offset,
        initial_offset + 3,
        "Scrolling with None region should fall back to results pane"
    );
}

#[test]
fn test_scroll_help_popup_down() {
    let mut app = setup_app_for_scroll_tests();
    app.help.visible = true;
    app.help.current_scroll_mut().max_offset = 100;
    let initial_offset = app.help.current_scroll().offset;

    handle_scroll(&mut app, Some(Region::HelpPopup), ScrollDirection::Down);

    assert_eq!(app.help.current_scroll().offset, initial_offset + 3);
}

#[test]
fn test_scroll_help_popup_up() {
    let mut app = setup_app_for_scroll_tests();
    app.help.visible = true;
    app.help.current_scroll_mut().offset = 10;
    app.help.current_scroll_mut().max_offset = 50;
    let initial_offset = app.help.current_scroll().offset;

    handle_scroll(&mut app, Some(Region::HelpPopup), ScrollDirection::Up);

    assert_eq!(app.help.current_scroll().offset, initial_offset - 3);
}

#[test]
fn test_scroll_ai_window_down() {
    let mut app = setup_app_for_scroll_tests();
    app.ai.visible = true;
    // Set up viewport and content so scrolling is possible
    app.ai.selection.update_layout(vec![1, 1, 1, 1, 1], 2); // 5 items, viewport of 2
    let initial_offset = app.ai.selection.scroll_offset();

    handle_scroll(&mut app, Some(Region::AiWindow), ScrollDirection::Down);

    assert_eq!(
        app.ai.selection.scroll_offset(),
        initial_offset + 1,
        "AI window scrolls by 1 item at a time"
    );
}

#[test]
fn test_scroll_ai_window_up() {
    let mut app = setup_app_for_scroll_tests();
    app.ai.visible = true;
    // Scroll down first to have something to scroll up from
    app.ai.selection.scroll_view_down(5);
    let initial_offset = app.ai.selection.scroll_offset();

    handle_scroll(&mut app, Some(Region::AiWindow), ScrollDirection::Up);

    assert_eq!(
        app.ai.selection.scroll_offset(),
        initial_offset.saturating_sub(1)
    );
}

#[test]
fn test_scroll_snippet_list_down() {
    let mut app = setup_app_for_scroll_tests();
    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![
        Snippet {
            name: "s1".to_string(),
            query: ".s1".to_string(),
            description: None,
        },
        Snippet {
            name: "s2".to_string(),
            query: ".s2".to_string(),
            description: None,
        },
        Snippet {
            name: "s3".to_string(),
            query: ".s3".to_string(),
            description: None,
        },
        Snippet {
            name: "s4".to_string(),
            query: ".s4".to_string(),
            description: None,
        },
    ]);
    app.snippets.open();
    // Set visible_count to create scrollable area
    app.snippets.set_visible_count(2);
    let initial_offset = app.snippets.scroll_offset();

    handle_scroll(&mut app, Some(Region::SnippetList), ScrollDirection::Down);

    assert_eq!(
        app.snippets.scroll_offset(),
        initial_offset + 1,
        "Snippet list scrolls by 1 item at a time"
    );
}

#[test]
fn test_scroll_snippet_list_up() {
    let mut app = setup_app_for_scroll_tests();
    app.snippets.disable_persistence();
    app.snippets.set_snippets(vec![
        Snippet {
            name: "s1".to_string(),
            query: ".s1".to_string(),
            description: None,
        },
        Snippet {
            name: "s2".to_string(),
            query: ".s2".to_string(),
            description: None,
        },
    ]);
    app.snippets.open();
    // Scroll down first to have something to scroll up from
    app.snippets.scroll_view_down(1);
    let initial_offset = app.snippets.scroll_offset();

    handle_scroll(&mut app, Some(Region::SnippetList), ScrollDirection::Up);

    assert!(
        app.snippets.scroll_offset() <= initial_offset,
        "Snippet list scroll up should decrease or maintain offset"
    );
}

#[test]
fn test_scroll_history_popup_down() {
    let mut app = setup_app_for_scroll_tests();
    app.history.add_entry(".entry1");
    app.history.add_entry(".entry2");
    app.history.open(None);
    // Scroll up first to create offset (history is displayed reversed)
    app.history.scroll_view_down(1);
    let initial_offset = app.history.scroll_offset();

    // Scroll down visually = scroll_view_up internally (due to reversed display)
    handle_scroll(&mut app, Some(Region::HistoryPopup), ScrollDirection::Down);

    assert_eq!(
        app.history.scroll_offset(),
        initial_offset.saturating_sub(1),
        "History popup scroll down (visual) decreases offset due to reversed display"
    );
}

#[test]
fn test_scroll_history_popup_up() {
    let mut app = setup_app_for_scroll_tests();
    // Need more entries than MAX_VISIBLE_HISTORY (15) for scrolling
    for i in 0..20 {
        app.history.add_entry(&format!(".entry{}", i));
    }
    app.history.open(None);
    let initial_offset = app.history.scroll_offset();

    // Scroll up visually = scroll_view_down internally (due to reversed display)
    handle_scroll(&mut app, Some(Region::HistoryPopup), ScrollDirection::Up);

    assert_eq!(
        app.history.scroll_offset(),
        initial_offset + 1,
        "History popup scroll up (visual) increases offset due to reversed display"
    );
}

#[test]
fn test_scroll_autocomplete_down() {
    let mut app = setup_app_for_scroll_tests();
    // Need more than MAX_VISIBLE_SUGGESTIONS (10) to enable scrolling
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| {
            Suggestion::new(
                format!(".suggestion{}", i),
                crate::autocomplete::SuggestionType::Field,
            )
        })
        .collect();
    app.autocomplete.update_suggestions(suggestions);
    let initial_offset = app.autocomplete.scroll_offset();

    handle_scroll(&mut app, Some(Region::Autocomplete), ScrollDirection::Down);

    assert_eq!(
        app.autocomplete.scroll_offset(),
        initial_offset + 1,
        "Autocomplete scrolls by 1 item at a time"
    );
}

#[test]
fn test_scroll_autocomplete_up() {
    let mut app = setup_app_for_scroll_tests();
    app.autocomplete.update_suggestions(vec![
        Suggestion::new(".suggestion1", crate::autocomplete::SuggestionType::Field),
        Suggestion::new(".suggestion2", crate::autocomplete::SuggestionType::Field),
        Suggestion::new(".suggestion3", crate::autocomplete::SuggestionType::Field),
    ]);
    // Scroll down first to have something to scroll up from
    app.autocomplete.scroll_view_down(1);
    let initial_offset = app.autocomplete.scroll_offset();

    handle_scroll(&mut app, Some(Region::Autocomplete), ScrollDirection::Up);

    assert!(
        app.autocomplete.scroll_offset() <= initial_offset,
        "Autocomplete scroll up should decrease or maintain offset"
    );
}

#[test]
fn test_scroll_input_field_does_nothing() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::InputField), ScrollDirection::Down);

    assert_eq!(
        app.results_scroll.offset, initial_offset,
        "Input field is not scrollable"
    );
}

#[test]
fn test_scroll_search_bar_does_nothing() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::SearchBar), ScrollDirection::Down);

    assert_eq!(
        app.results_scroll.offset, initial_offset,
        "Search bar is not scrollable"
    );
}

#[test]
fn test_scroll_tooltip_does_nothing() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::Tooltip), ScrollDirection::Down);

    assert_eq!(
        app.results_scroll.offset, initial_offset,
        "Tooltip is not scrollable"
    );
}

#[test]
fn test_scroll_error_overlay_does_nothing() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(&mut app, Some(Region::ErrorOverlay), ScrollDirection::Down);

    assert_eq!(
        app.results_scroll.offset, initial_offset,
        "Error overlay is not scrollable"
    );
}

#[test]
fn test_scroll_snippet_preview_does_nothing() {
    let mut app = setup_app_for_scroll_tests();
    let initial_offset = app.results_scroll.offset;

    handle_scroll(
        &mut app,
        Some(Region::SnippetPreview),
        ScrollDirection::Down,
    );

    assert_eq!(
        app.results_scroll.offset, initial_offset,
        "Snippet preview is not scrollable"
    );
}

#[test]
fn test_scroll_direction_enum() {
    assert_ne!(ScrollDirection::Up, ScrollDirection::Down);
    assert_eq!(ScrollDirection::Up, ScrollDirection::Up);
    assert_eq!(ScrollDirection::Down, ScrollDirection::Down);
}
