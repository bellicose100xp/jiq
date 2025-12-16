use crate::stats::types::{ElementType, ResultStats};

pub struct StatsParser;

impl StatsParser {
    pub fn parse(result: &str) -> ResultStats {
        let trimmed = result.trim();

        if trimmed.is_empty() {
            return ResultStats::Null;
        }

        if let Some(count) = Self::is_stream(trimmed) {
            return ResultStats::Stream { count };
        }

        match trimmed.chars().next() {
            Some('[') => {
                let count = Self::count_array_items(trimmed);
                let element_type = if count == 0 {
                    ElementType::Empty
                } else {
                    Self::detect_element_type(trimmed)
                };
                ResultStats::Array {
                    count,
                    element_type,
                }
            }
            Some('{') => ResultStats::Object,
            Some('"') => ResultStats::String,
            Some('t') | Some('f') => ResultStats::Boolean,
            Some('n') => ResultStats::Null,
            Some(c) if c.is_ascii_digit() || c == '-' => ResultStats::Number,
            _ => ResultStats::Null,
        }
    }

    fn count_array_items(result: &str) -> usize {
        let mut depth = 0;
        let mut comma_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut has_content = false;

        for ch in result.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                if depth == 1 {
                    has_content = true;
                }
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '[' | '{' => {
                    if depth == 1 {
                        has_content = true;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                }
                ',' => {
                    if depth == 1 {
                        comma_count += 1;
                    }
                }
                c if !c.is_whitespace() && depth == 1 => {
                    has_content = true;
                }
                _ => {}
            }
        }

