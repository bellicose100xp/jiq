use super::autocomplete_state::{Suggestion, SuggestionType};
use std::sync::LazyLock;

/// Metadata for a jq built-in function
#[derive(Debug, Clone)]
pub struct JqFunction {
    /// Function name (e.g., "select")
    pub name: &'static str,
    /// Display signature (e.g., "select(expr)" or "keys")
    pub signature: &'static str,
    /// Description of what the function does
    pub description: &'static str,
    /// Whether the function requires arguments (needs opening paren on insert)
    pub needs_parens: bool,
}

impl JqFunction {
    /// Create a new JqFunction with all metadata
    pub const fn new(
        name: &'static str,
        signature: &'static str,
        description: &'static str,
        needs_parens: bool,
    ) -> Self {
        Self {
            name,
            signature,
            description,
            needs_parens,
        }
    }
}

/// Static list of all jq built-in function metadata
/// This list contains the authoritative metadata for each function
pub static JQ_FUNCTION_METADATA: &[JqFunction] = &[
    // ===== Functions requiring arguments (needs_parens = true) =====
    
    // Array/filter functions with arguments
    JqFunction::new("map", "map(expr)", "Apply expression to each element", true),
    JqFunction::new("select", "select(expr)", "Filter elements by condition", true),
    JqFunction::new("sort_by", "sort_by(expr)", "Sort array by expression", true),
    JqFunction::new("group_by", "group_by(expr)", "Group array elements by expression", true),
    JqFunction::new("unique_by", "unique_by(expr)", "Remove duplicates by expression", true),
    JqFunction::new("min_by", "min_by(expr)", "Minimum by expression", true),
    JqFunction::new("max_by", "max_by(expr)", "Maximum by expression", true),
    JqFunction::new("limit", "limit(num; expr)", "Limit output count", true),
    JqFunction::new("nth", "nth(num)", "Nth element", true),
    JqFunction::new("range", "range(num)", "Generate range", true),
    JqFunction::new("until", "until(cond; update)", "Repeat until condition", true),
    JqFunction::new("while", "while(cond; update)", "Repeat while condition", true),
    JqFunction::new("recurse", "recurse(expr)", "Apply recursively", true),
    JqFunction::new("walk", "walk(expr)", "Apply to all values recursively", true),
    JqFunction::new("with_entries", "with_entries(expr)", "Transform object entries", true),
    
    // Object functions with arguments
    JqFunction::new("has", "has(key)", "Check if key exists", true),
    JqFunction::new("del", "del(path)", "Delete key/path", true),
    JqFunction::new("getpath", "getpath(path)", "Get value at path", true),
    JqFunction::new("setpath", "setpath(path; val)", "Set value at path", true),
    JqFunction::new("delpaths", "delpaths(paths)", "Delete multiple paths", true),
    
    // String functions with arguments
    JqFunction::new("split", "split(str)", "Split string by delimiter", true),
    JqFunction::new("join", "join(str)", "Join array with delimiter", true),
    JqFunction::new("ltrimstr", "ltrimstr(str)", "Remove prefix string", true),
    JqFunction::new("rtrimstr", "rtrimstr(str)", "Remove suffix string", true),
    JqFunction::new("startswith", "startswith(str)", "Check if starts with value", true),
    JqFunction::new("endswith", "endswith(str)", "Check if ends with value", true),
    JqFunction::new("test", "test(regex)", "Test regex match", true),
    JqFunction::new("match", "match(regex)", "Match regex", true),
    JqFunction::new("capture", "capture(regex)", "Capture regex groups", true),
    JqFunction::new("scan", "scan(regex)", "Scan for all regex matches", true),
    JqFunction::new("splits", "splits(regex)", "Split by regex", true),
    JqFunction::new("sub", "sub(regex; str)", "Replace first regex match", true),
    JqFunction::new("gsub", "gsub(regex; str)", "Replace all regex matches", true),
    
    // Comparison/search functions with arguments
    JqFunction::new("contains", "contains(val)", "Check if contains value", true),
    JqFunction::new("inside", "inside(val)", "Check if element is inside array", true),
    JqFunction::new("index", "index(val)", "Find first index of value", true),
    JqFunction::new("rindex", "rindex(val)", "Find last index of value", true),
    JqFunction::new("indices", "indices(val)", "Find all indices of value", true),
    
    // Date functions with arguments
    JqFunction::new("strftime", "strftime(fmt)", "Format timestamp", true),
    JqFunction::new("strptime", "strptime(fmt)", "Parse timestamp", true),
    
    // Date functions without arguments (ISO 8601 only)
    JqFunction::new("fromdate", "fromdate", "Parse ISO 8601 date to timestamp", false),
    JqFunction::new("todate", "todate", "Format timestamp as ISO 8601", false),
    
    // ===== Functions not requiring arguments (needs_parens = false) =====
    
    // Array functions without arguments
    JqFunction::new("keys", "keys", "Get object keys or array indices", false),
    JqFunction::new("keys_unsorted", "keys_unsorted", "Get object keys (unsorted)", false),
    JqFunction::new("values", "values", "Get all values", false),
    JqFunction::new("sort", "sort", "Sort array", false),
    JqFunction::new("reverse", "reverse", "Reverse array", false),
    JqFunction::new("unique", "unique", "Remove duplicate values", false),
    JqFunction::new("flatten", "flatten", "Flatten nested arrays", false),
    JqFunction::new("add", "add", "Sum array elements or concatenate", false),
    JqFunction::new("length", "length", "Length of array/object/string", false),
    JqFunction::new("first", "first", "First element", false),
    JqFunction::new("last", "last", "Last element", false),
    JqFunction::new("min", "min", "Minimum value", false),
    JqFunction::new("max", "max", "Maximum value", false),
    JqFunction::new("transpose", "transpose", "Transpose matrix", false),
    
    // Object functions without arguments
    JqFunction::new("to_entries", "to_entries", "Convert object to key-value pairs", false),
    JqFunction::new("from_entries", "from_entries", "Convert key-value pairs to object", false),
    JqFunction::new("paths", "paths", "Get all paths (leaf paths)", false),
    JqFunction::new("leaf_paths", "leaf_paths", "Get all leaf paths", false),
    
    // Type functions without arguments
    JqFunction::new("type", "type", "Get value type", false),
    JqFunction::new("tostring", "tostring", "Convert to string", false),
    JqFunction::new("tonumber", "tonumber", "Convert to number", false),
    JqFunction::new("arrays", "arrays", "Select arrays", false),
    JqFunction::new("objects", "objects", "Select objects", false),
    JqFunction::new("iterables", "iterables", "Select arrays/objects", false),
    JqFunction::new("booleans", "booleans", "Select booleans", false),
    JqFunction::new("numbers", "numbers", "Select numbers", false),
    JqFunction::new("strings", "strings", "Select strings", false),
    JqFunction::new("nulls", "nulls", "Select nulls", false),
    JqFunction::new("scalars", "scalars", "Select non-iterable values", false),
    
    // Math functions without arguments
    JqFunction::new("floor", "floor", "Round down", false),
    JqFunction::new("ceil", "ceil", "Round up", false),
    JqFunction::new("round", "round", "Round to nearest", false),
    JqFunction::new("sqrt", "sqrt", "Square root", false),
    JqFunction::new("abs", "abs", "Absolute value", false),
    
    // Other functions without arguments
    JqFunction::new("now", "now", "Current Unix timestamp", false),
    JqFunction::new("empty", "empty", "Produce no output", false),
    JqFunction::new("error", "error", "Raise error", false),
    JqFunction::new("not", "not", "Logical NOT", false),
    JqFunction::new("ascii_downcase", "ascii_downcase", "Convert to lowercase", false),
    JqFunction::new("ascii_upcase", "ascii_upcase", "Convert to uppercase", false),
    JqFunction::new("env", "env", "Access environment variables", false),
];

