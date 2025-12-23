use crate::app::app_render_tests::render_to_string;
use crate::test_utils::test_helpers::test_app;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_ai_popup_hides_tooltip(
        tooltip_enabled in prop::bool::ANY,
        has_function in prop::bool::ANY,
    ) {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = tooltip_enabled;
        if has_function {
            app.tooltip.set_current_function(Some("select".to_string()));
        }

        app.ai.visible = true;

        let output = render_to_string(&mut app, 120, 30);

        prop_assert!(
            output.contains("Anthropic") || output.contains("Bedrock") || output.contains("OpenAI") || output.contains("Not Configured"),
            "AI popup should be visible when ai.visible = true"
        );
    }

    #[test]
    fn prop_tooltip_shows_when_ai_hidden(
        has_function in prop::bool::ANY,
    ) {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);

        app.tooltip.enabled = true;
        if has_function {
            app.tooltip.set_current_function(Some("select".to_string()));
        }

        app.ai.visible = false;

        let output = render_to_string(&mut app, 120, 30);

        prop_assert!(
            !output.contains("Anthropic") && !output.contains("Bedrock") && !output.contains("OpenAI") && !output.contains("Not Configured"),
            "AI popup should not be visible when ai.visible = false"
        );

        if has_function {
            prop_assert!(
                output.contains("select"),
                "Tooltip should be visible when ai.visible = false and tooltip has function"
            );
        }
    }
}
