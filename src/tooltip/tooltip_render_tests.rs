//! Tests for tooltip/tooltip_render

use super::*;
use crate::autocomplete::jq_functions::JQ_FUNCTION_METADATA;
use crate::tooltip::operator_content::OPERATOR_CONTENT;
use proptest::prelude::*;

#[test]
fn test_format_tooltip_title_function() {
    assert_eq!(format_tooltip_title(true, "select"), "fn: select");
    assert_eq!(format_tooltip_title(true, "map"), "fn: map");
    assert_eq!(format_tooltip_title(true, "sort_by"), "fn: sort_by");
}

#[test]
fn test_format_tooltip_title_operator() {
    assert_eq!(format_tooltip_title(false, "//"), "operator: //");
    assert_eq!(format_tooltip_title(false, "|="), "operator: |=");
    assert_eq!(format_tooltip_title(false, "//="), "operator: //=");
    assert_eq!(format_tooltip_title(false, ".."), "operator: ..");
}

// **Feature: operator-tooltips, Property 6: Title format correctness**
// *For any* function name, the title generator SHALL produce `fn: <name>`.
// *For any* operator, the title generator SHALL produce `operator: <op>`.
// **Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.1, 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_function_title_format(func_index in 0usize..JQ_FUNCTION_METADATA.len()) {
        let func = &JQ_FUNCTION_METADATA[func_index];
        let func_name = func.name;

        let title = format_tooltip_title(true, func_name);

        // Title should start with "fn: "
        prop_assert!(
            title.starts_with("fn: "),
            "Function title '{}' should start with 'fn: '",
            title
        );

        // Title should end with the function name
        prop_assert!(
            title.ends_with(func_name),
            "Function title '{}' should end with function name '{}'",
            title,
            func_name
        );

        // Title should be exactly "fn: <name>"
        let expected = format!("fn: {}", func_name);
        prop_assert_eq!(
            title,
            expected,
            "Function title should be exactly 'fn: {}'",
            func_name
        );
    }

    #[test]
    fn prop_operator_title_format(op_index in 0usize..OPERATOR_CONTENT.len()) {
        let op = &OPERATOR_CONTENT[op_index];
        let op_name = op.function;

        let title = format_tooltip_title(false, op_name);

        // Title should start with "operator: "
        prop_assert!(
            title.starts_with("operator: "),
            "Operator title '{}' should start with 'operator: '",
            title
        );

        // Title should end with the operator
        prop_assert!(
            title.ends_with(op_name),
            "Operator title '{}' should end with operator '{}'",
            title,
            op_name
        );

        // Title should be exactly "operator: <op>"
        let expected = format!("operator: {}", op_name);
        prop_assert_eq!(
            title,
            expected,
            "Operator title should be exactly 'operator: {}'",
            op_name
        );
    }
}
