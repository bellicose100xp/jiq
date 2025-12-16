//! Function context insertion tests

use super::*;

#[test]
fn test_jq_keyword_autocomplete_no_dot_prefix() {
    // Test that jq keywords like "then", "else", "end" don't get a dot prefix
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
    let mut app = test_app(json);

    // Step 1: Type the beginning of an if statement
    app.input
        .textarea
        .insert_str(".services | if has(\"capacityProviderStrategy\")");
    app.query
        .execute(".services | if has(\"capacityProviderStrategy\")");

    // Step 2: Type partial "the" to trigger autocomplete for "then"
    app.input.textarea.insert_str(" the");

    // Step 3: Accept "then" from autocomplete
    // This should NOT add a dot before "then"
    insert_suggestion_from_app(&mut app, &test_suggestion("then"));

    // Should produce: .services | if has("capacityProviderStrategy") then
    // NOT: .services | if has("capacityProviderStrategy") .then
    assert_eq!(
        app.input.query(),
        ".services | if has(\"capacityProviderStrategy\") then"
    );

    // Verify no extra dot was added
    assert!(
        !app.input.query().contains(" .then"),
        "Should not have dot before 'then' keyword"
    );
}

#[test]
fn test_jq_keyword_else_autocomplete() {
    // Test "else" keyword autocomplete
    let json = r#"{"value": 42}"#;
    let mut app = test_app(json);

    // Type an if-then statement
    app.input
        .textarea
        .insert_str("if .value > 10 then \"high\" el");

    // Accept "else" from autocomplete
    insert_suggestion_from_app(&mut app, &test_suggestion("else"));

    // Should produce: if .value > 10 then "high" else
    // NOT: if .value > 10 then "high" .else
    assert_eq!(app.input.query(), "if .value > 10 then \"high\" else");
    assert!(
        !app.input.query().contains(".else"),
        "Should not have dot before 'else' keyword"
    );
}

#[test]
fn test_jq_keyword_end_autocomplete() {
    // Test "end" keyword autocomplete
    let json = r#"{"value": 42}"#;
    let mut app = test_app(json);

    // Type a complete if-then-else statement
    app.input
        .textarea
        .insert_str("if .value > 10 then \"high\" else \"low\" en");

    // Accept "end" from autocomplete
    insert_suggestion_from_app(&mut app, &test_suggestion("end"));

    // Should produce: if .value > 10 then "high" else "low" end
    // NOT: if .value > 10 then "high" else "low" .end
    assert_eq!(
        app.input.query(),
        "if .value > 10 then \"high\" else \"low\" end"
    );
    assert!(
        !app.input.query().contains(".end"),
        "Should not have dot before 'end' keyword"
    );
}
