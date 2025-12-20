//! Field context insertion tests

use super::*;

#[test]
fn test_array_suggestion_appends_to_path() {
    // When accepting [].field suggestion for .services, should produce .services[].field
    let json = r#"{"services": [{"name": "alice"}, {"name": "bob"}, {"name": "charlie"}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute ".services" to cache base
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    // Validate cached state after ".services"
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string()),
        "base_query should be '.services'"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects),
        "base_type should be ArrayOfObjects"
    );

    // Step 2: Accept autocomplete suggestion "[].name" (no leading dot since after NoOp)
    insert_suggestion_from_app(&mut app, &test_suggestion("[].name"));

    // Should produce .services[].name (append, not replace)
    assert_eq!(app.input.query(), ".services[].name");

    // CRITICAL: Verify the query EXECUTES correctly and returns ALL array elements
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("alice"), "Should contain first element");
    assert!(result.contains("bob"), "Should contain second element");
    assert!(result.contains("charlie"), "Should contain third element");

    // Verify it does NOT just return nulls or single value
    let line_count = result.lines().count();
    assert!(
        line_count >= 3,
        "Should return at least 3 lines for 3 array elements"
    );
}

#[test]
fn test_simple_path_continuation_with_dot() {
    // Test simple path continuation: .object.field
    // This is the bug: .services[0].deploymentConfiguration.alarms becomes deploymentConfigurationalarms
    let json = r#"{"user": {"name": "Alice", "age": 30, "address": {"city": "NYC"}}}"#;
    let mut app = test_app(json);

    // Step 1: Execute base query
    app.input.textarea.insert_str(".user");
    app.query.as_mut().unwrap().execute(".user");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".user".to_string())
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::Object)
    );

    // Step 2: Type ".na" (partial field access)
    app.input.textarea.insert_str(".na");

    // Step 3: Accept suggestion "name" (no leading dot since continuing path)
    insert_suggestion_from_app(&mut app, &test_suggestion("name"));

    // Should produce: .user.name
    // NOT: .username (missing dot)
    assert_eq!(app.input.query(), ".user.name");

    // Verify execution
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("Alice"));
}

#[test]
fn test_array_suggestion_replaces_partial_field() {
    // When user types partial field after array name, accepting [] suggestion should replace partial
    let json =
        r#"{"services": [{"serviceArn": "arn1"}, {"serviceArn": "arn2"}, {"serviceArn": "arn3"}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute ".services" to cache base
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string())
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects)
    );

    // Step 2: Type ".s" (partial)
    app.input.textarea.insert_char('.');
    app.input.textarea.insert_char('s');

    // Step 3: Accept autocomplete suggestion "[].serviceArn"
    insert_suggestion_from_app(&mut app, &test_suggestion("[].serviceArn"));

    // Should produce .services[].serviceArn (replace ".s" with "[].serviceArn")
    assert_eq!(app.input.query(), ".services[].serviceArn");

    // CRITICAL: Verify execution returns ALL serviceArns
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();

    assert!(result.contains("arn1"), "Should contain first serviceArn");
    assert!(result.contains("arn2"), "Should contain second serviceArn");
    assert!(result.contains("arn3"), "Should contain third serviceArn");

    // Should NOT have nulls (would indicate query failed to iterate array)
    let null_count = result.matches("null").count();
    assert_eq!(
        null_count, 0,
        "Should not have any null values - query should iterate all array elements"
    );
}

