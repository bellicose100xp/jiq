//! jq error enhancement
//!
//! jq's raw stderr is famously terse and varies between releases. This module
//! turns that stderr into a short, plain-language explanation plus a concrete
//! fix hint for the error overlay.
//!
//! It is version-tolerant by design and supports jq 1.6 through 1.8+. The three
//! releases phrase syntax errors differently:
//!   - 1.6:  `syntax error, unexpected $end (Unix shell quoting issues?) ... line 1:`
//!   - 1.7:  `syntax error, unexpected end of file (Unix shell quoting issues?) ... line 1:`
//!   - 1.8:  `syntax error, unexpected end of file ... line 1, column 5:` (+ caret)
//!
//! Runtime (type/index) errors are byte-identical across all three, so they are
//! matched by their stable wording.
//!
//! The `(Unix shell quoting issues?)` hint that 1.6/1.7 append is actively
//! misleading inside jiq: the query is passed to jq as a single argv entry, not
//! through a shell, so quoting is never the cause. It is always stripped.
//!
//! Only the human-facing overlay is enhanced. The AI assistant still receives
//! jq's raw stderr, which models read fluently and benefit from.

/// A human-friendly rendering of a jq error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnhancedError {
    /// Plain-language explanation of what went wrong. May contain `\n`.
    pub summary: String,
    /// Optional concrete fix suggestion (rendered after a "Try: " label).
    pub hint: Option<String>,
    /// Optional source location ("line 1, column 5") when jq reported one.
    pub location: Option<String>,
}

impl EnhancedError {
    fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            hint: None,
            location: None,
        }
    }

    fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    fn with_location(mut self, location: Option<String>) -> Self {
        self.location = location;
        self
    }

    /// Flatten into a single display string. The renderer styles the summary,
    /// hint, and location separately; this is the unstyled equivalent used by
    /// tests to assert on the full rendered text.
    #[cfg(test)]
    pub fn plain(&self) -> String {
        let mut out = self.summary.clone();
        if let Some(hint) = &self.hint {
            out.push_str("\n\nTry: ");
            out.push_str(hint);
        }
        if let Some(loc) = &self.location {
            out.push_str("\n\njq: ");
            out.push_str(loc);
        }
        out
    }
}

/// Enhance a raw jq error message.
///
/// `query` is the filter the user typed; it sharpens syntax-error hints (e.g.
/// detecting which bracket is unclosed) without depending on jq's snippet
/// formatting, which differs across versions.
///
/// Returns `None` when the text is not a jq error at all (e.g. jiq's own
/// "Query worker disconnected"), so the caller can show it verbatim.
pub fn enhance_jq_error(raw: &str, query: &str) -> Option<EnhancedError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // Runtime errors: `jq: error (at <stdin>:N): <message>`
    if let Some(message) = runtime_message(raw) {
        return Some(enhance_runtime(&message));
    }

    // Compile/syntax errors: `jq: error: <core> at <top-level>, line N...:`
    if let Some(payload) = compile_payload(raw) {
        let (core, location) = split_location(payload);
        let core = strip_shell_hint(core);
        return Some(enhance_compile(&core, query).with_location(location));
    }

    // Not a recognizable jq error (likely a jiq-internal message). Leave as-is.
    None
}

// --------------------------------------------------------------------------
// Runtime errors
// --------------------------------------------------------------------------

/// Extract the message from the first `jq: error (at ...)` line, dropping the
/// `(at <stdin>:N)` location noise. Handles the `(not a string): null` variant
/// emitted by a bare `error`.
fn runtime_message(raw: &str) -> Option<String> {
    // A runtime error is always the first line of jq's stderr.
    let line = raw.lines().next()?.trim_start();
    let rest = line.strip_prefix("jq: error (at ")?;
    // rest = "<stdin>:1): Cannot index..." or "<stdin>:1) (not a string): null"
    let close = rest.find(')')?;
    let after = rest[close + 1..].trim_start();
    let message = match after.strip_prefix(':') {
        Some(m) => m.trim().to_string(),
        None => after.to_string(),
    };
    Some(message)
}

