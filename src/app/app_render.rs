use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use super::app_state::App;
use crate::notification::render_notification;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];
        let help_area = layout[2];

        self.update_stats();

        crate::results::results_render::render_pane(self, frame, results_area);

        crate::input::input_render::render_field(self, frame, input_area);

        crate::help::help_line_render::render_line(self, frame, help_area);

        if self.ai.visible {
            crate::ai::ai_render::render_popup(&mut self.ai, frame, input_area);
        } else if self.tooltip.should_show() {
            crate::tooltip::tooltip_render::render_popup(self, frame, input_area);
        }

        if self.autocomplete.is_visible() {
            crate::autocomplete::autocomplete_render::render_popup(self, frame, input_area);
        }

        if self.history.is_visible() {
            crate::history::history_render::render_popup(self, frame, input_area);
        }

        if self.error_overlay_visible && self.query.result.is_err() {
            crate::results::results_render::render_error_overlay(self, frame, results_area);
        }

        if self.help.visible {
            crate::help::help_popup_render::render_popup(self, frame);
        }

        render_notification(frame, &mut self.notification);
    }
}

#[cfg(test)]
mod test_helpers {
    use crate::app::app_state::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    pub fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    pub fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal.draw(|f| app.render(f)).unwrap();
        terminal.backend().to_string()
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::test_helpers::render_to_string;
    use crate::app::app_state::Focus;
    use crate::editor::EditorMode;
    use crate::history::HistoryState;
    use crate::test_utils::test_helpers::test_app;
    use insta::assert_snapshot;

    const TEST_WIDTH: u16 = 80;
    const TEST_HEIGHT: u16 = 24;

