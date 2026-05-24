//! Tests for results_events

use super::*;
use crate::app::Focus;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use std::sync::Arc;

fn setup_app_with_content(line_count: u32, viewport_height: u16) -> crate::app::App {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..line_count).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = line_count;

    app.results_scroll
        .update_bounds(line_count, viewport_height);
    app.results_cursor.update_total_lines(line_count);
    app
}

#[test]
fn test_j_moves_cursor_down_one_line() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Char('j')));

    assert_eq!(app.results_cursor.cursor_line(), 1);
}

#[test]
fn test_k_moves_cursor_up_one_line() {
    let mut app = setup_app_with_content(20, 10);
    app.results_cursor.move_to_line(5);

    app.handle_key_event(key(KeyCode::Char('k')));

    assert_eq!(app.results_cursor.cursor_line(), 4);
}

#[test]
fn test_k_at_top_stays_at_zero() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Char('k')));

    assert_eq!(app.results_cursor.cursor_line(), 0);
}

#[test]
fn test_capital_j_moves_cursor_down_ten_lines() {
    let mut app = setup_app_with_content(30, 10);

    app.handle_key_event(key(KeyCode::Char('J')));

    assert_eq!(app.results_cursor.cursor_line(), 10);
}

#[test]
fn test_capital_k_moves_cursor_up_ten_lines() {
    let mut app = setup_app_with_content(30, 10);
    app.results_cursor.move_to_line(20);

    app.handle_key_event(key(KeyCode::Char('K')));

    assert_eq!(app.results_cursor.cursor_line(), 10);
}

#[test]
fn test_g_jumps_cursor_to_top() {
    let mut app = setup_app_with_content(50, 10);
    app.results_cursor.move_to_line(25);
    app.results_scroll.offset = 20;

    app.handle_key_event(key(KeyCode::Char('g')));

    assert_eq!(app.results_cursor.cursor_line(), 0);
    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_capital_g_jumps_cursor_to_bottom() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Char('G')));

    assert_eq!(app.results_cursor.cursor_line(), 19);
}

#[test]
fn test_page_up_moves_cursor_half_page() {
    let mut app = setup_app_with_content(50, 20);
    app.results_cursor.move_to_line(25);

    app.handle_key_event(key(KeyCode::PageUp));

    assert_eq!(app.results_cursor.cursor_line(), 15);
}

#[test]
fn test_page_down_moves_cursor_half_page() {
    let mut app = setup_app_with_content(50, 20);

    app.handle_key_event(key(KeyCode::PageDown));

    assert_eq!(app.results_cursor.cursor_line(), 10);
}

#[test]
fn test_ctrl_u_moves_cursor_half_page_up() {
    let mut app = setup_app_with_content(50, 20);
    app.results_cursor.move_to_line(25);

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

    assert_eq!(app.results_cursor.cursor_line(), 15);
}

#[test]
fn test_ctrl_d_moves_cursor_half_page_down() {
    let mut app = setup_app_with_content(50, 20);

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(app.results_cursor.cursor_line(), 10);
}

#[test]
fn test_up_arrow_moves_cursor_in_results_pane() {
    let mut app = setup_app_with_content(20, 10);
    app.results_cursor.move_to_line(5);

    app.handle_key_event(key(KeyCode::Up));

    assert_eq!(app.results_cursor.cursor_line(), 4);
}

#[test]
fn test_down_arrow_moves_cursor_in_results_pane() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Down));

    assert_eq!(app.results_cursor.cursor_line(), 1);
}