fn enhance_runtime(message: &str) -> EnhancedError {
    // `error` builtin invoked with a non-string value, e.g. `(not a string): null`.
    if message.starts_with("(not a string)") {
        return EnhancedError::new(
            "The query raised an error with a non-string value via `error`.",
        )
        .with_hint("Pass a string to error(), e.g. error(\"message\").");
    }

    if let Some(rest) = message.strip_prefix("Cannot index ") {
        return enhance_index(rest);
    }

    if let Some(rest) = message.strip_prefix("Cannot iterate over ") {
        let ty = leading_type(rest).unwrap_or("value");
        return EnhancedError::new(format!(
            "Can't iterate over a {ty}. The .[] iterator only works on arrays and objects."
        ))
        .with_hint("Use .[] on an array or object, or pick a field/index first.");
    }

    if message.starts_with("Cannot use ") && message.contains("as object key") {
        let ty = message
            .strip_prefix("Cannot use ")
            .and_then(leading_type)
            .unwrap_or("that value");
        return EnhancedError::new(format!(
            "Object keys must be strings, but the key here is a {ty}."
        ))
        .with_hint("Convert the key to a string, e.g. {(.id|tostring): .value}.");
    }

    if message.contains("cannot be divided") && message.contains("divisor is zero") {
        return EnhancedError::new("Division by zero.")
            .with_hint("Guard the divisor, e.g. (.a / .b) only when .b != 0.");
    }

    if let Some(arith) = enhance_arithmetic(message) {
        return arith;
    }

    if let Some((ty, what)) = parse_has_no(message) {
        return if what == "length" {
            EnhancedError::new(format!("`length` isn't defined for {ty} values."))
                .with_hint("length works on strings, arrays, objects, and numbers.")
        } else {
            EnhancedError::new(format!("`keys` isn't defined for {ty} values."))
                .with_hint("keys works on objects (and arrays, returning indices).")
        };
    }

    if message.contains("cannot be parsed as a number")
        || message.starts_with("Invalid numeric literal")
    {
        return EnhancedError::new("That string isn't a valid number, so `tonumber` failed.")
            .with_hint("Check the string contains only digits, e.g. \"42\" | tonumber.");
    }

    // Recognized-but-generic, or a user-thrown error message: drop the jq noise
    // (already done) and show the bare message.
    EnhancedError::new(capitalize_first(message))
}

/// Parse `Cannot index <T> with <KT>[ "<key>"]`.
fn enhance_index(rest: &str) -> EnhancedError {
    let container = leading_type(rest).unwrap_or("value");
    let with_key = rest.find(" with ").map(|i| &rest[i + 6..]).unwrap_or("");
    let key_is_string = with_key.starts_with("string");
    let key_is_number = with_key.starts_with("number");

    match (container, key_is_string, key_is_number) {
        ("array", true, _) => EnhancedError::new(
            "Can't index an array with a field name. Arrays are indexed by position.",
        )
        .with_hint("Use .[0] for the first item, or .[] to iterate every item."),
        ("object", _, true) => EnhancedError::new(
            "Can't index an object with a number. Objects are indexed by key name.",
        )
        .with_hint("Use .fieldName or .[\"field name\"]."),
        (ty, _, _) => EnhancedError::new(format!(
            "Can't index a {ty} value. Only objects and arrays can be indexed."
        ))
        .with_hint("Drill into an object (.field) or array (.[0]) first."),
    }
}