        if has_content { comma_count + 1 } else { 0 }
    }

    fn detect_element_type(result: &str) -> ElementType {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut first_type: Option<ElementType> = None;
        let mut elements_checked = 0;
        const MAX_ELEMENTS_TO_CHECK: usize = 10;

        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;

        while i < chars.len() && elements_checked < MAX_ELEMENTS_TO_CHECK {
            let ch = chars[i];

            if escape_next {
                escape_next = false;
                i += 1;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                i += 1;
                continue;
            }

            if ch == '"' {
                if depth == 1 && !in_string {
                    let element_type = ElementType::Strings;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Strings => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                in_string = !in_string;
                i += 1;
                continue;
            }

            if in_string {
                i += 1;
                continue;
            }

            match ch {
                '[' => {
                    if depth == 1 {
                        let element_type = ElementType::Arrays;
                        match &first_type {
                            None => first_type = Some(element_type),
                            Some(t) if *t != ElementType::Arrays => return ElementType::Mixed,
                            _ => {}
                        }
                        elements_checked += 1;
                    }
                    depth += 1;
                }
                '{' => {
                    if depth == 1 {
                        let element_type = ElementType::Objects;
                        match &first_type {
                            None => first_type = Some(element_type),
                            Some(t) if *t != ElementType::Objects => return ElementType::Mixed,
                            _ => {}
                        }
                        elements_checked += 1;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                }
                't' | 'f' if depth == 1 => {
                    let element_type = ElementType::Booleans;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Booleans => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                'n' if depth == 1 => {
                    let element_type = ElementType::Nulls;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Nulls => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                c if (c.is_ascii_digit() || c == '-') && depth == 1 => {
                    let element_type = ElementType::Numbers;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Numbers => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                _ => {}
            }

            i += 1;
        }

        first_type.unwrap_or(ElementType::Empty)
    }

    fn is_stream(result: &str) -> Option<usize> {
        let mut count = 0;
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut in_value = false;

        for ch in result.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                if !in_string && depth == 0 && !in_value {
                    count += 1;
                    in_value = true;
                }
                in_string = !in_string;
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '[' | '{' => {
                    if depth == 0 && !in_value {
                        count += 1;
                        in_value = true;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        in_value = false;
                    }
                }
                't' | 'f' | 'n' if depth == 0 && !in_value => {
                    count += 1;
                    in_value = true;
                }
                c if (c.is_ascii_digit() || c == '-') && depth == 0 && !in_value => {
                    count += 1;
                    in_value = true;
                }
                c if c.is_whitespace() && depth == 0 => {
                    in_value = false;
                }
                _ => {}
            }
        }

        if count > 1 { Some(count) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_count_empty_array() {
        assert_eq!(StatsParser::count_array_items("[]"), 0);
        assert_eq!(StatsParser::count_array_items("[  ]"), 0);
        assert_eq!(StatsParser::count_array_items("[\n\t]"), 0);
    }

    #[test]
    fn test_count_simple_arrays() {
        assert_eq!(StatsParser::count_array_items("[1]"), 1);
        assert_eq!(StatsParser::count_array_items("[1, 2]"), 2);
        assert_eq!(StatsParser::count_array_items("[1, 2, 3]"), 3);
        assert_eq!(StatsParser::count_array_items(r#"["a", "b", "c"]"#), 3);
    }

    #[test]
    fn test_count_nested_arrays() {
        assert_eq!(StatsParser::count_array_items("[[1, 2], [3, 4]]"), 2);
        assert_eq!(StatsParser::count_array_items("[[1, 2, 3]]"), 1);
        assert_eq!(StatsParser::count_array_items("[[[1]], [[2]]]"), 2);
    }

    #[test]
    fn test_count_nested_objects() {
        assert_eq!(StatsParser::count_array_items(r#"[{"a": 1, "b": 2}]"#), 1);
        assert_eq!(StatsParser::count_array_items(r#"[{"a": 1}, {"b": 2}]"#), 2);
    }

    #[test]
    fn test_count_strings_with_special_chars() {
        assert_eq!(StatsParser::count_array_items(r#"["a,b", "c,d"]"#), 2);
        assert_eq!(StatsParser::count_array_items(r#"["a\"b", "c"]"#), 2);
        assert_eq!(StatsParser::count_array_items(r#"["[1,2]", "{a,b}"]"#), 2);
    }

    /// Strategy to generate a simple JSON value (non-container)
    fn arb_simple_json_value() -> impl Strategy<Value = String> {
        prop_oneof![
            (-1000i64..1000).prop_map(|n| n.to_string()),
            "[a-zA-Z0-9]{0,10}".prop_map(|s| format!(r#""{}""#, s)),
            Just("true".to_string()),
            Just("false".to_string()),
            Just("null".to_string()),
        ]
    }

    /// Strategy to generate a JSON array with known element count
    fn arb_json_array_with_count() -> impl Strategy<Value = (String, usize)> {
        prop::collection::vec(arb_simple_json_value(), 0..20).prop_map(|elements| {
            let count = elements.len();
            let json = format!("[{}]", elements.join(", "));
            (json, count)
        })
    }

    /// Strategy to generate nested JSON arrays
    fn arb_nested_json_array() -> impl Strategy<Value = (String, usize)> {
        prop::collection::vec(
            prop::collection::vec(arb_simple_json_value(), 0..5)
                .prop_map(|inner| format!("[{}]", inner.join(", "))),
            0..10,
        )
        .prop_map(|elements| {
            let count = elements.len();
            let json = format!("[{}]", elements.join(", "));
            (json, count)
        })
    }

    // Feature: stats-bar, Property 3: Array count accuracy
    // *For any* JSON array, the count displayed in stats SHALL equal the number
    // of top-level elements in the array, regardless of nesting depth within elements.
    // **Validates: Requirements 2.1, 2.7, 3.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_array_count_matches_element_count((json, expected_count) in arb_json_array_with_count()) {
            let actual_count = StatsParser::count_array_items(&json);
            prop_assert_eq!(
                actual_count, expected_count,
                "Array count mismatch for '{}': expected {}, got {}",
                json, expected_count, actual_count
            );
        }

        #[test]
        fn prop_nested_array_count_is_top_level_only((json, expected_count) in arb_nested_json_array()) {
            let actual_count = StatsParser::count_array_items(&json);
            prop_assert_eq!(
                actual_count, expected_count,
                "Nested array count mismatch for '{}': expected {}, got {}",
                json, expected_count, actual_count
            );
        }

        #[test]
        fn prop_array_count_ignores_commas_in_strings(
            elements in prop::collection::vec("[a-zA-Z]{1,5}".prop_map(|s| format!(r#""{},{}""#, s, s)), 1..10)
        ) {
            // Each element is a string containing a comma
            let expected_count = elements.len();
            let json = format!("[{}]", elements.join(", "));
            let actual_count = StatsParser::count_array_items(&json);
            prop_assert_eq!(
                actual_count, expected_count,
                "Count should ignore commas inside strings: '{}', expected {}, got {}",
                json, expected_count, actual_count
            );
        }
    }

    #[test]
    fn test_detect_empty_array() {
        assert_eq!(StatsParser::detect_element_type("[]"), ElementType::Empty);
        assert_eq!(StatsParser::detect_element_type("[  ]"), ElementType::Empty);
    }

    #[test]
    fn test_detect_objects() {
        assert_eq!(
            StatsParser::detect_element_type(r#"[{}]"#),
            ElementType::Objects
        );
        assert_eq!(
            StatsParser::detect_element_type(r#"[{"a": 1}, {"b": 2}]"#),
            ElementType::Objects
        );
    }

    #[test]
    fn test_detect_arrays() {
        assert_eq!(
            StatsParser::detect_element_type("[[]]"),
            ElementType::Arrays
        );
        assert_eq!(
            StatsParser::detect_element_type("[[1], [2, 3]]"),
            ElementType::Arrays
        );
    }

    #[test]
    fn test_detect_strings() {
        assert_eq!(
            StatsParser::detect_element_type(r#"["a"]"#),
            ElementType::Strings
        );
        assert_eq!(
            StatsParser::detect_element_type(r#"["hello", "world"]"#),
            ElementType::Strings
        );
    }

    #[test]
    fn test_detect_numbers() {
        assert_eq!(
            StatsParser::detect_element_type("[1]"),
            ElementType::Numbers
        );
        assert_eq!(
            StatsParser::detect_element_type("[1, 2, 3]"),
            ElementType::Numbers
        );
        assert_eq!(
            StatsParser::detect_element_type("[-1, 0, 100]"),
            ElementType::Numbers
        );
    }

    #[test]
    fn test_detect_booleans() {
        assert_eq!(
            StatsParser::detect_element_type("[true]"),
            ElementType::Booleans
        );
        assert_eq!(
            StatsParser::detect_element_type("[true, false]"),
            ElementType::Booleans
        );
    }

    #[test]
    fn test_detect_nulls() {
        assert_eq!(
            StatsParser::detect_element_type("[null]"),
            ElementType::Nulls
        );
        assert_eq!(
            StatsParser::detect_element_type("[null, null]"),
            ElementType::Nulls
        );
    }

    #[test]
    fn test_detect_mixed() {
        assert_eq!(
            StatsParser::detect_element_type("[1, \"a\"]"),
            ElementType::Mixed
        );
        assert_eq!(
            StatsParser::detect_element_type("[{}, []]"),
            ElementType::Mixed
        );
        assert_eq!(
            StatsParser::detect_element_type("[true, null]"),
            ElementType::Mixed
        );
    }

    /// Strategy to generate homogeneous arrays of objects
    fn arb_array_of_objects() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::collection::vec(("[a-z]{1,5}", arb_simple_json_value()), 0..3).prop_map(
                |pairs| {
                    let fields: Vec<String> = pairs
                        .iter()
                        .map(|(k, v)| format!(r#""{}": {}"#, k, v))
                        .collect();
                    format!("{{{}}}", fields.join(", "))
                },
            ),
            1..10,
        )
        .prop_map(|objects| format!("[{}]", objects.join(", ")))
    }

    /// Strategy to generate homogeneous arrays of arrays
    fn arb_array_of_arrays() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::collection::vec((-100i64..100).prop_map(|n| n.to_string()), 0..5)
                .prop_map(|inner| format!("[{}]", inner.join(", "))),
            1..10,
        )
        .prop_map(|arrays| format!("[{}]", arrays.join(", ")))
    }

    /// Strategy to generate homogeneous arrays of strings
    fn arb_array_of_strings() -> impl Strategy<Value = String> {
        prop::collection::vec(
            "[a-zA-Z0-9]{0,10}".prop_map(|s| format!(r#""{}""#, s)),
            1..10,
        )
        .prop_map(|strings| format!("[{}]", strings.join(", ")))
    }

    /// Strategy to generate homogeneous arrays of numbers
    fn arb_array_of_numbers() -> impl Strategy<Value = String> {
        prop::collection::vec((-1000i64..1000).prop_map(|n| n.to_string()), 1..10)
            .prop_map(|numbers| format!("[{}]", numbers.join(", ")))
    }

    /// Strategy to generate homogeneous arrays of booleans
    fn arb_array_of_booleans() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::bool::ANY.prop_map(|b| if b { "true" } else { "false" }.to_string()),
            1..10,
        )
        .prop_map(|bools| format!("[{}]", bools.join(", ")))
    }

    /// Strategy to generate homogeneous arrays of nulls
    fn arb_array_of_nulls() -> impl Strategy<Value = String> {
        (1usize..10).prop_map(|count| {
            let nulls: Vec<&str> = (0..count).map(|_| "null").collect();
            format!("[{}]", nulls.join(", "))
        })
    }

    // Feature: stats-bar, Property 2: Array element type detection
    // *For any* JSON array with homogeneous elements, the stats display SHALL correctly
    // identify the element type (objects, arrays, strings, numbers, booleans, nulls).
    // For arrays with heterogeneous elements, the stats SHALL display "mixed".
    // **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_detect_array_of_objects(json in arb_array_of_objects()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Objects,
                "Array of objects should detect as Objects: {}",
                json
            );
        }

        #[test]
        fn prop_detect_array_of_arrays(json in arb_array_of_arrays()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Arrays,
                "Array of arrays should detect as Arrays: {}",
                json
            );
        }

        #[test]
        fn prop_detect_array_of_strings(json in arb_array_of_strings()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Strings,
                "Array of strings should detect as Strings: {}",
                json
            );
        }

        #[test]
        fn prop_detect_array_of_numbers(json in arb_array_of_numbers()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Numbers,
                "Array of numbers should detect as Numbers: {}",
                json
            );
        }

        #[test]
        fn prop_detect_array_of_booleans(json in arb_array_of_booleans()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Booleans,
                "Array of booleans should detect as Booleans: {}",
                json
            );
        }

        #[test]
        fn prop_detect_array_of_nulls(json in arb_array_of_nulls()) {
            let element_type = StatsParser::detect_element_type(&json);
            prop_assert_eq!(
                element_type, ElementType::Nulls,
                "Array of nulls should detect as Nulls: {}",
                json
            );
        }
    }

    #[test]
    fn test_single_value_not_stream() {
        assert_eq!(StatsParser::is_stream("{}"), None);
        assert_eq!(StatsParser::is_stream("[]"), None);
        assert_eq!(StatsParser::is_stream(r#""hello""#), None);
        assert_eq!(StatsParser::is_stream("123"), None);
        assert_eq!(StatsParser::is_stream("true"), None);
        assert_eq!(StatsParser::is_stream("null"), None);
    }

    #[test]
    fn test_multiple_values_is_stream() {
        assert_eq!(StatsParser::is_stream("{} {}"), Some(2));
        assert_eq!(StatsParser::is_stream("[] []"), Some(2));
        assert_eq!(StatsParser::is_stream("1 2 3"), Some(3));
        assert_eq!(StatsParser::is_stream(r#""a" "b""#), Some(2));
        assert_eq!(StatsParser::is_stream("true false"), Some(2));
        assert_eq!(StatsParser::is_stream("null null null"), Some(3));
    }

    #[test]
    fn test_mixed_stream() {
        assert_eq!(StatsParser::is_stream(r#"{} [] "a" 1 true null"#), Some(6));
        assert_eq!(StatsParser::is_stream(r#"{"a": 1} [1, 2]"#), Some(2));
    }

    #[test]
    fn test_stream_with_newlines() {
        assert_eq!(StatsParser::is_stream("{}\n{}"), Some(2));
        assert_eq!(StatsParser::is_stream("1\n2\n3"), Some(3));
    }

    /// Strategy to generate a stream of JSON values with known count
    fn arb_json_stream() -> impl Strategy<Value = (String, usize)> {
        prop::collection::vec(
            prop_oneof![
                Just("{}".to_string()),
                Just("[]".to_string()),
                (-100i64..100).prop_map(|n| n.to_string()),
                "[a-zA-Z]{1,5}".prop_map(|s| format!(r#""{}""#, s)),
                Just("true".to_string()),
                Just("false".to_string()),
                Just("null".to_string()),
            ],
            2..10,
        )
        .prop_map(|values| {
            let count = values.len();
            let stream = values.join("\n");
            (stream, count)
        })
    }

    /// Strategy to generate a single JSON value (not a stream)
    fn arb_single_json_value() -> impl Strategy<Value = String> {
        prop_oneof![
            prop::collection::vec(
                ("[a-z]{1,3}", (-100i64..100).prop_map(|n| n.to_string())),
                0..3
            )
            .prop_map(|pairs| {
                let fields: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!(r#""{}": {}"#, k, v))
                    .collect();
                format!("{{{}}}", fields.join(", "))
            }),
            prop::collection::vec((-100i64..100).prop_map(|n| n.to_string()), 0..5)
                .prop_map(|nums| format!("[{}]", nums.join(", "))),
            "[a-zA-Z0-9]{0,10}".prop_map(|s| format!(r#""{}""#, s)),
            (-1000i64..1000).prop_map(|n| n.to_string()),
            Just("true".to_string()),
            Just("false".to_string()),
            Just("null".to_string()),
        ]
    }

    // Feature: stats-bar, Property 4: Stream detection
    // *For any* jq output containing multiple separate JSON values, the stats SHALL
    // display "Stream [N]" where N equals the count of separate values. For single-value
    // outputs, the stats SHALL NOT display "Stream" format.
    // **Validates: Requirements 1.7, 5.1, 5.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_stream_detection_counts_correctly((stream, expected_count) in arb_json_stream()) {
            let result = StatsParser::is_stream(&stream);
            prop_assert_eq!(
                result, Some(expected_count),
                "Stream should be detected with count {}: '{}'",
                expected_count, stream
            );
        }

        #[test]
        fn prop_single_value_not_detected_as_stream(json in arb_single_json_value()) {
            let result = StatsParser::is_stream(&json);
            prop_assert_eq!(
                result, None,
                "Single value should not be detected as stream: '{}'",
                json
            );
        }
    }

    #[test]
    fn test_parse_array() {
        let result = StatsParser::parse("[1, 2, 3]");
        assert_eq!(
            result,
            ResultStats::Array {
                count: 3,
                element_type: ElementType::Numbers
            }
        );
    }

    #[test]
    fn test_parse_object() {
        let result = StatsParser::parse(r#"{"a": 1}"#);
        assert_eq!(result, ResultStats::Object);
    }

    #[test]
    fn test_parse_string() {
        let result = StatsParser::parse(r#""hello""#);
        assert_eq!(result, ResultStats::String);
    }

    #[test]
    fn test_parse_number() {
        let result = StatsParser::parse("42");
        assert_eq!(result, ResultStats::Number);
    }

    #[test]
    fn test_parse_boolean() {
        let result = StatsParser::parse("true");
        assert_eq!(result, ResultStats::Boolean);
    }

    #[test]
    fn test_parse_null() {
        let result = StatsParser::parse("null");
        assert_eq!(result, ResultStats::Null);
    }

    #[test]
    fn test_parse_stream() {
        let result = StatsParser::parse("{}\n{}\n{}");
        assert_eq!(result, ResultStats::Stream { count: 3 });
    }
}