/// Static list of all jq built-in functions, operators, and patterns
/// Built once at first access and reused for performance
static JQ_BUILTINS: LazyLock<Vec<Suggestion>> = LazyLock::new(|| {
    let mut builtins = Vec::new();

    // Common patterns
    builtins.extend(vec![
        Suggestion::new(".[]", SuggestionType::Pattern)
            .with_description("Iterate over array/object values"),
        Suggestion::new(".[0]", SuggestionType::Pattern).with_description("First array element"),
        Suggestion::new(".[-1]", SuggestionType::Pattern).with_description("Last array element"),
        Suggestion::new("..", SuggestionType::Pattern)
            .with_description("Recursive descent (all values)"),
    ]);

    // Operators
    builtins.extend(vec![
        Suggestion::new("|", SuggestionType::Operator).with_description("Pipe operator"),
        Suggestion::new("//", SuggestionType::Operator)
            .with_description("Alternative operator (default value)"),
        Suggestion::new("and", SuggestionType::Operator).with_description("Logical AND"),
        Suggestion::new("or", SuggestionType::Operator).with_description("Logical OR"),
    ]);

    // Add all functions from metadata
    for func in JQ_FUNCTION_METADATA {
        builtins.push(
            Suggestion::new(func.name, SuggestionType::Function)
                .with_description(func.description)
                .with_signature(func.signature)
                .with_needs_parens(func.needs_parens),
        );
    }

    // Additional functions not in metadata (special syntax, keywords, etc.)
    builtins.extend(vec![
        // I/O and formatting (format strings, not functions)
        Suggestion::new("@json", SuggestionType::Function)
            .with_description("Format as JSON string"),
        Suggestion::new("@uri", SuggestionType::Function).with_description("URL encode"),
        Suggestion::new("@csv", SuggestionType::Function).with_description("Format as CSV"),
        Suggestion::new("@tsv", SuggestionType::Function).with_description("Format as TSV"),
        Suggestion::new("@html", SuggestionType::Function).with_description("HTML encode"),
        Suggestion::new("@base64", SuggestionType::Function).with_description("Base64 encode"),
        Suggestion::new("@base64d", SuggestionType::Function).with_description("Base64 decode"),
    ]);

    // Date functions not in requirements list
    builtins.extend(vec![
        Suggestion::new("fromdateiso8601", SuggestionType::Function)
            .with_description("Parse ISO8601 date"),
        Suggestion::new("todateiso8601", SuggestionType::Function)
            .with_description("Format as ISO8601 date"),
    ]);

    // Conditional/logic keywords
    builtins.extend(vec![
        Suggestion::new("if", SuggestionType::Function)
            .with_description("Conditional expression"),
        Suggestion::new("then", SuggestionType::Function).with_description("Then clause"),
        Suggestion::new("else", SuggestionType::Function).with_description("Else clause"),
        Suggestion::new("elif", SuggestionType::Function).with_description("Else-if clause"),
        Suggestion::new("end", SuggestionType::Function).with_description("End block"),
    ]);

    // Special functions
    builtins.extend(vec![
        Suggestion::new("in", SuggestionType::Function)
            .with_description("Check if value is in object"),
        Suggestion::new("as", SuggestionType::Function)
            .with_description("Bind variable"),
        Suggestion::new("repeat", SuggestionType::Function)
            .with_description("Repeat expression infinitely"),
        Suggestion::new("$ENV", SuggestionType::Function)
            .with_description("Environment object"),
    ]);

    // Assignment/update operators
    builtins.extend(vec![
        Suggestion::new("|=", SuggestionType::Operator)
            .with_description("Update assignment"),
        Suggestion::new("+=", SuggestionType::Operator)
            .with_description("Addition assignment"),
        Suggestion::new("-=", SuggestionType::Operator)
            .with_description("Subtraction assignment"),
        Suggestion::new("*=", SuggestionType::Operator)
            .with_description("Multiplication assignment"),
        Suggestion::new("/=", SuggestionType::Operator)
            .with_description("Division assignment"),
        Suggestion::new("//=", SuggestionType::Operator)
            .with_description("Alternative assignment"),
    ]);

    builtins
});