/// Parse arithmetic type mismatches like
/// `number (1) and string ("a") cannot be added`.
fn enhance_arithmetic(message: &str) -> Option<EnhancedError> {
    let idx = message.find(" cannot be ")?;
    let (operands, tail) = message.split_at(idx);
    let verb = tail
        .strip_prefix(" cannot be ")?
        .trim_end_matches('.')
        .to_string();
    let parts: Vec<&str> = operands.split(" and ").collect();
    if parts.len() != 2 {
        return None;
    }
    let a = leading_type(parts[0])?;
    let b = leading_type(parts[1])?;
    let action = match verb.as_str() {
        "added" => "added together",
        "subtracted" => "subtracted",
        "multiplied" => "multiplied",
        "divided" => "divided",
        other => other,
    };
    Some(
        EnhancedError::new(format!(
            "Type mismatch: {} {a} and {} {b} can't be {action}.",
            article(a),
            article(b)
        ))
        .with_hint("Convert one side first, e.g. tostring or tonumber, so the types match."),
    )
}

/// Parse `<T> (<v>) has no <what>` -> (type, what).
fn parse_has_no(message: &str) -> Option<(&str, &str)> {
    let idx = message.find(" has no ")?;
    let ty = leading_type(&message[..idx])?;
    let what = message[idx + 8..].trim_end_matches('.').trim();
    Some((ty, what))
}

// --------------------------------------------------------------------------
// Compile / syntax errors
// --------------------------------------------------------------------------

/// Return the text after the first `jq: error:` prefix (a compile error).
fn compile_payload(raw: &str) -> Option<&str> {
    for line in raw.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("jq: error:") {
            return Some(rest.trim());
        }
    }
    None
}

/// Split a compile payload into its core message and an optional location,
/// dropping the ` at <top-level>, line N[, column M]:` suffix.
fn split_location(payload: &str) -> (&str, Option<String>) {
    if let Some(idx) = payload.find(" at <top-level>") {
        let core = payload[..idx].trim();
        let location = parse_line_col(&payload[idx..]);
        (core, location)
    } else {
        (payload.trim_end_matches(':').trim(), None)
    }
}

/// Build "line N" / "line N, column M" from a `..., line 1, column 5:` suffix.
fn parse_line_col(suffix: &str) -> Option<String> {
    let line = extract_number_after(suffix, "line ")?;
    match extract_number_after(suffix, "column ") {
        Some(col) => Some(format!("line {line}, column {col}")),
        None => Some(format!("line {line}")),
    }
}

