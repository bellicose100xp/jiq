use crate::app::app_render_tests::render_to_string;
use crate::test_utils::test_helpers::test_app;

#[test]
fn test_ai_popup_visible_when_enabled() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.ai.visible = true;

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        output.contains("Anthropic")
            || output.contains("Bedrock")
            || output.contains("OpenAI")
            || output.contains("Not Configured"),
        "AI popup should be visible when ai.visible = true"
    );
}

#[test]
fn test_ai_popup_hides_tooltip_when_visible() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.tooltip.enabled = true;
    app.tooltip.set_current_function(Some("select".to_string()));
    app.ai.visible = true;

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        output.contains("Anthropic")
            || output.contains("Bedrock")
            || output.contains("OpenAI")
            || output.contains("Not Configured"),
        "AI popup should be visible"
    );
}

#[test]
fn test_tooltip_shows_when_ai_hidden_with_function() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.tooltip.enabled = true;
    app.tooltip.set_current_function(Some("select".to_string()));
    app.ai.visible = false;

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        !output.contains("Anthropic")
            && !output.contains("Bedrock")
            && !output.contains("OpenAI")
            && !output.contains("Not Configured"),
        "AI popup should not be visible when ai.visible = false"
    );
    assert!(
        output.contains("select"),
        "Tooltip should be visible when ai.visible = false and tooltip has function"
    );
}

#[test]
fn test_tooltip_hidden_when_ai_hidden_no_function() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let mut app = test_app(json);
    app.tooltip.enabled = true;
    app.ai.visible = false;

    let output = render_to_string(&mut app, 120, 30);

    assert!(
        !output.contains("Anthropic")
            && !output.contains("Bedrock")
            && !output.contains("OpenAI")
            && !output.contains("Not Configured"),
        "AI popup should not be visible"
    );
}
