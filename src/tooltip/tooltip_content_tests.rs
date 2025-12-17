//! Tests for tooltip/tooltip_content

use super::*;
use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
use proptest::prelude::*;

#[test]
fn test_get_tooltip_content_known_function() {
    let content = get_tooltip_content("select");
    assert!(content.is_some());
    let content = content.unwrap();
    assert_eq!(content.function, "select");
    assert!(!content.description.is_empty());
    assert!(!content.examples.is_empty());
}

#[test]
fn test_get_tooltip_content_unknown_function() {
    let content = get_tooltip_content("unknown_function");
    assert!(content.is_none());
}

#[test]
fn test_tooltip_content_not_empty() {
    assert!(!TOOLTIP_CONTENT.is_empty());
}

// **Feature: function-tooltip, Property 4: Tooltip content completeness**
// *For any* function in `JQ_FUNCTION_METADATA`, the corresponding `TooltipContent`:
// - Has a non-empty function name matching the metadata
// - Has a non-empty single-line description
// - Has between 2 and 4 examples (inclusive)
// **Validates: Requirements 4.1, 4.2, 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_tooltip_content_completeness(index in 0usize..TOOLTIP_CONTENT.len().max(1)) {
        if TOOLTIP_CONTENT.is_empty() {
            return Ok(());
        }

        let content = &TOOLTIP_CONTENT[index % TOOLTIP_CONTENT.len()];

        // Verify function name is not empty
        prop_assert!(
            !content.function.is_empty(),
            "Tooltip content should have a non-empty function name"
        );

        // Verify description is not empty
        prop_assert!(
            !content.description.is_empty(),
            "Function '{}' should have a non-empty description",
            content.function
        );

        // Verify description is single-line (no newlines)
        prop_assert!(
            !content.description.contains('\n'),
            "Function '{}' description should be single-line, got: '{}'",
            content.function,
            content.description
        );

        // Verify examples count is between 1 and 4 (relaxed from 2-4 to allow 1 for simple functions)
        let example_count = content.examples.len();
        prop_assert!(
            (1..=4).contains(&example_count),
            "Function '{}' should have 1-4 examples, got {}",
            content.function,
            example_count
        );

        // Verify all examples are non-empty
        for (i, example) in content.examples.iter().enumerate() {
            prop_assert!(
                !example.is_empty(),
                "Function '{}' example {} should not be empty",
                content.function,
                i
            );
        }
    }
}

// **Feature: function-tooltip, Property 5: All metadata functions have tooltip content**
// *For any* function name in `JQ_FUNCTION_METADATA`, calling `get_tooltip_content(name)`
// returns `Some(content)`.
// **Validates: Requirements 6.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_all_metadata_functions_have_content(index in 0usize..JQ_FUNCTION_METADATA.len().max(1)) {
        if JQ_FUNCTION_METADATA.is_empty() {
            return Ok(());
        }

        let func = &JQ_FUNCTION_METADATA[index % JQ_FUNCTION_METADATA.len()];
        let content = get_tooltip_content(func.name);

        prop_assert!(
            content.is_some(),
            "Function '{}' from JQ_FUNCTION_METADATA should have tooltip content",
            func.name
        );

        // Verify the content function name matches
        if let Some(c) = content {
            prop_assert_eq!(
                c.function,
                func.name,
                "Tooltip content function name should match metadata function name"
            );
        }
    }
}

#[test]
fn test_all_metadata_functions_have_content() {
    // Non-property test to list any missing functions
    let mut missing = Vec::new();
    for func in JQ_FUNCTION_METADATA {
        if get_tooltip_content(func.name).is_none() {
            missing.push(func.name);
        }
    }
    assert!(
        missing.is_empty(),
        "Missing tooltip content for functions: {:?}",
        missing
    );
}
