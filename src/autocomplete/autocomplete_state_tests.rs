//! Tests for AutocompleteState and related types

use super::*;

mod suggestion_type_tests {
    use super::*;

    #[test]
    fn test_suggestion_type_display_function() {
        assert_eq!(SuggestionType::Function.to_string(), "function");
    }

    #[test]
    fn test_suggestion_type_display_field() {
        assert_eq!(SuggestionType::Field.to_string(), "field");
    }

    #[test]
    fn test_suggestion_type_display_operator() {
        assert_eq!(SuggestionType::Operator.to_string(), "operator");
    }

    #[test]
    fn test_suggestion_type_display_pattern() {
        assert_eq!(SuggestionType::Pattern.to_string(), "iterator");
    }

    #[test]
    fn test_suggestion_type_equality() {
        assert_eq!(SuggestionType::Function, SuggestionType::Function);
        assert_ne!(SuggestionType::Function, SuggestionType::Field);
    }

    #[test]
    fn test_suggestion_type_clone() {
        let original = SuggestionType::Function;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_suggestion_type_debug() {
        let typ = SuggestionType::Function;
        let debug_str = format!("{:?}", typ);
        assert!(debug_str.contains("Function"));
    }
}

mod json_field_type_tests {
    use super::*;

    #[test]
    fn test_json_field_type_display_string() {
        assert_eq!(JsonFieldType::String.to_string(), "String");
    }

    #[test]
    fn test_json_field_type_display_number() {
        assert_eq!(JsonFieldType::Number.to_string(), "Number");
    }

    #[test]
    fn test_json_field_type_display_boolean() {
        assert_eq!(JsonFieldType::Boolean.to_string(), "Boolean");
    }

    #[test]
    fn test_json_field_type_display_null() {
        assert_eq!(JsonFieldType::Null.to_string(), "Null");
    }

    #[test]
    fn test_json_field_type_display_object() {
        assert_eq!(JsonFieldType::Object.to_string(), "Object");
    }

    #[test]
    fn test_json_field_type_display_array() {
        assert_eq!(JsonFieldType::Array.to_string(), "Array");
    }

    #[test]
    fn test_json_field_type_display_array_of() {
        let array_of_string = JsonFieldType::ArrayOf(Box::new(JsonFieldType::String));
        assert_eq!(array_of_string.to_string(), "Array[String]");
    }

    #[test]
    fn test_json_field_type_nested_array_of() {
        let nested = JsonFieldType::ArrayOf(Box::new(JsonFieldType::ArrayOf(Box::new(
            JsonFieldType::Number,
        ))));
        assert_eq!(nested.to_string(), "Array[Array[Number]]");
    }

    #[test]
    fn test_json_field_type_equality() {
        assert_eq!(JsonFieldType::String, JsonFieldType::String);
        assert_ne!(JsonFieldType::String, JsonFieldType::Number);
    }

    #[test]
    fn test_json_field_type_clone() {
        let original = JsonFieldType::ArrayOf(Box::new(JsonFieldType::Object));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

mod suggestion_tests {
    use super::*;

    #[test]
    fn test_suggestion_new() {
        let suggestion = Suggestion::new("test", SuggestionType::Function);
        assert_eq!(suggestion.text, "test");
        assert_eq!(suggestion.suggestion_type, SuggestionType::Function);
        assert!(suggestion.description.is_none());
        assert!(suggestion.field_type.is_none());
        assert!(suggestion.signature.is_none());
        assert!(!suggestion.needs_parens);
    }

    #[test]
    fn test_suggestion_new_with_type() {
        let suggestion = Suggestion::new_with_type(
            "field_name",
            SuggestionType::Field,
            Some(JsonFieldType::String),
        );
        assert_eq!(suggestion.text, "field_name");
        assert_eq!(suggestion.suggestion_type, SuggestionType::Field);
        assert_eq!(suggestion.field_type, Some(JsonFieldType::String));
    }

    #[test]
    fn test_suggestion_with_description() {
        let suggestion =
            Suggestion::new("test", SuggestionType::Function).with_description("A test function");
        assert_eq!(suggestion.description, Some("A test function".to_string()));
    }

    #[test]
    fn test_suggestion_with_signature() {
        let suggestion = Suggestion::new("map", SuggestionType::Function).with_signature("map(f)");
        assert_eq!(suggestion.signature, Some("map(f)".to_string()));
    }

    #[test]
    fn test_suggestion_with_needs_parens() {
        let suggestion =
            Suggestion::new("select", SuggestionType::Function).with_needs_parens(true);
        assert!(suggestion.needs_parens);
    }

    #[test]
    fn test_suggestion_builder_chain() {
        let suggestion = Suggestion::new("map", SuggestionType::Function)
            .with_description("Transform each element")
            .with_signature("map(expr)")
            .with_needs_parens(true);

        assert_eq!(suggestion.text, "map");
        assert_eq!(
            suggestion.description,
            Some("Transform each element".to_string())
        );
        assert_eq!(suggestion.signature, Some("map(expr)".to_string()));
        assert!(suggestion.needs_parens);
    }

    #[test]
    fn test_suggestion_clone() {
        let original = Suggestion::new("test", SuggestionType::Function)
            .with_description("desc")
            .with_signature("sig");
        let cloned = original.clone();

        assert_eq!(original.text, cloned.text);
        assert_eq!(original.description, cloned.description);
        assert_eq!(original.signature, cloned.signature);
    }
}

mod state_tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = AutocompleteState::new();
        assert!(!state.is_visible());
        assert!(state.suggestions().is_empty());
        assert_eq!(state.selected_index(), 0);
        assert!(state.selected().is_none());
    }