fn extract_number_after(haystack: &str, label: &str) -> Option<u32> {
    let start = haystack.find(label)? + label.len();
    let digits: String = haystack[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Remove jq 1.6/1.7's misleading "(Unix shell quoting issues?)" hint.
fn strip_shell_hint(core: &str) -> String {
    core.replace(" (Unix shell quoting issues?)", "")
        .trim()
        .to_string()
}

fn enhance_compile(core: &str, query: &str) -> EnhancedError {
    // Field name with characters the .field shorthand can't express
    // (non-ASCII, hyphen, @, space, ...). jq surfaces this as an
    // INVALID_CHARACTER syntax error, a `field/0 is not defined` error, or the
    // `try .["field"]` advisory, depending on the exact character.
    if core.contains("INVALID_CHARACTER")
        || core.starts_with("try .[\"field\"]")
        || core == "field/0 is not defined"
    {
        return EnhancedError::new(
            "A field name here uses characters the .field shorthand can't express \
             (spaces, hyphens, @, non-ASCII, or a leading digit).",
        )
        .with_hint("Use bracket notation instead, e.g. .[\"field-name\"].");
    }

    if let Some(name) = core.strip_suffix(" is not defined") {
        return enhance_not_defined(name.trim());
    }

    if core.starts_with("Division by zero") {
        return EnhancedError::new("Division by zero.")
            .with_hint("Guard the divisor, e.g. (.a / .b) only when .b != 0.");
    }

    if core.starts_with("syntax error") {
        return enhance_syntax(core, query);
    }

    // Unrecognized compile error: show the cleaned core (shell hint already
    // stripped), which is still better than the raw multi-line dump.
    EnhancedError::new(capitalize_first(core))
}

fn enhance_not_defined(name: &str) -> EnhancedError {
    if let Some(var) = name.strip_prefix('$') {
        return EnhancedError::new(format!("Unknown variable `${var}`."))
            .with_hint("Bind it first with `... as $name`, or check for a typo.");
    }

    // jq reports functions as `name/arity`.
    let base = name.split('/').next().unwrap_or(name);
    let mut err = EnhancedError::new(format!("Unknown function `{base}`."));
    if let Some(suggestion) = closest_builtin(base) {
        err = err.with_hint(format!("Did you mean `{suggestion}`?"));
    } else {
        err = err.with_hint("Check the spelling, or see `jq` builtins for the right name.");
    }
    err
}

fn enhance_syntax(core: &str, query: &str) -> EnhancedError {
    let unexpected = extract_quoted_field(core, "unexpected ");
    let is_eof = unexpected
        .as_deref()
        .is_some_and(|u| u == "end of file" || u == "$end");

    // A leading pipe is a common, specific mistake.
    if query.trim_start().starts_with('|') {
        return EnhancedError::new("A '|' can't start the query; it must sit between two filters.")
            .with_hint("Remove the leading '|', or add a filter before it.");
    }

    if is_eof {
        return enhance_incomplete(query);
    }

    let token = unexpected
        .as_deref()
        .map(humanize_token)
        .unwrap_or_else(|| "something".to_string());
    EnhancedError::new(format!("Unexpected {token} in the query."))
        .with_hint("Check the syntax around that point for a missing or stray operator.")
}

/// Build the message for a query that ends while jq still expects more input.
fn enhance_incomplete(query: &str) -> EnhancedError {
    let base =
        EnhancedError::new("Incomplete query: jq reached the end while still expecting more.");
    match unclosed_delimiter(query) {
        Some('[') => base.with_hint("Close the '['; e.g. .foo[0] or .foo[]."),
        Some('(') => base.with_hint("Add the matching ')'."),
        Some('{') => base.with_hint("Add the matching '}'."),
        Some('"') => base.with_hint("Add the closing '\"' to finish the string."),
        _ => {
            let trimmed = query.trim_end();
            if trimmed.ends_with('|') {
                base.with_hint(
                    "There's a trailing '|' with nothing after it; add a filter or remove it.",
                )
            } else if trimmed.ends_with(',') {
                base.with_hint("There's a trailing ',' with nothing after it.")
            } else {
                base.with_hint("Finish the expression, or remove the dangling operator at the end.")
            }
        }
    }
}

/// Scan `query` for the first unclosed bracket/brace/paren or an unterminated
/// string, ignoring delimiters inside string literals. Returns the opener.
fn unclosed_delimiter(query: &str) -> Option<char> {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in query.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => stack.push(c),
            ')' if stack.last() == Some(&'(') => {
                stack.pop();
            }
            ']' if stack.last() == Some(&'[') => {
                stack.pop();
            }
            '}' if stack.last() == Some(&'{') => {
                stack.pop();
            }
            _ => {}
        }
    }
    if in_string {
        return Some('"');
    }
    stack.first().copied()
}

/// Turn a jq grammar token into readable text.
fn humanize_token(token: &str) -> String {
    match token {
        "end of file" | "$end" => "the end of the query".to_string(),
        "INVALID_CHARACTER" => "an invalid character".to_string(),
        "IDENT" => "a name".to_string(),
        "LITERAL" | "STRING" | "QQSTRING_START" => "a string".to_string(),
        "NUMBER" => "a number".to_string(),
        "FORMAT" => "a @format".to_string(),
        t if t.starts_with("QQSTRING") => "a string interpolation".to_string(),
        // Quoted punctuation like '|', ';', ')': keep the symbol verbatim.
        t => t.to_string(),
    }
}

/// Extract the value after `label` up to the next comma (used for jq's
/// `unexpected X, expecting Y` form). Quoted symbols keep their quotes.
fn extract_quoted_field(core: &str, label: &str) -> Option<String> {
    let start = core.find(label)? + label.len();
    let rest = &core[start..];
    let end = rest.find(',').unwrap_or(rest.len());
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

/// The JSON type word at the start of a jq operand description, e.g.
/// `number (1)` -> "number", `string ("a")` -> "string".
fn leading_type(s: &str) -> Option<&'static str> {
    const TYPES: [&str; 6] = ["number", "string", "object", "array", "boolean", "null"];
    let word = s.trim_start();
    TYPES.into_iter().find(|t| word.starts_with(t))
}