#[test]
fn test_array_suggestion_replaces_trailing_dot() {
    // When user types ".services." (trailing dot, no partial), array suggestion should replace the dot
    let json = r#"{"services": [{"deploymentConfiguration": {"x": 1}}, {"deploymentConfiguration": {"x": 2}}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute ".services" to cache base query and type
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string()),
        "base_query should be '.services'"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects),
        "base_type should be ArrayOfObjects"
    );

    // Step 2: Type a dot (syntax error, doesn't update base)
    app.input.textarea.insert_char('.');

    // Step 3: Accept autocomplete suggestion "[].deploymentConfiguration"
    insert_suggestion_from_app(&mut app, &test_suggestion("[].deploymentConfiguration"));

    // Should produce .services[].deploymentConfiguration (NOT .services.[].deploymentConfiguration)
    assert_eq!(app.input.query(), ".services[].deploymentConfiguration");

    // Verify the query executes correctly
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("x"));
    assert!(result.contains("1"));
    assert!(result.contains("2"));
}

#[test]
fn test_nested_array_suggestion_replaces_trailing_dot() {
    // Test deeply nested arrays: .services[].capacityProviderStrategy[].
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1}]}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute base query to cache state
    app.input
        .textarea
        .insert_str(".services[].capacityProviderStrategy[]");
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[]");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services[].capacityProviderStrategy[]".to_string())
    );
    // With only 1 service, this returns a single object, not destructured
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::Object)
    );

    // Step 2: Type trailing dot
    app.input.textarea.insert_char('.');

    // Step 3: Accept autocomplete suggestion "base"
    // Note: suggestion is "base" (no prefix) since Object after CloseBracket
    insert_suggestion_from_app(&mut app, &test_suggestion("base"));

    // Should produce .services[].capacityProviderStrategy[].base
    assert_eq!(
        app.input.query(),
        ".services[].capacityProviderStrategy[].base"
    );

    // Verify the query executes and returns the base values
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("0"));
}

#[test]
fn test_array_suggestion_after_pipe() {
    // After pipe, array suggestions should include leading dot
    let json = r#"{"services": [{"name": "svc1"}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute base query
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string())
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects)
    );

    // Step 2: Type " | ."
    app.input.textarea.insert_str(" | .");

    // Step 3: Accept autocomplete suggestion ".[].name" (WITH leading dot after pipe)
    insert_suggestion_from_app(&mut app, &test_suggestion(".[].name"));

    // Should produce .services | .[].name (NOT .services | . | .[].name)
    assert_eq!(app.input.query(), ".services | .[].name");

    // Verify execution
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("svc1"));
}

#[test]
fn test_array_suggestion_after_pipe_exact_user_flow() {
    // Replicate exact user flow: type partial, select, then pipe
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
    let mut app = test_app(json);

    // Step 1: Type ".ser" (partial)
    app.input.textarea.insert_str(".ser");
    // Note: .ser returns null, base stays at "."

    // Step 2: Select ".services" from autocomplete
    // In real app, user would Tab here with suggestion ".services"
    insert_suggestion_from_app(&mut app, &test_suggestion(".services"));

    // Validate: should now be ".services"
    assert_eq!(app.input.query(), ".services");

    // Step 3: Wait for async execution to complete (autocomplete now uses debouncer)
    execute_query_and_wait(&mut app);

    // Verify base is now cached after successful execution
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string()),
        "base should be '.services' after insertion executed it"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects)
    );

    // Step 4: Type " | ."
    app.input.textarea.insert_str(" | .");

    // Step 5: Select ".[].capacityProviderStrategy"
    insert_suggestion_from_app(&mut app, &test_suggestion(".[].capacityProviderStrategy"));

    // Should produce: .services | .[].capacityProviderStrategy
    // NOT: .services | . | .[].capacityProviderStrategy
    assert_eq!(
        app.input.query(),
        ".services | .[].capacityProviderStrategy"
    );
}

