//! Tests for jq error enhancement.
//!
//! The raw strings below are captured verbatim from real jq binaries:
//! jq 1.6, jq 1.7.1, and jq 1.8.1. Syntax errors differ across these
//! releases; runtime errors are identical. Each case asserts the enhanced
//! output is stable regardless of which jq produced it.

use super::*;

/// Raw stderr for the same mistake across the three supported jq lines.
struct VersionedRaw {
    v16: &'static str,
    v17: &'static str,
    v18: &'static str,
}

fn assert_all_versions<F: Fn(&EnhancedError)>(query: &str, raw: &VersionedRaw, check: F) {
    for (label, raw) in [("1.6", raw.v16), ("1.7", raw.v17), ("1.8", raw.v18)] {
        let enhanced =
            enhance_jq_error(raw, query).unwrap_or_else(|| panic!("{label}: expected enhancement"));
        check(&enhanced);
    }
}

// --------------------------------------------------------------------------
// Syntax: incomplete query (unclosed bracket)
// --------------------------------------------------------------------------

#[test]
fn unclosed_bracket_all_versions() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n.foo[    \njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected end of file (Unix shell quoting issues?) at <top-level>, line 1:\n.foo[    \njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected end of file at <top-level>, line 1, column 5:\n    .foo[\n        ^\njq: 1 compile error",
    };
    assert_all_versions(".foo[", &raw, |e| {
        assert!(
            e.summary.contains("Incomplete query"),
            "summary was: {}",
            e.summary
        );
        assert_eq!(
            e.hint.as_deref(),
            Some("Close the '['; e.g. .foo[0] or .foo[].")
        );
        // Misleading shell hint must never survive.
        assert!(!e.plain().contains("shell quoting"));
    });
}

#[test]
fn unclosed_paren_hint() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n(.a \njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected end of file (Unix shell quoting issues?) at <top-level>, line 1:\n(.a \njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected end of file, expecting '|' or ',' or ')' at <top-level>, line 1, column 2:\n    (.a\n     ^^\njq: 1 compile error",
    };
    assert_all_versions("(.a", &raw, |e| {
        assert!(e.summary.contains("Incomplete query"));
        assert_eq!(e.hint.as_deref(), Some("Add the matching ')'."));
    });
}

#[test]
fn unclosed_brace_hint() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n{a:  \njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected end of file (Unix shell quoting issues?) at <top-level>, line 1:\n{a:  \njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected end of file at <top-level>, line 1, column 3:\n    {a:\n      ^\njq: 1 compile error",
    };
    assert_all_versions("{a:", &raw, |e| {
        assert_eq!(e.hint.as_deref(), Some("Add the matching '}'."));
    });
}

#[test]
fn trailing_pipe_hint() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n.foo |     \njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected end of file (Unix shell quoting issues?) at <top-level>, line 1:\n.foo |     \njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected end of file at <top-level>, line 1, column 6:\n    .foo |\n         ^\njq: 1 compile error",
    };
    assert_all_versions(".foo |", &raw, |e| {
        assert!(e.summary.contains("Incomplete query"));
        assert!(e.hint.as_deref().unwrap().contains("trailing '|'"));
    });
}

#[test]
fn trailing_comma_hint() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n.a,  \njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected end of file (Unix shell quoting issues?) at <top-level>, line 1:\n.a,  \njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected end of file at <top-level>, line 1, column 3:\n    .a,\n      ^\njq: 1 compile error",
    };
    assert_all_versions(".a,", &raw, |e| {
        assert!(e.hint.as_deref().unwrap().contains("trailing ','"));
    });
}

// --------------------------------------------------------------------------
// Syntax: leading pipe
// --------------------------------------------------------------------------

#[test]
fn leading_pipe_all_versions() {
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected '|', expecting $end (Unix shell quoting issues?) at <top-level>, line 1:\n| .foo\njq: 1 compile error",
        v17: "jq: error: syntax error, unexpected '|', expecting end of file (Unix shell quoting issues?) at <top-level>, line 1:\n| .foo\njq: 1 compile error",
        v18: "jq: error: syntax error, unexpected '|', expecting end of file at <top-level>, line 1, column 1:\n    | .foo\n    ^\njq: 1 compile error",
    };
    assert_all_versions("| .foo", &raw, |e| {
        assert!(e.summary.contains("can't start the query"));
        assert!(e.hint.as_deref().unwrap().contains("leading '|'"));
    });
}