/// "a" or "an" for the given jq type word ("object"/"array" take "an").
fn article(ty: &str) -> &'static str {
    match ty {
        "object" | "array" => "an",
        _ => "a",
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Closest jq builtin to `name` within a small edit distance, for
/// "did you mean" suggestions on unknown-function errors.
fn closest_builtin(name: &str) -> Option<&'static str> {
    if name.len() < 2 {
        return None;
    }
    // Allow more slack for longer names, but never a wild guess.
    let max_distance = if name.len() <= 4 { 1 } else { 2 };
    let mut best: Option<(&'static str, usize)> = None;
    for &builtin in JQ_BUILTINS {
        let d = levenshtein(name, builtin);
        if d == 0 {
            continue; // identical -> not a typo we can fix
        }
        if d <= max_distance && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((builtin, d));
        }
    }
    best.map(|(b, _)| b)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// jq builtin function names (union of 1.6 through 1.8). Used only for
/// "did you mean" suggestions, so a slightly stale entry is harmless.
const JQ_BUILTINS: &[&str] = &[
    "abs",
    "acos",
    "acosh",
    "add",
    "all",
    "any",
    "arrays",
    "ascii_downcase",
    "ascii_upcase",
    "asin",
    "asinh",
    "atan",
    "atanh",
    "booleans",
    "bsearch",
    "builtins",
    "capture",
    "cbrt",
    "ceil",
    "combinations",
    "contains",
    "cos",
    "cosh",
    "debug",
    "del",
    "delpaths",
    "empty",
    "endswith",
    "env",
    "error",
    "exp",
    "exp10",
    "exp2",
    "explode",
    "fabs",
    "finites",
    "first",
    "flatten",
    "floor",
    "from_entries",
    "fromdate",
    "fromdateiso8601",
    "fromjson",
    "fromstream",
    "getpath",
    "gmtime",
    "group_by",
    "gsub",
    "halt",
    "halt_error",
    "has",
    "implode",
    "index",
    "indices",
    "input",
    "input_filename",
    "input_line_number",
    "inputs",
    "inside",
    "isempty",
    "isfinite",
    "isinfinite",
    "isnan",
    "isnormal",
    "iterables",
    "join",
    "keys",
    "keys_unsorted",
    "last",
    "leaf_paths",
    "length",
    "limit",
    "localtime",
    "log",
    "log10",
    "log2",
    "ltrimstr",
    "map",
    "map_values",
    "match",
    "max",
    "max_by",
    "min",
    "min_by",
    "mktime",
    "modulemeta",
    "nan",
    "nearbyint",
    "normals",
    "not",
    "now",
    "nth",
    "nulls",
    "numbers",
    "objects",
    "path",
    "paths",
    "pick",
    "pow",
    "range",
    "recurse",
    "recurse_down",
    "repeat",
    "reverse",
    "rindex",
    "round",
    "rtrimstr",
    "scalars",
    "scan",
    "select",
    "setpath",
    "sin",
    "sinh",
    "skip",
    "sort",
    "sort_by",
    "split",
    "splits",
    "sqrt",
    "startswith",
    "stderr",
    "strftime",
    "strings",
    "strptime",
    "sub",
    "tan",
    "tanh",
    "test",
    "to_entries",
    "todate",
    "todateiso8601",
    "tojson",
    "tonumber",
    "tostream",
    "tostring",
    "transpose",
    "trim",
    "trimstr",
    "trunc",
    "truncate_stream",
    "type",
    "unique",
    "unique_by",
    "until",
    "utf8bytelength",
    "values",
    "walk",
    "while",
    "with_entries",
];

#[cfg(test)]
#[path = "error_enhance_tests.rs"]
mod error_enhance_tests;