#[test]
fn test_pipe_after_typing_space() {
    // Test typing space then pipe character by character
    let json = r#"{"services": [{"name": "svc1"}]}"#;
    let mut app = test_app(json);

    // Step 1: Type and execute ".services"
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string())
    );

    // Step 2: Type space (executes ".services ")
    app.input.textarea.insert_char(' ');
    app.query.as_mut().unwrap().execute(".services ");

    // Step 3: Type | (executes ".services |" - syntax error, base stays at ".services")
    app.input.textarea.insert_char('|');
    app.query.as_mut().unwrap().execute(".services |");

    // Step 4: Type space then dot
    app.input.textarea.insert_str(" .");

    // Step 5: Accept suggestion
    insert_suggestion_from_app(&mut app, &test_suggestion(".[].name"));

    // Should be: base + " | " + suggestion
    // Base is trimmed, so: ".services" + " | " + ".[].name" = ".services | .[].name" ✅
    assert_eq!(app.input.query(), ".services | .[].name");
}

#[test]
fn test_suggestions_persist_when_typing_partial_after_array() {
    // Critical: When typing partial field after [], suggestions should persist
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1, "capacityProvider": "x"}]}]}"#;
    let mut app = test_app(json);

    // Step 1: Type the full path up to the last array
    app.input
        .textarea
        .insert_str(".services[].capacityProviderStrategy[]");
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[]");
    app.update_autocomplete();

    // Cache should have the array element objects with fields: base, weight, capacityProvider
    assert!(
        app.query
            .as_ref()
            .unwrap()
            .last_successful_result_unformatted
            .is_some()
    );
    let cached = app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
        .clone();

    // Step 2: Type a dot - should still have cached result
    app.input.textarea.insert_char('.');
    // Query is now ".services[].capacityProviderStrategy[]." which is syntax error
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[].");

    // Cache should NOT be cleared (syntax error doesn't update cache)
    assert_eq!(
        app.query
            .as_ref()
            .unwrap()
            .last_successful_result_unformatted,
        cached
    );

    // Step 3: Type a partial "b" - query returns multiple nulls
    app.input.textarea.insert_char('b');
    // Query is now ".services[].capacityProviderStrategy[].b" which returns multiple nulls
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[].b");

    // CRITICAL: Cache should STILL not be cleared (multiple nulls shouldn't overwrite)
    assert_eq!(
        app.query
            .as_ref()
            .unwrap()
            .last_successful_result_unformatted,
        cached
    );

    // Step 4: Update autocomplete - should still show suggestions based on cached result
    app.update_autocomplete();

    // Should have suggestions for the cached object fields
    let suggestions = app.autocomplete.suggestions();
    assert!(
        !suggestions.is_empty(),
        "Suggestions should persist when typing partial that returns null"
    );

    // Should have "base" suggestion (filtered by partial "b")
    assert!(
        suggestions.iter().any(|s| s.text.contains("base")),
        "Should suggest 'base' field when filtering by 'b'"
    );
}

#[test]
fn test_suggestions_persist_with_optional_chaining_and_partial() {
    // Critical: When typing partial after []?, suggestions should persist
    // Realistic scenario: some services have capacityProviderStrategy, some don't
    let json = r#"{
        "services": [
            {
                "serviceName": "service1",
                "capacityProviderStrategy": [
                    {"base": 0, "weight": 1, "capacityProvider": "FARGATE"},
                    {"base": 0, "weight": 2, "capacityProvider": "FARGATE_SPOT"}
                ]
            },
            {
                "serviceName": "service2"
            },
            {
                "serviceName": "service3",
                "capacityProviderStrategy": [
                    {"base": 1, "weight": 3, "capacityProvider": "EC2"}
                ]
            }
        ]
    }"#;
    let mut app = test_app(json);

    // Step 1: Execute query with optional chaining up to the array
    app.input
        .textarea
        .insert_str(".services[].capacityProviderStrategy[]?");
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[]?");

    // This should return the object with base, weight, capacityProvider fields
    let cached_before_partial = app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
        .clone();
    assert!(cached_before_partial.is_some());
    assert!(cached_before_partial.as_ref().unwrap().contains("base"));

    // Step 2: Type a dot
    app.input.textarea.insert_char('.');
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[]?.");
    // Syntax error - cache should remain
    assert_eq!(
        app.query
            .as_ref()
            .unwrap()
            .last_successful_result_unformatted,
        cached_before_partial
    );

    // Step 3: Type partial "b"
    app.input.textarea.insert_char('b');
    app.query
        .as_mut()
        .unwrap()
        .execute(".services[].capacityProviderStrategy[]?.b");

    // This returns single "null" (not multiple) due to optional chaining
    // Cache should NOT be updated
    assert_eq!(
        app.query
            .as_ref()
            .unwrap()
            .last_successful_result_unformatted,
        cached_before_partial,
        "Cache should not be overwritten by null result from partial field"
    );

    // Step 4: Update autocomplete
    app.update_autocomplete();

    // Should have suggestions based on the cached object
    let suggestions = app.autocomplete.suggestions();
    assert!(
        !suggestions.is_empty(),
        "Suggestions should persist when typing partial after []?"
    );

    // Should suggest "base" (filtered by partial "b")
    assert!(
        suggestions.iter().any(|s| s.text.contains("base")),
        "Should suggest 'base' field when filtering by 'b' after []?"
    );
}