// --------------------------------------------------------------------------
// Syntax: location parsing (1.8 has column, 1.6/1.7 don't)
// --------------------------------------------------------------------------

#[test]
fn location_includes_column_only_when_present() {
    let with_col = "jq: error: syntax error, unexpected end of file at <top-level>, line 1, column 5:\n    .foo[\n        ^\njq: 1 compile error";
    let no_col = "jq: error: syntax error, unexpected $end (Unix shell quoting issues?) at <top-level>, line 1:\n.foo[    \njq: 1 compile error";

    let e18 = enhance_jq_error(with_col, ".foo[").unwrap();
    assert_eq!(e18.location.as_deref(), Some("line 1, column 5"));

    let e16 = enhance_jq_error(no_col, ".foo[").unwrap();
    assert_eq!(e16.location.as_deref(), Some("line 1"));
}

// --------------------------------------------------------------------------
// Compile: function / variable not defined
// --------------------------------------------------------------------------

#[test]
fn unknown_function_with_did_you_mean() {
    let raw = VersionedRaw {
        v16: "jq: error: lengths/0 is not defined at <top-level>, line 1:\nlengths\njq: 1 compile error",
        v17: "jq: error: lengths/0 is not defined at <top-level>, line 1:\nlengths\njq: 1 compile error",
        v18: "jq: error: lengths/0 is not defined at <top-level>, line 1, column 1:\n    lengths\n    ^^^^^^^\njq: 1 compile error",
    };
    assert_all_versions("lengths", &raw, |e| {
        assert_eq!(e.summary, "Unknown function `lengths`.");
        assert_eq!(e.hint.as_deref(), Some("Did you mean `length`?"));
    });
}

#[test]
fn unknown_function_no_close_match() {
    let raw = "jq: error: zzzzzzzz/0 is not defined at <top-level>, line 1, column 1:\n    zzzzzzzz\n    ^^^^^^^^\njq: 1 compile error";
    let e = enhance_jq_error(raw, "zzzzzzzz").unwrap();
    assert_eq!(e.summary, "Unknown function `zzzzzzzz`.");
    assert!(e.hint.as_deref().unwrap().contains("Check the spelling"));
}

#[test]
fn unknown_variable() {
    let raw = VersionedRaw {
        v16: "jq: error: $foo is not defined at <top-level>, line 1:\n$foo\njq: 1 compile error",
        v17: "jq: error: $foo is not defined at <top-level>, line 1:\n$foo\njq: 1 compile error",
        v18: "jq: error: $foo is not defined at <top-level>, line 1, column 1:\n    $foo\n    ^^^^\njq: 1 compile error",
    };
    assert_all_versions("$foo", &raw, |e| {
        assert_eq!(e.summary, "Unknown variable `$foo`.");
        assert!(e.hint.as_deref().unwrap().contains("as $name"));
    });
}

// --------------------------------------------------------------------------
// Compile: unusable field name (hyphen, non-ASCII, @)
// --------------------------------------------------------------------------

#[test]
fn hyphen_field_becomes_bracket_advice() {
    // jq parses `.my-field` as `.my - field`, so `field/0 is not defined`.
    let raw = VersionedRaw {
        v16: "jq: error: field/0 is not defined at <top-level>, line 1:\n.my-field    \njq: 1 compile error",
        v17: "jq: error: field/0 is not defined at <top-level>, line 1:\n.my-field    \njq: 1 compile error",
        v18: "jq: error: field/0 is not defined at <top-level>, line 1, column 5:\n    .my-field\n        ^^^^^\njq: 1 compile error",
    };
    assert_all_versions(".my-field", &raw, |e| {
        assert!(
            e.summary
                .contains("characters the .field shorthand can't express")
        );
        assert!(e.hint.as_deref().unwrap().contains("bracket notation"));
    });
}