#[test]
fn test_home_jumps_cursor_to_top() {
    let mut app = setup_app_with_content(50, 10);
    app.results_cursor.move_to_line(25);
    app.results_scroll.offset = 20;

    app.handle_key_event(key(KeyCode::Home));

    assert_eq!(app.results_cursor.cursor_line(), 0);
    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_scroll_follows_cursor_with_scrolloff() {
    let mut app = setup_app_with_content(50, 10);

    for _ in 0..10 {
        app.handle_key_event(key(KeyCode::Char('j')));
    }

    assert_eq!(app.results_cursor.cursor_line(), 10);
    assert!(app.results_scroll.offset > 0);
}

#[test]
fn test_cursor_clamped_at_last_line() {
    let mut app = setup_app_with_content(5, 10);

    for _ in 0..100 {
        app.handle_key_event(key(KeyCode::Char('j')));
    }

    assert_eq!(app.results_cursor.cursor_line(), 4);
}

#[test]
fn test_scroll_clamped_with_content() {
    let mut app = setup_app_with_content(20, 10);

    for _ in 0..100 {
        app.handle_key_event(key(KeyCode::Char('j')));
    }

    assert_eq!(app.results_cursor.cursor_line(), 19);
    assert!(app.results_scroll.offset <= 10);
}

#[test]
fn test_scroll_page_down_clamped() {
    let mut app = setup_app_with_content(15, 10);

    app.handle_key_event(key(KeyCode::PageDown));
    let cursor_after_first = app.results_cursor.cursor_line();

    app.handle_key_event(key(KeyCode::PageDown));
    let cursor_after_second = app.results_cursor.cursor_line();

    assert!(cursor_after_second >= cursor_after_first);
    assert!(cursor_after_second < 15);
}

#[test]
fn test_scroll_j_clamped() {
    let mut app = setup_app_with_content(5, 3);

    app.handle_key_event(key(KeyCode::Char('J')));

    assert_eq!(app.results_cursor.cursor_line(), 4);
}

#[test]
fn test_question_mark_toggles_help_in_results_pane() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(app.help.visible);
}

fn app_with_wide_content() -> crate::app::App {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    let content: String = (0..10)
        .map(|i| format!("{}{}\n", i, "x".repeat(100)))
        .collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.last_successful_result_unformatted = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;
    query_state.cached_max_line_width = content.lines().map(|l| l.len()).max().unwrap_or(0) as u16;
    app.results_scroll.update_h_bounds(101, 40);

    let widths: Vec<u16> = content
        .lines()
        .map(|l| l.len().min(u16::MAX as usize) as u16)
        .collect();
    app.results_cursor
        .update_line_widths(std::sync::Arc::new(widths));
    app.results_cursor.update_total_lines(10);
    app
}

#[test]
fn test_h_scrolls_left_one_column() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 10;

    app.handle_key_event(key(KeyCode::Char('h')));

    assert_eq!(app.results_scroll.h_offset, 9);
}

#[test]
fn test_l_scrolls_right_one_column() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('l')));

    assert_eq!(app.results_scroll.h_offset, 1);
}

#[test]
fn test_left_arrow_scrolls_left() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 10;

    app.handle_key_event(key(KeyCode::Left));

    assert_eq!(app.results_scroll.h_offset, 9);
}

#[test]
fn test_right_arrow_scrolls_right() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Right));

    assert_eq!(app.results_scroll.h_offset, 1);
}

#[test]
fn test_capital_h_scrolls_left_ten_columns() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 30;

    app.handle_key_event(key(KeyCode::Char('H')));

    assert_eq!(app.results_scroll.h_offset, 20);
}

#[test]
fn test_capital_l_scrolls_right_ten_columns() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('L')));

    assert_eq!(app.results_scroll.h_offset, 10);
}

#[test]
fn test_zero_jumps_to_left_edge() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 50;

    app.handle_key_event(key(KeyCode::Char('0')));

    assert_eq!(app.results_scroll.h_offset, 0);
}

#[test]
fn test_dollar_jumps_to_cursor_line_end() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;
    app.results_cursor.move_to_line(0);

    app.handle_key_event(key(KeyCode::Char('$')));

    let cursor_line_width = app.results_cursor.get_cursor_line_width();
    let viewport_width = app.results_scroll.viewport_width;
    let expected = cursor_line_width.saturating_sub(viewport_width);
    assert_eq!(app.results_scroll.h_offset, expected);
}