#[test]
fn test_field_access_after_jq_keyword_preserves_space() {
    // Test that field access after "then" preserves the space
    // Bug: ".services[] | if has(\"x\") then .field" becomes "then.field" (no space)
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
    let mut app = test_app(json);

    // Step 1: Execute base query
    app.input.textarea.insert_str(".services[]");
    app.query.as_mut().unwrap().execute(".services[]");

    // Step 2: Type if-then with field access
    app.input
        .textarea
        .insert_str(" | if has(\"capacityProviderStrategy\") then .ca");

    // Step 3: Accept field suggestion (with leading dot as it would come from get_suggestions)
    insert_suggestion_from_app(&mut app, &test_suggestion(".capacityProviderStrategy"));

    // Should produce: .services[] | if has("capacityProviderStrategy") then .capacityProviderStrategy
    // NOT: .services[] | if has("capacityProviderStrategy") thencapacityProviderStrategy
    assert_eq!(
        app.input.query(),
        ".services[] | if has(\"capacityProviderStrategy\") then .capacityProviderStrategy"
    );

    // Verify there's a space before the field name
    assert!(
        app.input.query().contains("then .capacityProviderStrategy"),
        "Should have space between 'then' and field name"
    );
    assert!(
        !app.input.query().contains("thencapacityProviderStrategy"),
        "Should NOT concatenate 'then' with field name"
    );
}

#[test]
fn test_field_access_after_else_preserves_space() {
    // Test that field access after "else" preserves the space
    let json = r#"{"services": [{"name": "test"}]}"#;
    let mut app = test_app(json);

    // Execute base query
    app.input.textarea.insert_str(".services[]");
    app.query.as_mut().unwrap().execute(".services[]");

    // Type if-then-else with field access
    app.input
        .textarea
        .insert_str(" | if has(\"name\") then .name else .na");

    // Accept field suggestion (with leading dot as it would come from get_suggestions)
    insert_suggestion_from_app(&mut app, &test_suggestion(".name"));

    // Should have space between "else" and field
    assert!(
        app.input.query().contains("else .name"),
        "Should have space between 'else' and field name"
    );
    assert!(
        !app.input.query().contains("elsename"),
        "Should NOT concatenate 'else' with field name"
    );
}

#[test]
fn test_autocomplete_inside_if_statement() {
    // Autocomplete inside complex query should only replace the local part
    let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
    let mut app = test_app(json);

    // User types complex query with if/then
    app.input
        .textarea
        .insert_str(".services | if has(\"capacityProviderStrategy\") then .ca");

    // Execute to cache state (this will likely error due to incomplete query)
    app.query
        .as_mut()
        .unwrap()
        .execute(".services | if has(\"capacityProviderStrategy\") then .ca");

    // The issue: when Tab is pressed, entire query gets replaced with base + suggestion
    // Expected: only ".ca" should be replaced
    // Actual: entire query replaced with ".services[].capacityProviderStrategy"

    // TODO: This test documents the bug - we need smarter insertion
    // For now, this is a known limitation when using autocomplete inside complex expressions
}