#[test]
fn non_ascii_field_invalid_character() {
    // Two compile errors; we key off INVALID_CHARACTER / the try-advisory.
    let raw = VersionedRaw {
        v16: "jq: error: syntax error, unexpected INVALID_CHARACTER (Unix shell quoting issues?) at <top-level>, line 1:\n.名前 \njq: error: try .[\"field\"] instead of .field for unusually named fields at <top-level>, line 1:\n.名前\njq: 2 compile errors",
        v17: "jq: error: syntax error, unexpected INVALID_CHARACTER (Unix shell quoting issues?) at <top-level>, line 1:\n.名前 \njq: error: try .[\"field\"] instead of .field for unusually named fields at <top-level>, line 1:\n.名前\njq: 2 compile errors",
        v18: "jq: error: syntax error, unexpected INVALID_CHARACTER at <top-level>, line 1, column 2:\n    .名前\n     ^\njq: error: try .[\"field\"] instead of .field for unusually named fields at <top-level>, line 1, column 1:\n    .名前\n    ^^\njq: 2 compile errors",
    };
    assert_all_versions(".名前", &raw, |e| {
        assert!(
            e.summary
                .contains("characters the .field shorthand can't express")
        );
        assert!(e.hint.as_deref().unwrap().contains("bracket notation"));
    });
}

// --------------------------------------------------------------------------
// Runtime: indexing
// --------------------------------------------------------------------------

#[test]
fn index_array_with_string() {
    let raw = "jq: error (at <stdin>:1): Cannot index array with string \"foo\"";
    let e = enhance_jq_error(raw, ".foo").unwrap();
    assert!(e.summary.contains("Can't index an array with a field name"));
    assert!(e.hint.as_deref().unwrap().contains(".[0]"));
    // Runtime errors carry no compile location.
    assert_eq!(e.location, None);
}

#[test]
fn index_object_with_number() {
    let raw = "jq: error (at <stdin>:1): Cannot index object with number";
    let e = enhance_jq_error(raw, ".[0]").unwrap();
    assert!(e.summary.contains("Can't index an object with a number"));
}

#[test]
fn index_scalar() {
    let raw = "jq: error (at <stdin>:1): Cannot index number with string \"a\"";
    let e = enhance_jq_error(raw, ".a").unwrap();
    assert!(e.summary.contains("Can't index a number value"));
}

// --------------------------------------------------------------------------
// Runtime: iteration
// --------------------------------------------------------------------------

#[test]
fn iterate_over_number() {
    let raw = "jq: error (at <stdin>:1): Cannot iterate over number (5)";
    let e = enhance_jq_error(raw, ".[]").unwrap();
    assert!(e.summary.contains("Can't iterate over a number"));
    assert!(e.hint.as_deref().unwrap().contains(".[]"));
}

#[test]
fn iterate_over_string() {
    let raw = "jq: error (at <stdin>:1): Cannot iterate over string (\"hi\")";
    let e = enhance_jq_error(raw, ".[]").unwrap();
    assert!(e.summary.contains("Can't iterate over a string"));
}

// --------------------------------------------------------------------------
// Runtime: arithmetic type mismatch
// --------------------------------------------------------------------------

#[test]
fn arithmetic_add_mismatch() {
    let raw = "jq: error (at <stdin>:1): number (1) and string (\"a\") cannot be added";
    let e = enhance_jq_error(raw, "1 + \"a\"").unwrap();
    assert!(
        e.summary
            .contains("a number and a string can't be added together")
    );
    assert!(e.summary.starts_with("Type mismatch:"));
    assert!(e.hint.is_some());
}

#[test]
fn arithmetic_subtract_mismatch() {
    let raw = "jq: error (at <stdin>:1): object ({}) and number (1) cannot be subtracted";
    let e = enhance_jq_error(raw, "{} - 1").unwrap();
    assert!(
        e.summary
            .contains("an object and a number can't be subtracted")
    );
    assert!(e.hint.is_some());
}

#[test]
fn division_by_zero_runtime() {
    let raw = "jq: error (at <stdin>:1): number (1) and number (0) cannot be divided because the divisor is zero";
    let e = enhance_jq_error(raw, "1/0").unwrap();
    assert_eq!(e.summary, "Division by zero.");
}

#[test]
fn division_by_zero_compile_16() {
    // jq 1.6 reports `1/0` at compile time.
    let raw = "jq: error: Division by zero? at <top-level>, line 1:\n1/0\njq: 1 compile error";
    let e = enhance_jq_error(raw, "1/0").unwrap();
    assert_eq!(e.summary, "Division by zero.");
}

// --------------------------------------------------------------------------
// Runtime: has no length / keys
// --------------------------------------------------------------------------

#[test]
fn boolean_has_no_length() {
    let raw = "jq: error (at <stdin>:1): boolean (true) has no length";
    let e = enhance_jq_error(raw, "length").unwrap();
    assert!(e.summary.contains("`length` isn't defined for boolean"));
}