#[test]
fn test_h_scroll_left_clamped_at_zero() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('h')));

    assert_eq!(app.results_scroll.h_offset, 0);
}

#[test]
fn test_l_scroll_right_clamped_at_max() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 61;

    app.handle_key_event(key(KeyCode::Char('l')));

    assert_eq!(app.results_scroll.h_offset, 61);
}

#[test]
fn test_end_jumps_cursor_to_bottom() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::End));

    assert_eq!(app.results_cursor.cursor_line(), 19);
}

#[test]
fn test_tab_switches_focus_to_input_field() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_tab_with_ctrl_does_not_switch_focus() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key_with_mods(KeyCode::Tab, KeyModifiers::CONTROL));

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_i_key_switches_to_input_field_in_insert_mode() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
}

#[test]
fn test_i_key_switches_to_insert_mode_even_if_already_in_insert() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
}

#[test]
fn test_tab_restores_ai_popup_state() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.saved_ai_visibility_for_results = true;
    app.ai.visible = false;

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
}

#[test]
fn test_tab_restores_tooltip_state() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.saved_tooltip_visibility_for_results = true;
    app.tooltip.enabled = false;

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.tooltip.enabled);
}

#[test]
fn test_i_key_restores_ai_popup_state() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;
    app.saved_ai_visibility_for_results = true;
    app.ai.visible = false;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
    assert!(app.ai.visible);
}

#[test]
fn test_i_key_restores_tooltip_state() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;
    app.saved_tooltip_visibility_for_results = true;
    app.tooltip.enabled = false;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.tooltip.enabled);
}

#[test]
fn test_v_enters_visual_mode() {
    let mut app = setup_app_with_content(20, 10);
    app.results_cursor.move_to_line(5);

    app.handle_key_event(key(KeyCode::Char('v')));

    assert!(app.results_cursor.is_visual_mode());
    assert_eq!(app.results_cursor.selection_range(), (5, 5));
}

#[test]
fn test_visual_mode_extends_selection_with_j() {
    let mut app = setup_app_with_content(20, 10);
    app.results_cursor.move_to_line(5);

    app.handle_key_event(key(KeyCode::Char('v')));
    app.handle_key_event(key(KeyCode::Char('j')));
    app.handle_key_event(key(KeyCode::Char('j')));

    assert!(app.results_cursor.is_visual_mode());
    assert_eq!(app.results_cursor.selection_range(), (5, 7));
}

#[test]
fn test_visual_mode_extends_selection_with_k() {
    let mut app = setup_app_with_content(20, 10);
    app.results_cursor.move_to_line(10);

    app.handle_key_event(key(KeyCode::Char('v')));
    app.handle_key_event(key(KeyCode::Char('k')));
    app.handle_key_event(key(KeyCode::Char('k')));

    assert!(app.results_cursor.is_visual_mode());
    assert_eq!(app.results_cursor.selection_range(), (8, 10));
}

#[test]
fn test_esc_exits_visual_mode() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Char('v')));
    assert!(app.results_cursor.is_visual_mode());

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.results_cursor.is_visual_mode());
}

#[test]
fn test_tab_exits_visual_mode() {
    let mut app = setup_app_with_content(20, 10);

    app.handle_key_event(key(KeyCode::Char('v')));
    assert!(app.results_cursor.is_visual_mode());

    app.handle_key_event(key(KeyCode::Tab));
    assert!(!app.results_cursor.is_visual_mode());
    assert_eq!(app.focus, Focus::InputField);
}

mod drill_tests {
    use super::*;
    use crate::test_utils::test_helpers::test_app;

    fn place(app: &mut crate::app::App, row: u32) {
        app.focus = Focus::ResultsPane;
        let total = app.results_line_count_u32();
        app.results_cursor.update_total_lines(total);
        app.results_cursor.move_to_line(row);
    }

