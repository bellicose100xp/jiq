use crate::app::app_render_tests::render_to_string;
use crate::history::HistoryState;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;
const TOOLTIP_TEST_WIDTH: u16 = 120;
const TOOLTIP_TEST_HEIGHT: u16 = 30;

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

    app.ai.visible = false;
    app.tooltip.enabled = false;
    app.tooltip.set_current_function(None);

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_border_no_ai_hint_when_ai_visible() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);

    app.ai.visible = true;
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
fn snapshot_history_popup_scrolled_middle() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    app.history = HistoryState::empty();
    for i in 0..20 {
        app.history.add_entry_in_memory(&format!(".entry{:02}", i));
    }
    app.history.open(None);

    for _ in 0..16 {
        app.history.select_next();
    }

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_history_popup_scrolled_bottom() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    app.history = HistoryState::empty();
    for i in 0..20 {
        app.history.add_entry_in_memory(&format!(".query{:02}", i));
    }
    app.history.open(None);

    app.history.select_previous();

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
