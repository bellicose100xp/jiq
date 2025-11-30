//! Type definitions for result statistics

use std::fmt;

/// Type of elements in an array
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    /// Array contains only objects
    Objects,
    /// Array contains only arrays
    Arrays,
    /// Array contains only strings
    Strings,
    /// Array contains only numbers
    Numbers,
    /// Array contains only booleans
    Booleans,
    /// Array contains only nulls
    Nulls,
    /// Array contains mixed types
    Mixed,
    /// Array is empty
    Empty,
}

impl fmt::Display for ElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementType::Objects => write!(f, "objects"),
            ElementType::Arrays => write!(f, "arrays"),
            ElementType::Strings => write!(f, "strings"),
            ElementType::Numbers => write!(f, "numbers"),
            ElementType::Booleans => write!(f, "booleans"),
            ElementType::Nulls => write!(f, "nulls"),
            ElementType::Mixed => write!(f, "mixed"),
            ElementType::Empty => write!(f, ""),
        }
    }
}

/// Statistics about a JSON result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultStats {
    /// Array with count and element type
    Array { count: usize, element_type: ElementType },
    /// Object (no key count - users care more about which keys exist)
    Object,
    /// String value
    String,
    /// Number value
    Number,
    /// Boolean value
    Boolean,
    /// Null value
    Null,
    /// Stream of separate JSON outputs (from jq iteration like .[])
    Stream { count: usize },
}