    #[test]
    fn drill_in_pipe_composes_onto_existing_query() {
        let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
        app.input.textarea.insert_str(".users");
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".users");
        }
        place(&mut app, 2);

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".users | .[0].name");
        assert_eq!(app.focus, Focus::ResultsPane, "stays on results pane");
        assert!(!app.query_undo.is_empty(), "snapshot pushed");
    }

    #[test]
    fn drill_in_replaces_when_query_empty() {
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        place(&mut app, 2);

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".b");
    }

    #[test]
    fn drill_in_at_root_no_ops_with_notification() {
        let mut app = test_app(r#"{"a": 1}"#);
        app.input.textarea.insert_str(".existing");
        place(&mut app, 0);

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".existing");
        assert_eq!(app.notification.current_message(), Some("Already at root"));
        assert!(app.query_undo.is_empty());
    }

    #[test]
    fn drill_in_with_no_resolvable_path_notifies() {
        let mut app = test_app(r#"{"a": 1}"#);
        app.input.textarea.insert_str(".existing");
        app.focus = Focus::ResultsPane;
        if let Some(qs) = app.query.as_mut() {
            qs.last_successful_result_parsed = None;
        }

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".existing");
        assert_eq!(
            app.notification.current_message(),
            Some("No path at cursor")
        );
    }

    #[test]
    fn drill_back_round_trips() {
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        app.input.textarea.insert_str(".");
        place(&mut app, 1);

        app.handle_key_event(key(KeyCode::Char('>')));
        assert_eq!(app.input.query(), ".a");

        app.handle_key_event(key(KeyCode::Char('<')));
        assert_eq!(app.input.query(), ".");
        assert!(app.query_undo.is_empty());
    }

    #[test]
    fn drill_back_on_empty_ring_notifies() {
        let mut app = test_app(r#"{"a": 1}"#);
        app.focus = Focus::ResultsPane;

        app.handle_key_event(key(KeyCode::Char('<')));

        assert_eq!(
            app.notification.current_message(),
            Some("Nothing to go back to")
        );
    }

    #[test]
    fn drill_back_after_manual_edit_pops_immediately_discarding_edits() {
        // `<` always undoes the most recent `>`, even when the user typed
        // something into the textarea in between. Simpler mental model:
        // `<` reverses `>`, edits between the two are lost.
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        place(&mut app, 1);

        app.handle_key_event(key(KeyCode::Char('>')));
        app.input.textarea.insert_str(" | .extra");

        app.handle_key_event(key(KeyCode::Char('<')));

        assert_eq!(
            app.input.query(),
            "",
            "edits discarded; prior input restored"
        );
        assert!(app.query_undo.is_empty());
    }

    #[test]
    fn deep_chain_drill_in_then_back_to_root() {
        // `>` only rewrites the textarea — execution is debounced. To
        // model a user drilling repeatedly, we simulate the worker having
        // delivered each new result by re-running `qs.execute(...)` between
        // chord presses, then repositioning the cursor onto a row of the
        // *new* parsed result.
        let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
        app.input.textarea.insert_str(".");
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".");
        }
        place(&mut app, 1);

        app.handle_key_event(key(KeyCode::Char('>')));
        assert_eq!(app.input.query(), ".users");

        if let Some(qs) = app.query.as_mut() {
            qs.execute(".users");
        }
        app.path_at_cursor.invalidate();
        place(&mut app, 1);
        app.handle_key_event(key(KeyCode::Char('>')));
        assert_eq!(app.input.query(), ".users | .[0]");

        app.handle_key_event(key(KeyCode::Char('<')));
        assert_eq!(app.input.query(), ".users");

        app.handle_key_event(key(KeyCode::Char('<')));
        assert_eq!(app.input.query(), ".");

        app.handle_key_event(key(KeyCode::Char('<')));
        assert_eq!(
            app.notification.current_message(),
            Some("Nothing to go back to")
        );
    }

    #[test]
    fn drill_in_preserves_results_pane_focus() {
        let mut app = test_app(r#"{"a": 1}"#);
        place(&mut app, 1);

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.focus, Focus::ResultsPane);
    }

    #[test]
    fn drill_in_works_in_visual_mode_after_exit() {
        // Visual mode swallows certain keys; `>` is not handled by the
        // visual-mode arm, so it falls through to the main key match arm
        // and triggers drill-in (the user's intent is most likely to drill
        // into the visually-marked region's anchor row).
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        place(&mut app, 1);
        app.handle_key_event(key(KeyCode::Char('v')));
        assert!(app.results_cursor.is_visual_mode());

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".a");
    }

    #[test]
    fn drill_in_special_key_uses_bracket_notation() {
        let mut app = test_app(r#"{"foo-bar": 1}"#);
        place(&mut app, 1);

        app.handle_key_event(key(KeyCode::Char('>')));

        assert_eq!(app.input.query(), ".[\"foo-bar\"]");
    }

    #[test]
    fn drill_back_defers_viewport_restore_until_result_lands() {
        // `<` cannot restore the cursor synchronously: the new query is
        // executed asynchronously via the debouncer, so the result-pane
        // line layout is still the *drilled* layout at the moment `<`
        // fires. Restoring against that layout would clamp the cursor.
        // Instead, `<` parks the viewport on `app.pending_viewport_restore`
        // and the main event loop applies it once the worker delivers
        // the prior query's result.
        let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3, "d": 4, "e": 5}"#);
        place(&mut app, 3);
        app.results_scroll.offset = 2;
        app.results_scroll.h_offset = 5;

        app.handle_key_event(key(KeyCode::Char('>')));
        assert_eq!(app.results_cursor.cursor_line(), 0);
        assert_eq!(app.results_scroll.offset, 0);
        assert_eq!(app.results_scroll.h_offset, 0);

        app.handle_key_event(key(KeyCode::Char('<')));
        // Snapshot is staged but cursor / scroll have NOT moved yet.
        assert_eq!(app.results_cursor.cursor_line(), 0);
        assert_eq!(app.results_scroll.offset, 0);
        assert_eq!(app.results_scroll.h_offset, 0);
        assert!(app.pending_viewport_restore.is_some());

        // Simulate the worker delivering the prior result + the main loop
        // consuming the pending restore.
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".");
        }
        app.update_stats();
        crate::path_at_cursor_apply::apply_pending_viewport_restore(&mut app);

        assert_eq!(app.results_cursor.cursor_line(), 3);
        assert_eq!(app.results_scroll.offset, 2);
        assert_eq!(app.results_scroll.h_offset, 5);
        assert!(app.pending_viewport_restore.is_none());
    }

    /// Pretty-print of `{"users": [{"name": "alice"}]}` is:
    /// ```text
    /// 0: {
    /// 1:   "users": [
    /// 2:     {
    /// 3:       "name": "alice"
    /// 4:     }
    /// 5:   ]
    /// 6: }
    /// ```
    /// Each row's "natural drill" target should be the value rendered on
    /// that row: container paths on opening / closing brackets, leaf paths
    /// on key:value rows.
    #[test]
    fn drill_in_row_semantics_match_natural_intuition() {
        let json = r#"{"users": [{"name": "alice"}]}"#;
        let cases = [
            (0, "."),              // root `{`
            (1, ".users"),         // `"users": [`
            (2, ".users[0]"),      // inner `{`
            (3, ".users[0].name"), // leaf
            (4, ".users[0]"),      // inner `}`
            (5, ".users"),         // outer `]`
            (6, "."),              // outer `}`
        ];
        for (row, expected) in cases {
            let mut app = test_app(json);
            place(&mut app, row);
            app.handle_key_event(key(KeyCode::Char('>')));
            if expected == "." {
                assert_eq!(
                    app.notification.current_message(),
                    Some("Already at root"),
                    "row {} should be at root",
                    row
                );
            } else {
                assert_eq!(
                    app.input.query(),
                    expected,
                    "row {} should drill to {}",
                    row,
                    expected
                );
            }
        }
    }
}