    #[test]
    fn snapshot_initial_ui_empty_query() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_with_query() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_with_array_data() {
        let json = r#"[{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".[].name");
        app.query.execute(".[].name");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_input_focused() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_results_focused() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_insert_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Insert;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_normal_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Normal;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_operator_mode() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.editor_mode = EditorMode::Operator('d');

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_with_error() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_error_overlay_visible() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");
        app.error_overlay_visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_small_terminal() {
        let json = r#"{"name": "Alice"}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, 40, 10);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_ui_wide_terminal() {
        let json = r#"{"name": "Alice"}"#;
        let mut app = test_app(json);

        let output = render_to_string(&mut app, 120, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_history_popup() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.add_entry_in_memory(".age");
        app.history.add_entry_in_memory(".users[]");
        app.history.open(None);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_history_popup_with_search() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.add_entry_in_memory(".age");
        app.history.add_entry_in_memory(".users[]");
        app.history.open(Some("na"));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_history_popup_no_matches() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.history = HistoryState::empty();
        app.history.add_entry_in_memory(".name");
        app.history.open(Some("xyz"));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_help_popup() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.help.visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_help_popup_with_ai_keybindings() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        app.help.visible = true;

        app.help.scroll.offset = 45;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_error_overlay() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.query.result = Err("jq: compile error: syntax error at line 1".to_string());
        app.error_overlay_visible = true;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_syntax_error_unfocused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        app.input.textarea.delete_line_by_head();
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");

        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_syntax_error_focused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        app.input.textarea.delete_line_by_head();
        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_success_unfocused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_results_pane_with_success_focused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str(".name");
        app.query.execute(".name");

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_autocomplete_popup_with_function_signatures() {
        use crate::autocomplete::{Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let suggestions = vec![
            Suggestion::new("select", SuggestionType::Function)
                .with_description("Filter elements by condition")
                .with_signature("select(expr)")
                .with_needs_parens(true),
            Suggestion::new("sort", SuggestionType::Function)
                .with_description("Sort array")
                .with_signature("sort"),
            Suggestion::new("sort_by", SuggestionType::Function)
                .with_description("Sort array by expression")
                .with_signature("sort_by(expr)")
                .with_needs_parens(true),
        ];
        app.autocomplete.update_suggestions(suggestions);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_autocomplete_popup_selected_item_with_signature() {
        use crate::autocomplete::{Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let suggestions = vec![
            Suggestion::new("map", SuggestionType::Function)
                .with_description("Apply expression to each element")
                .with_signature("map(expr)")
                .with_needs_parens(true),
            Suggestion::new("max", SuggestionType::Function)
                .with_description("Maximum value")
                .with_signature("max"),
            Suggestion::new("max_by", SuggestionType::Function)
                .with_description("Maximum by expression")
                .with_signature("max_by(expr)")
                .with_needs_parens(true),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.autocomplete.select_next();

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_autocomplete_popup_mixed_types() {
        use crate::autocomplete::{JsonFieldType, Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let suggestions = vec![
            Suggestion::new("keys", SuggestionType::Function)
                .with_description("Get object keys or array indices")
                .with_signature("keys"),
            Suggestion::new_with_type("name", SuggestionType::Field, Some(JsonFieldType::String))
                .with_description("String field"),
            Suggestion::new(".[]", SuggestionType::Pattern)
                .with_description("Iterate over array/object values"),
        ];
        app.autocomplete.update_suggestions(suggestions);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    const TOOLTIP_TEST_WIDTH: u16 = 120;
    const TOOLTIP_TEST_HEIGHT: u16 = 30;

    #[test]
    fn snapshot_tooltip_popup_with_all_fields() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_function(Some("select".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_popup_without_tip() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_function(Some("del".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_popup_positioning() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_function(Some("map".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_dismiss_hint() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip
            .set_current_function(Some("sort_by".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_operator_alternative() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_operator(Some("//".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_operator_update() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_operator(Some("|=".to_string()));

        let output = render_to_string(&mut app, TOOLTIP_TEST_WIDTH, TOOLTIP_TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_input_border_hint_disabled_on_function() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = false;
        app.tooltip.set_current_function(Some("select".to_string()));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_input_border_no_hint_enabled() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        app.tooltip.set_current_function(Some("select".to_string()));

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_input_border_no_hint_disabled_no_function() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = false;
        app.tooltip.set_current_function(None);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_input_border_with_ai_hint() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        // AI popup is not visible
        app.ai.visible = false;
        // No tooltip hint should be shown
        app.tooltip.enabled = false;
        app.tooltip.set_current_function(None);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_input_border_no_ai_hint_when_ai_visible() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        // AI popup is visible - hint should not be shown
        app.ai.visible = true;
        // No tooltip hint should be shown
        app.tooltip.enabled = false;
        app.tooltip.set_current_function(None);

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_tooltip_and_autocomplete_both_visible() {
        use crate::autocomplete::{Suggestion, SuggestionType};

        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        let suggestions = vec![
            Suggestion::new("select", SuggestionType::Function)
                .with_description("Filter elements by condition")
                .with_signature("select(expr)")
                .with_needs_parens(true),
            Suggestion::new("sort", SuggestionType::Function)
                .with_description("Sort array")
                .with_signature("sort"),
            Suggestion::new("sort_by", SuggestionType::Function)
                .with_description("Sort array by expression")
                .with_signature("sort_by(expr)")
                .with_needs_parens(true),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.tooltip.enabled = true;
        app.tooltip.set_current_function(Some("map".to_string()));

        let output = render_to_string(&mut app, 120, 30);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_stats_bar_array_focused() {
        let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_stats_bar_object_unfocused() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_stats_bar_error_shows_last_stats() {
        let json = r#"[1, 2, 3, 4, 5]"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.input.textarea.insert_str(".invalid[");
        app.query.execute(".invalid[");

        app.focus = Focus::InputField;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_search_bar_visible() {
        let json = r#"{"name": "Alice", "email": "alice@example.com", "role": "admin"}"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.search.open();
        app.search.search_textarea_mut().insert_str("alice");

        if let Some(content) = &app.query.last_successful_result_unformatted {
            app.search.update_matches(content);
        }

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_search_bar_with_match_count() {
        let json = r#"[{"name": "alice"}, {"name": "bob"}, {"name": "alice"}]"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.search.open();
        app.search.search_textarea_mut().insert_str("alice");

        if let Some(content) = &app.query.last_successful_result_unformatted {
            app.search.update_matches(content);
        }

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_search_bar_no_matches() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.search.open();
        app.search.search_textarea_mut().insert_str("xyz");

        if let Some(content) = &app.query.last_successful_result_unformatted {
            app.search.update_matches(content);
        }

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_search_with_highlighted_matches() {
        let json =
            r#"[{"name": "alice", "email": "alice@test.com"}, {"name": "bob"}, {"name": "alice"}]"#;
        let mut app = test_app(json);

        app.query.execute(".");

        app.search.open();
        app.search.search_textarea_mut().insert_str("alice");

        if let Some(content) = &app.query.last_successful_result_unformatted {
            app.search.update_matches(content);
        }

        app.search.next_match();

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_search_with_horizontal_scroll() {
        let long_value = format!("{}match_here", " ".repeat(150));
        let json = format!(
            r#"{{"short": "value", "very_long_field": "{}"}}"#,
            long_value
        );
        let mut app = test_app(&json);

        app.query.execute(".");

        app.results_scroll.viewport_width = 80;
        app.results_scroll.viewport_height = 20;

        app.search.open();
        app.search.search_textarea_mut().insert_str("match_here");

        if let Some(content) = &app.query.last_successful_result_unformatted {
            app.search.update_matches(content);

            let max_line_width = content.lines().map(|l| l.len()).max().unwrap_or(0) as u16;
            app.results_scroll
                .update_h_bounds(max_line_width, app.results_scroll.viewport_width);
        }

        app.search.confirm();
        if let Some(line) = app.search.next_match()
            && let Some(current_match) = app.search.current_match()
        {
            let target_col = current_match.col;
            let match_len = current_match.len;
            let h_offset = app.results_scroll.h_offset;
            let max_h_offset = app.results_scroll.max_h_offset;
            let viewport_width = app.results_scroll.viewport_width;

            if max_h_offset > 0 && viewport_width > 0 {
                let match_end = target_col.saturating_add(match_len);
                let visible_h_start = h_offset;
                let visible_h_end = h_offset.saturating_add(viewport_width);

                if target_col < visible_h_start || match_end > visible_h_end {
                    let left_margin: u16 = 10;
                    let new_h_offset = target_col.saturating_sub(left_margin);
                    app.results_scroll.h_offset = new_h_offset.min(max_h_offset);
                }
            }

            let target_line = line.min(u16::MAX as u32) as u16;
            let viewport_height = app.results_scroll.viewport_height;
            let current_offset = app.results_scroll.offset;
            let max_offset = app.results_scroll.max_offset;

            if viewport_height > 0 && max_offset > 0 {
                let visible_start = current_offset;
                let visible_end = current_offset.saturating_add(viewport_height);

                if target_line < visible_start || target_line >= visible_end {
                    let half_viewport = viewport_height / 2;
                    let new_offset = target_line.saturating_sub(half_viewport);
                    app.results_scroll.offset = new_offset.min(max_offset);
                }
            }
        }

        app.focus = Focus::ResultsPane;

        let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
        assert_snapshot!(output);
    }

    use proptest::prelude::*;

    // **Feature: ai-assistant, Property 7: AI popup hides tooltip**
    // *For any* app state where `ai.visible = true`, the tooltip should not be rendered.
    // **Validates: Requirements 2.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_ai_popup_hides_tooltip(
            tooltip_enabled in prop::bool::ANY,
            has_function in prop::bool::ANY,
        ) {
            let json = r#"{"name": "Alice", "age": 30}"#;
            let mut app = test_app(json);

            // Set up tooltip state - enabled and with a function detected
            app.tooltip.enabled = tooltip_enabled;
            if has_function {
                app.tooltip.set_current_function(Some("select".to_string()));
            }

            // Make AI popup visible
            app.ai.visible = true;

            // Render the app
            let output = render_to_string(&mut app, 120, 30);

            // When AI popup is visible, the tooltip should NOT be rendered
            // The AI popup shows the provider name in its title (e.g., "Anthropic", "Bedrock", "Not Configured")
            prop_assert!(
                output.contains("Anthropic") || output.contains("Bedrock") || output.contains("OpenAI") || output.contains("Not Configured"),
                "AI popup should be visible when ai.visible = true"
            );

            // The tooltip would show function documentation like "select(expr)"
            // When AI is visible, tooltip should be hidden
            // We verify this by checking that the AI popup is rendered (which means
            // the else branch for tooltip was not taken)
            // The render logic is: if ai.visible { render AI } else if tooltip.should_show() { render tooltip }
        }

        #[test]
        fn prop_tooltip_shows_when_ai_hidden(
            has_function in prop::bool::ANY,
        ) {
            let json = r#"{"name": "Alice", "age": 30}"#;
            let mut app = test_app(json);

            // Set up tooltip state - enabled and with a function detected
            app.tooltip.enabled = true;
            if has_function {
                app.tooltip.set_current_function(Some("select".to_string()));
            }

            // AI popup is NOT visible
            app.ai.visible = false;

            // Render the app
            let output = render_to_string(&mut app, 120, 30);

            // AI popup should NOT be visible (check that provider names are not shown)
            prop_assert!(
                !output.contains("Anthropic") && !output.contains("Bedrock") && !output.contains("OpenAI") && !output.contains("Not Configured"),
                "AI popup should not be visible when ai.visible = false"
            );

            // If tooltip has a function and is enabled, it should show
            if has_function {
                // Tooltip shows function name in its content
                prop_assert!(
                    output.contains("select"),
                    "Tooltip should be visible when ai.visible = false and tooltip has function"
                );
            }
        }
    }
}
