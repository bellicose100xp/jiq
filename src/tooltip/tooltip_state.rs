use crate::app::App;
use crate::tooltip::{detect_function_at_cursor, detect_operator_at_cursor};

/// Update tooltip state based on cursor position. Functions take priority over operators.
pub fn update_tooltip_from_app(app: &mut App) {
    let query = app.input.query();
    let cursor_pos = app.input.textarea.cursor().1; // Column position

    // Detect function (takes priority)
    let detected_function = detect_function_at_cursor(query, cursor_pos);
    app.tooltip
        .set_current_function(detected_function.map(|s| s.to_string()));

    // Detect operator only if no function detected (function takes priority)
    let detected_operator = if detected_function.is_none() {
        detect_operator_at_cursor(query, cursor_pos)
    } else {
        None
    };
    app.tooltip
        .set_current_operator(detected_operator.map(|s| s.to_string()));
}

pub struct TooltipState {
    /// Whether tooltip feature is enabled (shows automatically)
    pub enabled: bool,
    /// Currently detected function name (if any)
    pub current_function: Option<String>,
    /// Currently detected operator (if any)
    pub current_operator: Option<String>,
}

impl TooltipState {
    pub fn new(auto_show: bool) -> Self {
        Self {
            enabled: auto_show,
            current_function: None,
            current_operator: None,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_current_function(&mut self, func: Option<String>) {
        self.current_function = func;
    }

    pub fn set_current_operator(&mut self, op: Option<String>) {
        self.current_operator = op;
    }

    pub fn should_show(&self) -> bool {
        self.enabled && (self.current_function.is_some() || self.current_operator.is_some())
    }
}

impl Default for TooltipState {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_new_tooltip_state_with_auto_show_true() {
        let state = TooltipState::new(true);
        assert!(state.enabled);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_new_tooltip_state_with_auto_show_false() {
        let state = TooltipState::new(false);
        assert!(!state.enabled);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_default_creates_enabled_state() {
        let state = TooltipState::default();
        assert!(state.enabled);
    }

    #[test]
    fn test_toggle() {
        let mut state = TooltipState::new(true);
        assert!(state.enabled);
        state.toggle();
        assert!(!state.enabled);
        state.toggle();
        assert!(state.enabled);
    }

    #[test]
    fn test_set_current_function() {
        let mut state = TooltipState::new(true);
        state.set_current_function(Some("select".to_string()));
        assert_eq!(state.current_function, Some("select".to_string()));
        state.set_current_function(None);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_should_show() {
        let mut state = TooltipState::new(true);
        // Enabled but no function
        assert!(!state.should_show());

        // Enabled with function
        state.set_current_function(Some("map".to_string()));
        assert!(state.should_show());

        // Disabled with function
        state.toggle();
        assert!(!state.should_show());

        // Disabled without function
        state.set_current_function(None);
        assert!(!state.should_show());
    }

    #[test]
    fn test_new_tooltip_state_has_no_operator() {
        let state = TooltipState::new(true);
        assert!(state.current_operator.is_none());
    }

    #[test]
    fn test_set_current_operator() {
        let mut state = TooltipState::new(true);
        state.set_current_operator(Some("//".to_string()));
        assert_eq!(state.current_operator, Some("//".to_string()));
        state.set_current_operator(None);
        assert!(state.current_operator.is_none());
    }

    #[test]
    fn test_should_show_with_operator() {
        let mut state = TooltipState::new(true);
        // Enabled but no function or operator
        assert!(!state.should_show());

        // Enabled with operator
        state.set_current_operator(Some("//".to_string()));
        assert!(state.should_show());

        // Disabled with operator
        state.toggle();
        assert!(!state.should_show());

        // Disabled without operator
        state.set_current_operator(None);
        assert!(!state.should_show());
    }

    #[test]
    fn test_should_show_with_function_or_operator() {
        let mut state = TooltipState::new(true);

        // Only function set
        state.set_current_function(Some("map".to_string()));
        assert!(state.should_show());

        // Both function and operator set
        state.set_current_operator(Some("//".to_string()));
        assert!(state.should_show());

        // Only operator set
        state.set_current_function(None);
        assert!(state.should_show());

        // Neither set
        state.set_current_operator(None);
        assert!(!state.should_show());
    }

    // **Feature: function-tooltip, Property 2: Toggle round-trip preserves state consistency**
    // *For any* tooltip state, toggling twice returns to the original enabled state.
    // Additionally, toggling from enabled to disabled sets `should_show()` to false,
    // and toggling from disabled to enabled while on a function sets `should_show()` to true.
    // **Validates: Requirements 2.1, 2.2, 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_toggle_round_trip(initial_enabled: bool, has_function: bool, function_name in "[a-z_]+") {
            let mut state = TooltipState::new(initial_enabled);
            if has_function {
                state.set_current_function(Some(function_name.clone()));
            }

            let original_enabled = state.enabled;
            let original_function = state.current_function.clone();

            // Toggle twice should return to original enabled state
            state.toggle();
            state.toggle();

            prop_assert_eq!(state.enabled, original_enabled, "Toggle round-trip should preserve enabled state");
            prop_assert_eq!(state.current_function, original_function, "Toggle should not affect current_function");
        }

        #[test]
        fn prop_toggle_disabled_hides_tooltip(function_name in "[a-z_]+") {
            let mut state = TooltipState::new(true);
            state.set_current_function(Some(function_name));

            // When enabled with function, should_show is true
            prop_assert!(state.should_show(), "Should show when enabled with function");

            // Toggle to disabled
            state.toggle();

            // should_show must be false when disabled
            prop_assert!(!state.should_show(), "Should not show when disabled");
            prop_assert!(!state.enabled, "Should be disabled after toggle");
        }

        #[test]
        fn prop_toggle_enabled_with_function_shows_tooltip(function_name in "[a-z_]+") {
            let mut state = TooltipState::new(false);
            state.set_current_function(Some(function_name));

            // When disabled, should_show is false
            prop_assert!(!state.should_show(), "Should not show when disabled");

            // Toggle to enabled
            state.toggle();

            // should_show must be true when enabled with function
            prop_assert!(state.should_show(), "Should show when enabled with function");
            prop_assert!(state.enabled, "Should be enabled after toggle");
        }

        // **Feature: function-tooltip, Property 3: Hint visibility matches tooltip state**
        // *For any* tooltip state and cursor position:
        // - When disabled AND on a function: input border hint is visible
        // - When disabled AND not on a function: no hint visible
        // - When enabled AND showing tooltip: dismiss hint on tooltip border
        // - When enabled: no hint on input border
        // **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
        #[test]
        fn prop_hint_visibility_matches_state(
            enabled: bool,
            has_function: bool,
            function_name in "[a-z_]+"
        ) {
            let mut state = TooltipState::new(enabled);
            if has_function {
                state.set_current_function(Some(function_name));
            } else {
                state.set_current_function(None);
            }

            // Compute expected hint visibility for input border
            // Input border hint shows when: disabled AND on a function
            let should_show_input_hint = !state.enabled && state.current_function.is_some();

            // Compute expected tooltip visibility (which has dismiss hint)
            // Tooltip shows when: enabled AND on a function
            let should_show_tooltip = state.should_show();

            // Verify the logic:
            // 1. When disabled AND on function: input hint visible, tooltip hidden
            if !state.enabled && state.current_function.is_some() {
                prop_assert!(should_show_input_hint, "Input hint should show when disabled + on function");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when disabled");
            }

            // 2. When disabled AND not on function: no hint, no tooltip
            if !state.enabled && state.current_function.is_none() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when disabled + no function");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when disabled");
            }

            // 3. When enabled AND on function: tooltip visible (with dismiss hint), no input hint
            if state.enabled && state.current_function.is_some() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when enabled");
                prop_assert!(should_show_tooltip, "Tooltip should show when enabled + on function");
            }

            // 4. When enabled AND not on function: no tooltip, no input hint
            if state.enabled && state.current_function.is_none() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when enabled");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when no function");
            }
        }
    }

    // ========== Integration Tests for update_tooltip_from_app ==========
    // These tests verify the delegation function works correctly with App state

    use crate::test_utils::test_helpers::test_app;

    #[test]
    fn test_update_tooltip_detects_function() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with a function
        app.input.textarea.insert_str("select(.name)");

        // Move cursor to be on "select"
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Forward); // Position 1

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_function, Some("select".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_inside_function_parens() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with a function
        app.input.textarea.insert_str("select(.name)");

        // Move cursor inside the parentheses (on ".name")
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..8 {
            // Position 8 is inside parens
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_function, Some("select".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_no_function() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query without a function
        app.input.textarea.insert_str(".name");

        update_tooltip_from_app(&mut app);

        assert!(app.tooltip.current_function.is_none());
        assert!(!app.tooltip.should_show());
    }

    #[test]
    fn test_tooltip_disabled_does_not_show() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with a function
        app.input.textarea.insert_str("select(.name)");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);

        // Disable tooltip
        app.tooltip.toggle();
        assert!(!app.tooltip.enabled);

        update_tooltip_from_app(&mut app);

        // Function should be detected but should_show returns false
        assert_eq!(app.tooltip.current_function, Some("select".to_string()));
        assert!(!app.tooltip.should_show());
    }

    // ========== Operator Tooltip Integration Tests ==========

    #[test]
    fn test_update_tooltip_detects_operator_double_slash() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with the // operator
        app.input.textarea.insert_str(".name // \"default\"");

        // Move cursor to be on the first /
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..6 {
            // Position 6 is on first /
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }

        update_tooltip_from_app(&mut app);

        assert!(app.tooltip.current_function.is_none());
        assert_eq!(app.tooltip.current_operator, Some("//".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_detects_operator_pipe_equals() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with the |= operator
        app.input.textarea.insert_str(".name |= ascii_upcase");

        // Move cursor to be on the |
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..6 {
            // Position 6 is on |
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }

        update_tooltip_from_app(&mut app);

        assert!(app.tooltip.current_function.is_none());
        assert_eq!(app.tooltip.current_operator, Some("|=".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_detects_operator_alternative_assignment() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with the //= operator
        app.input.textarea.insert_str(".count //= 0");

        // Move cursor to be on the first /
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..7 {
            // Position 7 is on first /
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }

        update_tooltip_from_app(&mut app);

        assert!(app.tooltip.current_function.is_none());
        assert_eq!(app.tooltip.current_operator, Some("//=".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_detects_operator_recursive_descent() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with the .. operator
        app.input.textarea.insert_str(".. | numbers");

        // Move cursor to be on the first .
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);

        update_tooltip_from_app(&mut app);

        assert!(app.tooltip.current_function.is_none());
        assert_eq!(app.tooltip.current_operator, Some("..".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_no_operator_on_single_pipe() {
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type a query with a single pipe (not |=)
        app.input.textarea.insert_str(".name | length");

        // Move cursor to be on the |
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Head);
        for _ in 0..6 {
            // Position 6 is on |
            app.input
                .textarea
                .move_cursor(tui_textarea::CursorMove::Forward);
        }

        update_tooltip_from_app(&mut app);

        // Single pipe should NOT trigger operator tooltip
        assert!(app.tooltip.current_function.is_none());
        assert!(app.tooltip.current_operator.is_none());
        assert!(!app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_cursor_after_operator_detects_it() {
        // When user types "//", cursor ends up at position 2 (after the operator)
        // The tooltip should still show because we detect operators immediately before cursor
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type just "//" - cursor will be at position 2 (after the operator)
        app.input.textarea.insert_str("//");

        // Cursor is now at position 2 (after the //)
        let cursor_pos = app.input.textarea.cursor().1;
        assert_eq!(
            cursor_pos, 2,
            "Cursor should be at position 2 after typing //"
        );

        update_tooltip_from_app(&mut app);

        // Cursor AFTER operator should now detect it (extended behavior)
        assert_eq!(app.tooltip.current_operator, Some("//".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_cursor_on_operator_detects_it() {
        // When cursor is ON the operator (e.g., user moved cursor back), it should detect
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        // Type "//" then move cursor back onto it
        app.input.textarea.insert_str("//");
        app.input
            .textarea
            .move_cursor(tui_textarea::CursorMove::Back); // Now on second /

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_operator, Some("//".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_cursor_after_pipe_equals() {
        // Test |= operator detection when cursor is after it
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str("|=");

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_operator, Some("|=".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_cursor_after_alternative_assignment() {
        // Test //= operator detection when cursor is after it
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str("//=");

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_operator, Some("//=".to_string()));
        assert!(app.tooltip.should_show());
    }

    #[test]
    fn test_update_tooltip_cursor_after_double_dot() {
        // Test .. operator detection when cursor is after it
        let json = r#"{"name": "test"}"#;
        let mut app = test_app(json);

        app.input.textarea.insert_str("..");

        update_tooltip_from_app(&mut app);

        assert_eq!(app.tooltip.current_operator, Some("..".to_string()));
        assert!(app.tooltip.should_show());
    }

    // ========== Property Tests for update_tooltip_from_app ==========
    // **Feature: function-tooltip, Property 1: Tooltip visibility follows cursor on functions**
    // *For any* query string containing jq functions and any cursor position, when tooltip is enabled:
    // - If cursor is inside a function's parentheses, `should_show()` returns true and `current_function` matches the innermost enclosing function
    // - If cursor is on a recognized function name, `should_show()` returns true and `current_function` matches that function
    // - If cursor is not within any function context, `should_show()` returns false
    // **Validates: Requirements 1.1, 1.2, 1.3**
    mod app_property_tests {
        use super::*;
        use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_tooltip_visibility_on_function_name(
                func_index in 0usize..JQ_FUNCTION_METADATA.len(),
                prefix in "[.| ]{0,3}",
                suffix in "[()| ]{0,5}"
            ) {
                let func = &JQ_FUNCTION_METADATA[func_index];
                let func_name = func.name;

                let query = format!("{}{}{}", prefix, func_name, suffix);
                let func_start = prefix.len();

                let json = r#"{"test": true}"#;
                let mut app = test_app(json);

                // Insert the query
                app.input.textarea.insert_str(&query);

                // Move cursor to be on the function name
                app.input.textarea.move_cursor(tui_textarea::CursorMove::Head);
                for _ in 0..func_start {
                    app.input.textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }

                // Update tooltip
                update_tooltip_from_app(&mut app);

                // When cursor is on a function name and tooltip is enabled, should_show should be true
                prop_assert!(
                    app.tooltip.should_show(),
                    "Tooltip should show when cursor is on function '{}' in query '{}'",
                    func_name,
                    query
                );
                prop_assert_eq!(
                    app.tooltip.current_function.as_deref(),
                    Some(func_name),
                    "Current function should be '{}' when cursor is on it",
                    func_name
                );
            }

            #[test]
            fn prop_tooltip_visibility_inside_function_parens(
                func_index in 0usize..JQ_FUNCTION_METADATA.len(),
                inner_content in "[.a-z0-9]{1,8}"
            ) {
                let func = &JQ_FUNCTION_METADATA[func_index];

                // Only test functions that take arguments
                if !func.needs_parens {
                    return Ok(());
                }

                let func_name = func.name;
                let query = format!("{}({})", func_name, inner_content);
                let content_start = func_name.len() + 1; // Position after opening paren

                let json = r#"{"test": true}"#;
                let mut app = test_app(json);

                // Insert the query
                app.input.textarea.insert_str(&query);

                // Move cursor inside the parentheses
                app.input.textarea.move_cursor(tui_textarea::CursorMove::Head);
                for _ in 0..content_start {
                    app.input.textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }

                // Update tooltip
                update_tooltip_from_app(&mut app);

                // When cursor is inside function parens and tooltip is enabled, should_show should be true
                prop_assert!(
                    app.tooltip.should_show(),
                    "Tooltip should show when cursor is inside '{}' parens in query '{}'",
                    func_name,
                    query
                );
                prop_assert_eq!(
                    app.tooltip.current_function.as_deref(),
                    Some(func_name),
                    "Current function should be '{}' when cursor is inside its parens",
                    func_name
                );
            }

            #[test]
            fn prop_tooltip_not_visible_outside_function_context(
                field_name in "[a-z]{1,8}"
            ) {
                // Skip field names that happen to also be jq function names
                // (e.g., "env", "add", "map", "min", "max", "not", "type", etc.)
                if JQ_FUNCTION_METADATA.iter().any(|f| f.name == field_name) {
                    return Ok(());
                }

                // Query with just field access, no function
                let query = format!(".{}", field_name);

                let json = r#"{"test": true}"#;
                let mut app = test_app(json);

                // Insert the query
                app.input.textarea.insert_str(&query);

                // Cursor is at end of query (after field name)
                update_tooltip_from_app(&mut app);

                // When cursor is not in any function context, should_show should be false
                prop_assert!(
                    !app.tooltip.should_show(),
                    "Tooltip should not show when cursor is outside function context in query '{}'",
                    query
                );
                prop_assert!(
                    app.tooltip.current_function.is_none(),
                    "Current function should be None when cursor is outside function context"
                );
            }

            #[test]
            fn prop_tooltip_disabled_never_shows(
                func_index in 0usize..JQ_FUNCTION_METADATA.len()
            ) {
                let func = &JQ_FUNCTION_METADATA[func_index];
                let func_name = func.name;
                let query = format!("{}(.x)", func_name);

                let json = r#"{"test": true}"#;
                let mut app = test_app(json);

                // Insert the query
                app.input.textarea.insert_str(&query);
                app.input.textarea.move_cursor(tui_textarea::CursorMove::Head);

                // Disable tooltip
                app.tooltip.toggle();

                // Update tooltip
                update_tooltip_from_app(&mut app);

                // When tooltip is disabled, should_show should always be false
                prop_assert!(
                    !app.tooltip.should_show(),
                    "Tooltip should not show when disabled, even on function '{}'",
                    func_name
                );
                // But current_function should still be detected
                prop_assert_eq!(
                    app.tooltip.current_function.as_deref(),
                    Some(func_name),
                    "Current function should still be detected when disabled"
                );
            }

            // **Feature: operator-tooltips, Property 4: Function priority over operator**
            // *For any* query where cursor position could match both a function context
            // (inside function parens) and an operator, the system SHALL show the function
            // tooltip, not the operator tooltip.
            // **Validates: Requirements 5.3**
            #[test]
            fn prop_function_priority_over_operator(
                func_index in 0usize..JQ_FUNCTION_METADATA.len(),
                op_index in 0usize..4usize,
                left_operand in "[.a-z]{1,5}",
                right_operand in "[a-z0-9\"]{1,5}"
            ) {
                let func = &JQ_FUNCTION_METADATA[func_index];

                // Only test functions that take arguments (have parens)
                if !func.needs_parens {
                    return Ok(());
                }

                let func_name = func.name;
                let operators = ["//", "|=", "//=", ".."];
                let op = operators[op_index];

                // Build query like "select(. // x)" or "map(.x |= . + 1)"
                let query = format!("{}({} {} {})", func_name, left_operand, op, right_operand);

                // Calculate position of operator inside the parens
                let op_start = func_name.len() + 1 + left_operand.len() + 1; // func( + left + space

                let json = r#"{"test": true}"#;
                let mut app = test_app(json);

                // Insert the query
                app.input.textarea.insert_str(&query);

                // Move cursor to be on the operator
                app.input.textarea.move_cursor(tui_textarea::CursorMove::Head);
                for _ in 0..op_start {
                    app.input.textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }

                // Update tooltip
                update_tooltip_from_app(&mut app);

                // Function should take priority - current_function should be set
                prop_assert_eq!(
                    app.tooltip.current_function.as_deref(),
                    Some(func_name),
                    "Function '{}' should be detected (priority over operator '{}') in query '{}'",
                    func_name,
                    op,
                    query
                );

                // Operator should NOT be detected when inside function parens
                prop_assert!(
                    app.tooltip.current_operator.is_none(),
                    "Operator '{}' should NOT be detected when inside function '{}' parens in query '{}'",
                    op,
                    func_name,
                    query
                );

                // Tooltip should show (function detected)
                prop_assert!(
                    app.tooltip.should_show(),
                    "Tooltip should show for function '{}' in query '{}'",
                    func_name,
                    query
                );
            }
        }
    }
}
