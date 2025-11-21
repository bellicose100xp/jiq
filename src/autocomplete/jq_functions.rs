use super::state::{Suggestion, SuggestionType};
use std::sync::LazyLock;

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
        Suggestion::new("not", SuggestionType::Operator).with_description("Logical NOT"),
    ]);

    // Array functions
    builtins.extend(vec![
        Suggestion::new("map", SuggestionType::Function)
            .with_description("Apply expression to each element"),
        Suggestion::new("select", SuggestionType::Function)
            .with_description("Filter elements by condition"),
        Suggestion::new("sort", SuggestionType::Function).with_description("Sort array"),
        Suggestion::new("sort_by", SuggestionType::Function)
            .with_description("Sort array by expression"),
        Suggestion::new("reverse", SuggestionType::Function).with_description("Reverse array"),
        Suggestion::new("unique", SuggestionType::Function)
            .with_description("Remove duplicate values"),
        Suggestion::new("unique_by", SuggestionType::Function)
            .with_description("Remove duplicates by expression"),
        Suggestion::new("group_by", SuggestionType::Function)
            .with_description("Group array elements by expression"),
        Suggestion::new("flatten", SuggestionType::Function)
            .with_description("Flatten nested arrays"),
        Suggestion::new("add", SuggestionType::Function)
            .with_description("Sum array elements or concatenate"),
        Suggestion::new("length", SuggestionType::Function)
            .with_description("Length of array/object/string"),
        Suggestion::new("first", SuggestionType::Function).with_description("First element"),
        Suggestion::new("last", SuggestionType::Function).with_description("Last element"),
        Suggestion::new("nth", SuggestionType::Function).with_description("Nth element"),
        Suggestion::new("indices", SuggestionType::Function)
            .with_description("Find all indices of value"),
        Suggestion::new("index", SuggestionType::Function)
            .with_description("Find first index of value"),
        Suggestion::new("rindex", SuggestionType::Function)
            .with_description("Find last index of value"),
        Suggestion::new("inside", SuggestionType::Function)
            .with_description("Check if element is inside array"),
        Suggestion::new("contains", SuggestionType::Function)
            .with_description("Check if contains value"),
        Suggestion::new("startswith", SuggestionType::Function)
            .with_description("Check if starts with value"),
        Suggestion::new("endswith", SuggestionType::Function)
            .with_description("Check if ends with value"),
        Suggestion::new("limit", SuggestionType::Function).with_description("Limit output count"),
        Suggestion::new("range", SuggestionType::Function).with_description("Generate range"),
        Suggestion::new("min", SuggestionType::Function).with_description("Minimum value"),
        Suggestion::new("max", SuggestionType::Function).with_description("Maximum value"),
        Suggestion::new("min_by", SuggestionType::Function)
            .with_description("Minimum by expression"),
        Suggestion::new("max_by", SuggestionType::Function)
            .with_description("Maximum by expression"),
    ]);

    // Object functions
    builtins.extend(vec![
        Suggestion::new("keys", SuggestionType::Function)
            .with_description("Get object keys or array indices"),
        Suggestion::new("keys_unsorted", SuggestionType::Function)
            .with_description("Get object keys (unsorted)"),
        Suggestion::new("values", SuggestionType::Function).with_description("Get all values"),
        Suggestion::new("to_entries", SuggestionType::Function)
            .with_description("Convert object to key-value pairs"),
        Suggestion::new("from_entries", SuggestionType::Function)
            .with_description("Convert key-value pairs to object"),
        Suggestion::new("with_entries", SuggestionType::Function)
            .with_description("Transform object entries"),
        Suggestion::new("has", SuggestionType::Function).with_description("Check if key exists"),
        Suggestion::new("in", SuggestionType::Function)
            .with_description("Check if value is in object"),
        Suggestion::new("del", SuggestionType::Function).with_description("Delete key/path"),
        Suggestion::new("getpath", SuggestionType::Function).with_description("Get value at path"),
        Suggestion::new("setpath", SuggestionType::Function).with_description("Set value at path"),
        Suggestion::new("delpaths", SuggestionType::Function)
            .with_description("Delete multiple paths"),
        Suggestion::new("paths", SuggestionType::Function)
            .with_description("Get all paths (leaf paths)"),
        Suggestion::new("leaf_paths", SuggestionType::Function)
            .with_description("Get all leaf paths"),
    ]);

    // String functions
    builtins.extend(vec![
        Suggestion::new("tostring", SuggestionType::Function)
            .with_description("Convert to string"),
        Suggestion::new("tonumber", SuggestionType::Function)
            .with_description("Convert to number"),
        Suggestion::new("split", SuggestionType::Function)
            .with_description("Split string by delimiter"),
        Suggestion::new("join", SuggestionType::Function)
            .with_description("Join array with delimiter"),
        Suggestion::new("ltrimstr", SuggestionType::Function)
            .with_description("Remove prefix string"),
        Suggestion::new("rtrimstr", SuggestionType::Function)
            .with_description("Remove suffix string"),
        Suggestion::new("ascii_downcase", SuggestionType::Function)
            .with_description("Convert to lowercase"),
        Suggestion::new("ascii_upcase", SuggestionType::Function)
            .with_description("Convert to uppercase"),
        Suggestion::new("test", SuggestionType::Function)
            .with_description("Test regex match"),
        Suggestion::new("match", SuggestionType::Function).with_description("Match regex"),
        Suggestion::new("capture", SuggestionType::Function)
            .with_description("Capture regex groups"),
        Suggestion::new("scan", SuggestionType::Function)
            .with_description("Scan for all regex matches"),
        Suggestion::new("splits", SuggestionType::Function)
            .with_description("Split by regex"),
        Suggestion::new("sub", SuggestionType::Function)
            .with_description("Replace first regex match"),
        Suggestion::new("gsub", SuggestionType::Function)
            .with_description("Replace all regex matches"),
    ]);

    // Type functions
    builtins.extend(vec![
        Suggestion::new("type", SuggestionType::Function).with_description("Get value type"),
        Suggestion::new("arrays", SuggestionType::Function).with_description("Select arrays"),
        Suggestion::new("objects", SuggestionType::Function).with_description("Select objects"),
        Suggestion::new("iterables", SuggestionType::Function)
            .with_description("Select arrays/objects"),
        Suggestion::new("booleans", SuggestionType::Function).with_description("Select booleans"),
        Suggestion::new("numbers", SuggestionType::Function).with_description("Select numbers"),
        Suggestion::new("strings", SuggestionType::Function).with_description("Select strings"),
        Suggestion::new("nulls", SuggestionType::Function).with_description("Select nulls"),
        Suggestion::new("values", SuggestionType::Function)
            .with_description("Select non-null values"),
        Suggestion::new("scalars", SuggestionType::Function)
            .with_description("Select non-iterable values"),
    ]);

    // Math functions
    builtins.extend(vec![
        Suggestion::new("floor", SuggestionType::Function).with_description("Round down"),
        Suggestion::new("ceil", SuggestionType::Function).with_description("Round up"),
        Suggestion::new("round", SuggestionType::Function).with_description("Round to nearest"),
        Suggestion::new("sqrt", SuggestionType::Function).with_description("Square root"),
        Suggestion::new("abs", SuggestionType::Function).with_description("Absolute value"),
    ]);

    // Date functions
    builtins.extend(vec![
        Suggestion::new("now", SuggestionType::Function)
            .with_description("Current Unix timestamp"),
        Suggestion::new("fromdateiso8601", SuggestionType::Function)
            .with_description("Parse ISO8601 date"),
        Suggestion::new("todateiso8601", SuggestionType::Function)
            .with_description("Format as ISO8601 date"),
        Suggestion::new("fromdate", SuggestionType::Function)
            .with_description("Parse date string"),
        Suggestion::new("todate", SuggestionType::Function).with_description("Format date"),
        Suggestion::new("strftime", SuggestionType::Function)
            .with_description("Format timestamp"),
        Suggestion::new("strptime", SuggestionType::Function).with_description("Parse timestamp"),
    ]);

    // I/O and formatting
    builtins.extend(vec![
        Suggestion::new("@json", SuggestionType::Function)
            .with_description("Format as JSON string"),
        Suggestion::new("@uri", SuggestionType::Function).with_description("URL encode"),
        Suggestion::new("@csv", SuggestionType::Function).with_description("Format as CSV"),
        Suggestion::new("@tsv", SuggestionType::Function).with_description("Format as TSV"),
        Suggestion::new("@html", SuggestionType::Function).with_description("HTML encode"),
        Suggestion::new("@base64", SuggestionType::Function).with_description("Base64 encode"),
        Suggestion::new("@base64d", SuggestionType::Function).with_description("Base64 decode"),
    ]);

    // Advanced functions
    builtins.extend(vec![
        Suggestion::new("recurse", SuggestionType::Function)
            .with_description("Apply recursively"),
        Suggestion::new("walk", SuggestionType::Function)
            .with_description("Apply to all values recursively"),
        Suggestion::new("transpose", SuggestionType::Function)
            .with_description("Transpose matrix"),
        Suggestion::new("until", SuggestionType::Function)
            .with_description("Repeat until condition"),
        Suggestion::new("while", SuggestionType::Function)
            .with_description("Repeat while condition"),
        Suggestion::new("repeat", SuggestionType::Function)
            .with_description("Repeat expression infinitely"),
        Suggestion::new("env", SuggestionType::Function)
            .with_description("Access environment variables"),
        Suggestion::new("$ENV", SuggestionType::Function)
            .with_description("Environment object"),
        Suggestion::new("error", SuggestionType::Function).with_description("Raise error"),
        Suggestion::new("empty", SuggestionType::Function).with_description("Produce no output"),
    ]);

    // Conditional/logic
    builtins.extend(vec![
        Suggestion::new("if", SuggestionType::Function)
            .with_description("Conditional expression"),
        Suggestion::new("then", SuggestionType::Function).with_description("Then clause"),
        Suggestion::new("else", SuggestionType::Function).with_description("Else clause"),
        Suggestion::new("elif", SuggestionType::Function).with_description("Else-if clause"),
        Suggestion::new("end", SuggestionType::Function).with_description("End block"),
    ]);

    // Assignment/update
    builtins.extend(vec![
        Suggestion::new("as", SuggestionType::Function)
            .with_description("Bind variable"),
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