    #[test]
    fn test_default_state() {
        let state = AutocompleteState::default();
        assert!(!state.is_visible());
        assert!(state.suggestions().is_empty());
    }

    #[test]
    fn test_update_suggestions_makes_visible() {
        let mut state = AutocompleteState::new();
        let suggestions = vec![Suggestion::new("test", SuggestionType::Function)];

        state.update_suggestions(suggestions);

        assert!(state.is_visible());
        assert_eq!(state.suggestions().len(), 1);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_update_suggestions_empty_hides() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![Suggestion::new("test", SuggestionType::Function)]);
        state.update_suggestions(vec![]);

        assert!(!state.is_visible());
        assert!(state.suggestions().is_empty());
    }

    #[test]
    fn test_hide() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![Suggestion::new("test", SuggestionType::Function)]);
        state.hide();

        assert!(!state.is_visible());
        assert!(state.suggestions().is_empty());
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_next() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![
            Suggestion::new("a", SuggestionType::Function),
            Suggestion::new("b", SuggestionType::Function),
            Suggestion::new("c", SuggestionType::Function),
        ]);

        assert_eq!(state.selected_index(), 0);

        state.select_next();
        assert_eq!(state.selected_index(), 1);

        state.select_next();
        assert_eq!(state.selected_index(), 2);

        // Wraps around
        state.select_next();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_next_empty() {
        let mut state = AutocompleteState::new();
        state.select_next(); // Should not panic
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_previous() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![
            Suggestion::new("a", SuggestionType::Function),
            Suggestion::new("b", SuggestionType::Function),
            Suggestion::new("c", SuggestionType::Function),
        ]);

        // Wraps to end
        state.select_previous();
        assert_eq!(state.selected_index(), 2);

        state.select_previous();
        assert_eq!(state.selected_index(), 1);

        state.select_previous();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_previous_empty() {
        let mut state = AutocompleteState::new();
        state.select_previous(); // Should not panic
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_selected_returns_correct_suggestion() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![
            Suggestion::new("first", SuggestionType::Function),
            Suggestion::new("second", SuggestionType::Field),
        ]);

        assert_eq!(state.selected().unwrap().text, "first");

        state.select_next();
        assert_eq!(state.selected().unwrap().text, "second");
    }

    #[test]
    fn test_selected_when_not_visible() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![Suggestion::new("test", SuggestionType::Function)]);
        state.hide();

        assert!(state.selected().is_none());
    }

    #[test]
    fn test_update_suggestions_resets_index() {
        let mut state = AutocompleteState::new();
        state.update_suggestions(vec![
            Suggestion::new("a", SuggestionType::Function),
            Suggestion::new("b", SuggestionType::Function),
        ]);

        state.select_next();
        assert_eq!(state.selected_index(), 1);

        // New suggestions should reset index
        state.update_suggestions(vec![Suggestion::new("new", SuggestionType::Field)]);
        assert_eq!(state.selected_index(), 0);
    }
}
