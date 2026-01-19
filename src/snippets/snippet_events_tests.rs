use crate::autocomplete::{Suggestion, SuggestionType};
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_ctrl_s_opens_snippet_popup() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    assert!(!app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
}

#[test]
fn test_esc_closes_snippet_popup() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_hides_autocomplete_on_open() {
    let mut app = app_with_query(".a");
    app.input.editor_mode = EditorMode::Insert;
    app.autocomplete
        .update_suggestions(vec![Suggestion::new("test", SuggestionType::Field)]);
    assert!(app.autocomplete.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_snippet_popup_closes_history_on_open() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".test");
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(app.history.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
    assert!(!app.history.is_visible());
}

#[test]
fn test_snippet_popup_allows_f1_help() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.help.visible);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_allows_question_mark_help() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.help.visible);

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(app.help.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_allows_ctrl_c_quit() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.should_quit);

    app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit);
}

#[test]
fn test_snippet_popup_blocks_other_global_keys() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert!(!app.ai.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_ctrl_s_when_popup_already_open() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_captures_backtab() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::BackTab));
    assert!(app.snippets.is_visible());
    assert_eq!(app.focus, crate::app::Focus::InputField);
}

#[test]
fn test_down_arrow_navigates_to_next_snippet() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);
}

#[test]
fn test_up_arrow_navigates_to_prev_snippet() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_j_key_types_into_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Char('j')));
    assert_eq!(app.snippets.search_query(), "j");
}

#[test]
fn test_k_key_types_into_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.snippets.search_query(), "k");
}

#[test]
fn test_navigation_stops_at_last_item() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);
}

#[test]
fn test_navigation_stops_at_first_item() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".".to_string(),
            description: None,
        },
    ]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_navigation_with_empty_list() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![]);
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 0);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.snippets.selected_index(), 0);
}

#[test]
fn test_enter_applies_selected_snippet_and_closes_popup() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".foo".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".bar".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".foo");
}

#[test]
fn test_enter_applies_snippet_after_navigation() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".foo".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".bar".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".bar");
}

#[test]
fn test_enter_replaces_existing_query() {
    use crate::snippets::Snippet;

    let mut app = app_with_query(".existing | query");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".new_query".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.input.query(), ".new_query");
}

#[test]
fn test_enter_clears_error_overlay() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.error_overlay_visible = true;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.error_overlay_visible);
}

#[test]
fn test_enter_resets_scroll_position() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.results_scroll.offset = 100;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_enter_with_empty_list_just_closes() {
    let mut app = app_with_query(".existing");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".existing");
}

#[test]
fn test_enter_executes_query() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "keys query".to_string(),
        query: "keys".to_string(),
        description: Some("Get all keys".to_string()),
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(app.query.is_some());
    if let Some(ref query_state) = app.query {
        assert_eq!(
            query_state.base_query_for_suggestions,
            Some("keys".to_string())
        );
    }
}

#[test]
fn test_typing_filters_snippets() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select items".to_string(),
            query: ".[]".to_string(),
            description: None,
        },
    ]);

    assert_eq!(app.snippets.filtered_count(), 3);

    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('l')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('c')));
    app.handle_key_event(key(KeyCode::Char('t')));

    assert_eq!(app.snippets.search_query(), "select");
    assert_eq!(app.snippets.filtered_count(), 2);
}

#[test]
fn test_search_then_select_applies_filtered_snippet() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "Flatten arrays".to_string(),
            query: "flatten".to_string(),
            description: None,
        },
        Snippet {
            name: "Select keys".to_string(),
            query: "keys".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Char('k')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('s')));

    assert_eq!(app.snippets.filtered_count(), 1);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), "keys");
}

#[test]
fn test_backspace_updates_search() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    assert_eq!(app.snippets.search_query(), "test");

    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.snippets.search_query(), "tes");
}

#[test]
fn test_search_clears_when_popup_closes() {
    use crate::snippets::Snippet;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Char('x')));
    assert_eq!(app.snippets.search_query(), "x");

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(app.snippets.search_query(), "");
}

#[test]
fn test_n_key_enters_create_mode() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test | keys");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);

    app.handle_key_event(key(KeyCode::Char('n')));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.pending_query(), ".test | keys");
}

#[test]
fn test_create_mode_captures_current_query() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".foo | .bar");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.pending_query(), ".foo | .bar");
}

#[test]
fn test_esc_in_create_mode_cancels() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::Esc));
    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_typing_in_create_mode_updates_name() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));
    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));

    assert_eq!(app.snippets.name_input(), "Test");
}

#[test]
fn test_enter_in_create_mode_saves_snippet() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test | keys");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('M')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char(' ')));
    app.handle_key_event(key(KeyCode::Char('S')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('p')));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].name, "My Snip");
    assert_eq!(app.snippets.snippets()[0].query, ".test | keys");
}

#[test]
fn test_enter_with_empty_name_stays_in_create_mode() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert_eq!(app.snippets.snippets().len(), 0);
}

#[test]
fn test_create_mode_blocks_navigation_keys() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Down));
    app.handle_key_event(key(KeyCode::Up));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
}

#[test]
fn test_is_editing_true_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(!app.snippets.is_editing());

    app.handle_key_event(key(KeyCode::Char('n')));
    assert!(app.snippets.is_editing());
}

#[test]
fn test_question_mark_blocked_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('?')));

    assert!(!app.help.visible);
    assert_eq!(app.snippets.name_input(), "?");
}

#[test]
fn test_backspace_in_create_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('A')));
    app.handle_key_event(key(KeyCode::Char('B')));
    app.handle_key_event(key(KeyCode::Char('C')));
    assert_eq!(app.snippets.name_input(), "ABC");

    app.handle_key_event(key(KeyCode::Backspace));
    assert_eq!(app.snippets.name_input(), "AB");
}

#[test]
fn test_popup_closes_after_successful_save() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter));

    assert!(app.snippets.is_visible());
}

#[test]
fn test_empty_name_shows_error_notification() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Name cannot be empty"));
}

#[test]
fn test_empty_query_shows_error_notification() {
    use crate::snippets::SnippetMode;

    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('T')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("Query cannot be empty"));
}

#[test]
fn test_duplicate_name_shows_error_notification() {
    use crate::snippets::{Snippet, SnippetMode};

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Existing".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('E')));
    app.handle_key_event(key(KeyCode::Char('x')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('g')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("already exists"));
}

#[test]
fn test_case_insensitive_duplicate_shows_notification() {
    use crate::snippets::{Snippet, SnippetMode};

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "MySnippet".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('m')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('s')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('i')));
    app.handle_key_event(key(KeyCode::Char('p')));
    app.handle_key_event(key(KeyCode::Char('p')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('t')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::CreateName);
    assert!(app.notification.current().is_some());
}

#[test]
fn test_new_snippets_appear_at_top_of_list() {
    use crate::snippets::Snippet;

    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".old".to_string(),
        description: None,
    }]);
    app.handle_key_event(key(KeyCode::Char('n')));

    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.snippets.snippets()[0].name, "New");
    assert_eq!(app.snippets.selected_index(), 0);
}