impl fmt::Display for ResultStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultStats::Array { count, element_type } => {
                match element_type {
                    ElementType::Empty => write!(f, "Array [0]"),
                    _ => write!(f, "Array [{} {}]", count, element_type),
                }
            }
            ResultStats::Object => write!(f, "Object"),
            ResultStats::String => write!(f, "String"),
            ResultStats::Number => write!(f, "Number"),
            ResultStats::Boolean => write!(f, "Boolean"),
            ResultStats::Null => write!(f, "null"),
            ResultStats::Stream { count } => write!(f, "Stream [{}]", count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_element_type_display() {
        assert_eq!(ElementType::Objects.to_string(), "objects");
        assert_eq!(ElementType::Arrays.to_string(), "arrays");
        assert_eq!(ElementType::Strings.to_string(), "strings");
        assert_eq!(ElementType::Numbers.to_string(), "numbers");
        assert_eq!(ElementType::Booleans.to_string(), "booleans");
        assert_eq!(ElementType::Nulls.to_string(), "nulls");
        assert_eq!(ElementType::Mixed.to_string(), "mixed");
        assert_eq!(ElementType::Empty.to_string(), "");
    }

    #[test]
    fn test_result_stats_array_display() {
        let stats = ResultStats::Array { count: 42, element_type: ElementType::Objects };
        assert_eq!(stats.to_string(), "Array [42 objects]");

        let stats = ResultStats::Array { count: 10, element_type: ElementType::Arrays };
        assert_eq!(stats.to_string(), "Array [10 arrays]");

        let stats = ResultStats::Array { count: 5, element_type: ElementType::Strings };
        assert_eq!(stats.to_string(), "Array [5 strings]");

        let stats = ResultStats::Array { count: 100, element_type: ElementType::Numbers };
        assert_eq!(stats.to_string(), "Array [100 numbers]");

        let stats = ResultStats::Array { count: 3, element_type: ElementType::Booleans };
        assert_eq!(stats.to_string(), "Array [3 booleans]");

        let stats = ResultStats::Array { count: 2, element_type: ElementType::Nulls };
        assert_eq!(stats.to_string(), "Array [2 nulls]");

        let stats = ResultStats::Array { count: 50, element_type: ElementType::Mixed };
        assert_eq!(stats.to_string(), "Array [50 mixed]");

        let stats = ResultStats::Array { count: 0, element_type: ElementType::Empty };
        assert_eq!(stats.to_string(), "Array [0]");
    }

    #[test]
    fn test_result_stats_scalar_display() {
        assert_eq!(ResultStats::Object.to_string(), "Object");
        assert_eq!(ResultStats::String.to_string(), "String");
        assert_eq!(ResultStats::Number.to_string(), "Number");
        assert_eq!(ResultStats::Boolean.to_string(), "Boolean");
        assert_eq!(ResultStats::Null.to_string(), "null");
    }

    #[test]
    fn test_result_stats_stream_display() {
        let stats = ResultStats::Stream { count: 3 };
        assert_eq!(stats.to_string(), "Stream [3]");
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    /// Strategy to generate arbitrary ElementType values
    fn arb_element_type() -> impl Strategy<Value = ElementType> {
        prop_oneof![
            Just(ElementType::Objects),
            Just(ElementType::Arrays),
            Just(ElementType::Strings),
            Just(ElementType::Numbers),
            Just(ElementType::Booleans),
            Just(ElementType::Nulls),
            Just(ElementType::Mixed),
            Just(ElementType::Empty),
        ]
    }

    /// Strategy to generate arbitrary ResultStats values
    fn arb_result_stats() -> impl Strategy<Value = ResultStats> {
        prop_oneof![
            // Array with arbitrary count and element type
            (0usize..10000, arb_element_type())
                .prop_map(|(count, element_type)| ResultStats::Array { count, element_type }),
            // Scalar types
            Just(ResultStats::Object),
            Just(ResultStats::String),
            Just(ResultStats::Number),
            Just(ResultStats::Boolean),
            Just(ResultStats::Null),
            // Stream with arbitrary count
            (1usize..10000).prop_map(|count| ResultStats::Stream { count }),
        ]
    }

    // Feature: stats-bar, Property 1: Type detection consistency
    // *For any* valid JSON value, the stats display type SHALL match the actual
    // JSON type of the value (Array, Object, String, Number, Boolean, null, or
    // Stream for multi-value outputs).
    // **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_type_display_consistency(stats in arb_result_stats()) {
            let display = stats.to_string();

            // Verify the display string starts with the correct type prefix
            match &stats {
                ResultStats::Array { count, element_type } => {
                    prop_assert!(
                        display.starts_with("Array ["),
                        "Array stats should start with 'Array [', got: {}",
                        display
                    );
                    prop_assert!(
                        display.ends_with(']'),
                        "Array stats should end with ']', got: {}",
                        display
                    );
                    // Verify count is present in the display
                    if *element_type == ElementType::Empty {
                        prop_assert_eq!(display, "Array [0]");
                    } else {
                        prop_assert!(
                            display.contains(&count.to_string()),
                            "Array stats should contain count {}, got: {}",
                            count,
                            display
                        );
                        // Verify element type is present
                        prop_assert!(
                            display.contains(&element_type.to_string()),
                            "Array stats should contain element type '{}', got: {}",
                            element_type,
                            display
                        );
                    }
                }
                ResultStats::Object => {
                    prop_assert_eq!(display, "Object");
                }
                ResultStats::String => {
                    prop_assert_eq!(display, "String");
                }
                ResultStats::Number => {
                    prop_assert_eq!(display, "Number");
                }
                ResultStats::Boolean => {
                    prop_assert_eq!(display, "Boolean");
                }
                ResultStats::Null => {
                    prop_assert_eq!(display, "null");
                }
                ResultStats::Stream { count } => {
                    prop_assert!(
                        display.starts_with("Stream ["),
                        "Stream stats should start with 'Stream [', got: {}",
                        display
                    );
                    prop_assert!(
                        display.ends_with(']'),
                        "Stream stats should end with ']', got: {}",
                        display
                    );
                    prop_assert!(
                        display.contains(&count.to_string()),
                        "Stream stats should contain count {}, got: {}",
                        count,
                        display
                    );
                }
            }
        }

        #[test]
        fn prop_array_display_format_matches_requirements(
            count in 0usize..10000,
            element_type in arb_element_type()
        ) {
            let stats = ResultStats::Array { count, element_type: element_type.clone() };
            let display = stats.to_string();

            // Verify format matches requirements:
            // - Empty array: "Array [0]"
            // - Non-empty array: "Array [N type]"
            match element_type {
                ElementType::Empty => {
                    prop_assert_eq!(
                        display, "Array [0]",
                        "Empty array should display as 'Array [0]'"
                    );
                }
                _ => {
                    let expected = format!("Array [{} {}]", count, element_type);
                    prop_assert_eq!(
                        display, expected,
                        "Array display format mismatch"
                    );
                }
            }
        }

        #[test]
        fn prop_stream_display_format_matches_requirements(count in 1usize..10000) {
            let stats = ResultStats::Stream { count };
            let display = stats.to_string();

            let expected = format!("Stream [{}]", count);
            prop_assert_eq!(
                display, expected,
                "Stream display format should be 'Stream [N]'"
            );
        }
    }
}