#[test]
fn string_has_no_keys() {
    let raw = "jq: error (at <stdin>:1): string (\"hi\") has no keys";
    let e = enhance_jq_error(raw, "keys").unwrap();
    assert!(e.summary.contains("`keys` isn't defined for string"));
}

// --------------------------------------------------------------------------
// Runtime: object key, tonumber, error()
// --------------------------------------------------------------------------

#[test]
fn null_object_key() {
    let raw = "jq: error (at <stdin>:1): Cannot use null (null) as object key";
    let e = enhance_jq_error(raw, "{(.a):1}").unwrap();
    assert!(e.summary.contains("Object keys must be strings"));
    assert!(e.summary.contains("null"));
}

#[test]
fn tonumber_invalid_18() {
    let raw = "jq: error (at <stdin>:1): string (\"abc\") cannot be parsed as a number";
    let e = enhance_jq_error(raw, "tonumber").unwrap();
    assert!(e.summary.contains("isn't a valid number"));
}

#[test]
fn tonumber_invalid_16() {
    // 1.6/1.7 phrasing.
    let raw = "jq: error (at <stdin>:1): Invalid numeric literal at EOF at line 1, column 3 (while parsing 'abc')";
    let e = enhance_jq_error(raw, "tonumber").unwrap();
    assert!(e.summary.contains("isn't a valid number"));
}

#[test]
fn error_with_non_string() {
    let raw = "jq: error (at <stdin>:1) (not a string): null";
    let e = enhance_jq_error(raw, "error").unwrap();
    assert!(e.summary.contains("non-string value via `error`"));
}

#[test]
fn user_thrown_error_message_passes_through() {
    let raw = "jq: error (at <stdin>:1): boom";
    let e = enhance_jq_error(raw, "error(\"boom\")").unwrap();
    assert_eq!(e.summary, "Boom");
    assert_eq!(e.hint, None);
}

// --------------------------------------------------------------------------
// Fall-through: non-jq messages return None
// --------------------------------------------------------------------------

#[test]
fn non_jq_message_returns_none() {
    assert!(enhance_jq_error("Query worker disconnected", ".").is_none());
    assert!(enhance_jq_error("", ".").is_none());
    assert!(enhance_jq_error("   ", ".").is_none());
}

#[test]
fn unrecognized_runtime_error_still_enhances_cleanly() {
    // Unknown runtime wording: we still strip the `(at <stdin>:N)` noise.
    let raw = "jq: error (at <stdin>:1): some brand new error wording";
    let e = enhance_jq_error(raw, ".").unwrap();
    assert_eq!(e.summary, "Some brand new error wording");
    assert!(!e.plain().contains("<stdin>"));
}

#[test]
fn unrecognized_compile_error_strips_shell_hint() {
    let raw = "jq: error: syntax error, unexpected ';', expecting '|' or ',' or ':' or ']' (Unix shell quoting issues?) at <top-level>, line 1:\n.[1;2]\njq: 1 compile error";
    let e = enhance_jq_error(raw, ".[1;2]").unwrap();
    assert!(!e.plain().contains("shell quoting"));
    assert!(e.summary.contains("Unexpected"));
}

// --------------------------------------------------------------------------
// Levenshtein / closest_builtin unit coverage
// --------------------------------------------------------------------------

#[test]
fn levenshtein_basic() {
    assert_eq!(levenshtein("", ""), 0);
    assert_eq!(levenshtein("abc", "abc"), 0);
    assert_eq!(levenshtein("abc", "abd"), 1);
    assert_eq!(levenshtein("kitten", "sitting"), 3);
    assert_eq!(levenshtein("", "abc"), 3);
}

#[test]
fn closest_builtin_finds_typos() {
    assert_eq!(closest_builtin("lengths"), Some("length"));
    assert_eq!(closest_builtin("slect"), Some("select"));
    assert_eq!(closest_builtin("mapp"), Some("map"));
}

#[test]
fn closest_builtin_rejects_wild_guesses() {
    assert_eq!(closest_builtin("z"), None);
    assert_eq!(closest_builtin("qwertyuiop"), None);
}

#[test]
fn plain_renders_summary_hint_and_location() {
    let e = EnhancedError::new("Summary here.")
        .with_hint("do the thing")
        .with_location(Some("line 1, column 5".to_string()));
    let plain = e.plain();
    assert!(plain.contains("Summary here."));
    assert!(plain.contains("Try: do the thing"));
    assert!(plain.contains("jq: line 1, column 5"));
}
