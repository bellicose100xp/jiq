use super::tooltip_content::TooltipContent;

/// Static array of tooltip content for jq operators
pub static OPERATOR_CONTENT: &[TooltipContent] = &[
    TooltipContent::new(
        "//",
        "Alternative operator - returns right side if left is null or false",
        &[
            ".name // \"anonymous\"   # default for null/missing",
            ".count // 0            # default number",
            ".config // {}          # default empty object",
            "(.x // .y) // .z       # chain alternatives",
        ],
        Some("Only triggers on null/false - use 'if . == \"\" then ... end' for empty strings"),
    ),
    TooltipContent::new(
        "|=",
        "Update operator - transform value in place using expression",
        &[
            ".name |= ascii_upcase  # uppercase the name field",
            ".count |= . + 1        # increment count",
            ".items[] |= . * 2      # double each item",
            ".config |= . + {new: 1} # add to object",
        ],
        Some("Right side receives current value as input; use = for simple assignment"),
    ),
    TooltipContent::new(
        "//=",
        "Alternative assignment - set value only if currently null or false",
        &[
            ".count //= 0           # initialize if missing",
            ".name //= \"default\"    # set default name",
            ".config //= {}         # ensure object exists",
            ".items //= []          # ensure array exists",
        ],
        Some("Equivalent to: .field = (.field // value)"),
    ),
    TooltipContent::new(
        "..",
        "Recursive descent - generate all values in structure",
        &[
            ".. | numbers           # all numbers in tree",
            ".. | strings           # all strings in tree",
            "[.. | .id?] | map(select(. != null)) # all id fields",
            ".. | objects | select(has(\"error\")) # find error objects",
        ],
        Some("Shorthand for recurse; use with type filters to avoid duplicates"),
    ),
];

pub fn get_operator_content(operator: &str) -> Option<&'static TooltipContent> {
    OPERATOR_CONTENT.iter().find(|c| c.function == operator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_get_operator_content_alternative() {
        let content = get_operator_content("//");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.function, "//");
        assert!(!content.description.is_empty());
        assert!(content.examples.len() >= 2);
    }

    #[test]
    fn test_get_operator_content_update() {
        let content = get_operator_content("|=");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.function, "|=");
        assert!(!content.description.is_empty());
        assert!(content.examples.len() >= 2);
    }

    #[test]
    fn test_get_operator_content_alternative_assignment() {
        let content = get_operator_content("//=");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.function, "//=");
        assert!(!content.description.is_empty());
        assert!(content.examples.len() >= 2);
    }

    #[test]
    fn test_get_operator_content_recursive_descent() {
        let content = get_operator_content("..");
        assert!(content.is_some());
        let content = content.unwrap();
        assert_eq!(content.function, "..");
        assert!(!content.description.is_empty());
        assert!(content.examples.len() >= 2);
    }

    #[test]
    fn test_get_operator_content_unknown() {
        assert!(get_operator_content("??").is_none());
        assert!(get_operator_content("+").is_none());
        assert!(get_operator_content("").is_none());
    }

    // **Feature: operator-tooltips, Property 5: Operator content completeness**
    // *For any* supported operator, the tooltip content SHALL have a non-empty
    // description, at least 2 examples, and each example SHALL follow the
    // `code # comment` format.
    // **Validates: Requirements 6.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_operator_content_completeness(op_index in 0usize..OPERATOR_CONTENT.len()) {
            let content = &OPERATOR_CONTENT[op_index];

            // Non-empty description
            prop_assert!(
                !content.description.is_empty(),
                "Operator '{}' should have non-empty description",
                content.function
            );

            // At least 2 examples
            prop_assert!(
                content.examples.len() >= 2,
                "Operator '{}' should have at least 2 examples, found {}",
                content.function,
                content.examples.len()
            );

            // Each example should follow code # comment format
            for (i, example) in content.examples.iter().enumerate() {
                prop_assert!(
                    example.contains('#'),
                    "Operator '{}' example {} should contain '#' for code/comment format: '{}'",
                    content.function,
                    i,
                    example
                );
            }
        }
    }

    #[test]
    fn test_all_operators_have_content() {
        let expected_operators = ["//", "|=", "//=", ".."];
        for op in expected_operators {
            assert!(
                get_operator_content(op).is_some(),
                "Operator '{}' should have content defined",
                op
            );
        }
    }
}