#[test]
fn test_root_field_suggestion() {
    // At root, typing "." and selecting field should replace "." with ".field"
    let json = r#"{"services": [{"name": "test"}], "status": "active"}"#;
    let mut app = test_app(json);

    // Validate initial state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".".to_string()),
        "base_query should be '.' initially"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::Object),
        "base_type should be Object"
    );

    // User types "."
    app.input.textarea.insert_str(".");

    // Accept suggestion ".services" (with leading dot since at root after NoOp)
    insert_suggestion_from_app(&mut app, &test_suggestion(".services"));

    // Should produce ".services" NOT "..services"
    assert_eq!(app.input.query(), ".services");

    // Verify query executes correctly
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();
    assert!(result.contains("name"));
}

#[test]
fn test_field_suggestion_replaces_from_dot() {
    // When accepting .field suggestion at root, should replace from last dot
    let json = r#"{"name": "test", "age": 30}"#;
    let mut app = test_app(json);

    // Initial state: "." was executed during App::new()
    // Validate initial state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".".to_string()),
        "base_query should be '.' initially"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::Object),
        "base_type should be Object for root"
    );

    // Simulate: user typed ".na" and cursor is at end
    app.input.textarea.insert_str(".na");

    // Accept autocomplete suggestion "name" (no leading dot since after Dot)
    insert_suggestion_from_app(&mut app, &test_suggestion("name"));

    // Should produce .name (replace from the dot)
    assert_eq!(app.input.query(), ".name");
}

#[test]
fn test_autocomplete_with_real_ecs_like_data() {
    // Test with data structure similar to AWS ECS services
    let json = r#"{
        "services": [
            {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc1", "serviceName": "service1"},
            {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc2", "serviceName": "service2"},
            {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc3", "serviceName": "service3"},
            {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc4", "serviceName": "service4"},
            {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc5", "serviceName": "service5"}
        ]
    }"#;
    let mut app = test_app(json);

    // Step 1: Execute ".services" to cache base
    app.input.textarea.insert_str(".services");
    app.query.as_mut().unwrap().execute(".services");

    // Validate cached state
    assert_eq!(
        app.query.as_ref().unwrap().base_query_for_suggestions,
        Some(".services".to_string()),
        "base_query should be '.services'"
    );
    assert_eq!(
        app.query.as_ref().unwrap().base_type_for_suggestions,
        Some(ResultType::ArrayOfObjects),
        "base_type should be ArrayOfObjects"
    );

    // Step 2: Type ".s" (partial)
    app.input.textarea.insert_str(".s");

    // Step 3: Accept "[].serviceArn" (no leading dot since after NoOp)
    insert_suggestion_from_app(&mut app, &test_suggestion("[].serviceArn"));

    let query_text = app.input.query();
    assert_eq!(query_text, ".services[].serviceArn");

    // Verify execution returns ALL 5 serviceArns
    let result = app.query.as_ref().unwrap().result.as_ref().unwrap();

    // Check for all service ARNs
    assert!(result.contains("svc1"));
    assert!(result.contains("svc2"));
    assert!(result.contains("svc3"));
    assert!(result.contains("svc4"));
    assert!(result.contains("svc5"));

    // Count non-null values
    let lines: Vec<&str> = result.lines().collect();
    let non_null_lines: Vec<&str> = lines
        .iter()
        .filter(|line| !line.trim().contains("null"))
        .copied()
        .collect();

    assert!(
        non_null_lines.len() >= 5,
        "Should have at least 5 non-null results, got {}",
        non_null_lines.len()
    );
}