/// Filter jq builtins by prefix (optimized for performance)
pub fn filter_builtins(prefix: &str) -> Vec<Suggestion> {
    if prefix.is_empty() {
        return Vec::new();
    }

    let prefix_lower = prefix.to_lowercase();
    JQ_BUILTINS
        .iter()
        .filter(|s| s.text.to_lowercase().starts_with(&prefix_lower))
        .cloned()
        .collect()
}

/// Get all jq function metadata for testing
#[cfg(test)]
pub fn get_all_function_metadata() -> &'static [JqFunction] {
    JQ_FUNCTION_METADATA
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // **Feature: enhanced-autocomplete, Property 6: All functions have complete metadata**
    // *For any* function in the jq builtins list, the function SHALL have both
    // a `signature` field and a `needs_parens` field defined.
    // **Validates: Requirements 3.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_all_functions_have_complete_metadata(index in 0usize..JQ_FUNCTION_METADATA.len().max(1)) {
            // Skip test if metadata list is empty (will be populated in task 2)
            if JQ_FUNCTION_METADATA.is_empty() {
                return Ok(());
            }

            let func = &JQ_FUNCTION_METADATA[index % JQ_FUNCTION_METADATA.len()];

            // Verify name is not empty
            prop_assert!(!func.name.is_empty(), "Function name should not be empty");

            // Verify signature is not empty
            prop_assert!(!func.signature.is_empty(), "Function {} should have a non-empty signature", func.name);

            // Verify description is not empty
            prop_assert!(!func.description.is_empty(), "Function {} should have a non-empty description", func.name);

            // Verify signature contains the function name
            prop_assert!(
                func.signature.starts_with(func.name),
                "Function {} signature '{}' should start with the function name",
                func.name,
                func.signature
            );
        }
    }

    #[test]
    fn test_metadata_list_not_empty() {
        let metadata = get_all_function_metadata();
        assert!(!metadata.is_empty(), "JQ_FUNCTION_METADATA should not be empty");
    }

    #[test]
    fn test_functions_requiring_args_have_needs_parens_true() {
        // Verify specific functions from requirements 3.2 have needs_parens = true
        let functions_requiring_args = [
            "map", "select", "sort_by", "group_by", "unique_by", "min_by", "max_by",
            "has", "contains", "test", "match", "split", "join", "sub", "gsub",
            "with_entries", "recurse", "walk", "until", "while", "limit", "nth", "range",
            "getpath", "setpath", "delpaths", "del", "ltrimstr", "rtrimstr",
            "startswith", "endswith", "inside", "index", "rindex", "indices",
            "capture", "scan", "splits", "strftime", "strptime",
        ];

        for name in functions_requiring_args {
            let func = JQ_FUNCTION_METADATA.iter().find(|f| f.name == name);
            assert!(
                func.is_some(),
                "Function '{}' should be in JQ_FUNCTION_METADATA",
                name
            );
            assert!(
                func.unwrap().needs_parens,
                "Function '{}' should have needs_parens = true",
                name
            );
        }
    }

    #[test]
    fn test_functions_not_requiring_args_have_needs_parens_false() {
        // Verify specific functions from requirements 3.3 have needs_parens = false
        let functions_not_requiring_args = [
            "keys", "keys_unsorted", "values", "sort", "reverse", "unique", "flatten",
            "add", "length", "first", "last", "min", "max", "type", "tostring", "tonumber",
            "floor", "ceil", "round", "sqrt", "abs", "now", "empty", "error", "not",
            "ascii_downcase", "ascii_upcase", "arrays", "objects", "iterables", "booleans",
            "numbers", "strings", "nulls", "scalars", "to_entries", "from_entries",
            "paths", "leaf_paths", "transpose", "env", "fromdate", "todate",
        ];

        for name in functions_not_requiring_args {
            let func = JQ_FUNCTION_METADATA.iter().find(|f| f.name == name);
            assert!(
                func.is_some(),
                "Function '{}' should be in JQ_FUNCTION_METADATA",
                name
            );
            assert!(
                !func.unwrap().needs_parens,
                "Function '{}' should have needs_parens = false",
                name
            );
        }
    }

    // Helper function to get functions requiring arguments
    fn get_functions_requiring_args() -> Vec<&'static JqFunction> {
        JQ_FUNCTION_METADATA
            .iter()
            .filter(|f| f.needs_parens)
            .collect()
    }

    // Helper function to get functions not requiring arguments
    fn get_functions_not_requiring_args() -> Vec<&'static JqFunction> {
        JQ_FUNCTION_METADATA
            .iter()
            .filter(|f| !f.needs_parens)
            .collect()
    }

    // **Feature: enhanced-autocomplete, Property 4: Signature format for argument functions**
    // *For any* jq function marked with `needs_parens = true`, the signature field SHALL
    // match the pattern `name(...)` where `name` is the function name and `...` represents
    // one or more parameter indicators.
    // **Validates: Requirements 2.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_signature_format_for_argument_functions(index in 0usize..100) {
            let funcs = get_functions_requiring_args();
            if funcs.is_empty() {
                return Ok(());
            }

            let func = funcs[index % funcs.len()];

            // Verify signature starts with function name
            prop_assert!(
                func.signature.starts_with(func.name),
                "Function {} signature '{}' should start with the function name",
                func.name,
                func.signature
            );

            // Verify signature contains opening parenthesis after the name
            let after_name = &func.signature[func.name.len()..];
            prop_assert!(
                after_name.starts_with('('),
                "Function {} signature '{}' should have '(' immediately after the name",
                func.name,
                func.signature
            );

            // Verify signature contains closing parenthesis
            prop_assert!(
                func.signature.ends_with(')'),
                "Function {} signature '{}' should end with ')'",
                func.name,
                func.signature
            );

            // Verify there's content between parentheses (parameter indicators)
            let paren_start = func.signature.find('(').unwrap();
            let paren_end = func.signature.rfind(')').unwrap();
            prop_assert!(
                paren_end > paren_start + 1,
                "Function {} signature '{}' should have parameter indicators between parentheses",
                func.name,
                func.signature
            );
        }
    }

    // **Feature: enhanced-autocomplete, Property 5: Signature format for no-argument functions**
    // *For any* jq function marked with `needs_parens = false`, the signature field SHALL
    // equal the function name without any parentheses.
    // **Validates: Requirements 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_signature_format_for_no_argument_functions(index in 0usize..100) {
            let funcs = get_functions_not_requiring_args();
            if funcs.is_empty() {
                return Ok(());
            }

            let func = funcs[index % funcs.len()];

            // Verify signature equals the function name exactly
            prop_assert!(
                func.signature == func.name,
                "Function {} with needs_parens=false should have signature equal to name, but got '{}'",
                func.name,
                func.signature
            );

            // Verify signature does not contain parentheses
            prop_assert!(
                !func.signature.contains('('),
                "Function {} signature '{}' should not contain '('",
                func.name,
                func.signature
            );

            prop_assert!(
                !func.signature.contains(')'),
                "Function {} signature '{}' should not contain ')'",
                func.name,
                func.signature
            );
        }
    }
}
