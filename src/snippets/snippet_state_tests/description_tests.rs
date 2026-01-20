use super::*;

#[test]
fn test_next_field_transitions_name_to_query() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    assert_eq!(*state.mode(), SnippetMode::CreateName);

    state.next_field();
    assert_eq!(*state.mode(), SnippetMode::CreateQuery);
}

#[test]
fn test_next_field_transitions_query_to_description() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    assert_eq!(*state.mode(), SnippetMode::CreateQuery);

    state.next_field();
    assert_eq!(*state.mode(), SnippetMode::CreateDescription);
}

#[test]
fn test_next_field_cycles_from_description_to_name() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    assert_eq!(*state.mode(), SnippetMode::CreateDescription);

    state.next_field();
    assert_eq!(*state.mode(), SnippetMode::CreateName);
}

#[test]
fn test_prev_field_transitions_description_to_query() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    assert_eq!(*state.mode(), SnippetMode::CreateDescription);

    state.prev_field();
    assert_eq!(*state.mode(), SnippetMode::CreateQuery);
}

#[test]
fn test_prev_field_transitions_query_to_name() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    assert_eq!(*state.mode(), SnippetMode::CreateQuery);

    state.prev_field();
    assert_eq!(*state.mode(), SnippetMode::CreateName);
}

#[test]
fn test_prev_field_cycles_from_name_to_description() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    assert_eq!(*state.mode(), SnippetMode::CreateName);

    state.prev_field();
    assert_eq!(*state.mode(), SnippetMode::CreateDescription);
}

#[test]
fn test_is_editing_in_create_description_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    assert!(state.is_editing());
}

#[test]
fn test_is_editing_in_create_query_mode() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    assert!(state.is_editing());
}

#[test]
fn test_save_new_snippet_with_description() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("Test Snippet");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state
        .description_textarea_mut()
        .insert_str("A test description");

    let result = state.save_new_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets().len(), 1);
    assert_eq!(state.snippets()[0].name, "Test Snippet");
    assert_eq!(state.snippets()[0].query, ".test | keys");
    assert_eq!(
        state.snippets()[0].description,
        Some("A test description".to_string())
    );
}

#[test]
fn test_save_new_snippet_without_description() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test | keys");
    state.name_textarea_mut().insert_str("Test Snippet");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description

    let result = state.save_new_snippet();
    assert!(result.is_ok());
    assert_eq!(state.snippets()[0].description, None);
}

#[test]
fn test_save_new_snippet_trims_description() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state.description_textarea_mut().insert_str("  trimmed  ");

    state.save_new_snippet().unwrap();
    assert_eq!(state.snippets()[0].description, Some("trimmed".to_string()));
}

#[test]
fn test_save_new_snippet_empty_description_is_none() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".test");
    state.name_textarea_mut().insert_str("Test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state.description_textarea_mut().insert_str("   ");

    state.save_new_snippet().unwrap();
    assert_eq!(state.snippets()[0].description, None);
}

#[test]
fn test_close_resets_description_textarea() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state.description_textarea_mut().insert_str("Test desc");
    assert_eq!(state.description_input(), "Test desc");

    state.close();
    assert_eq!(state.description_input(), "");
}

#[test]
fn test_cancel_create_resets_description_textarea() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state.description_textarea_mut().insert_str("Test desc");
    assert_eq!(state.description_input(), "Test desc");

    state.cancel_create();
    assert_eq!(state.description_input(), "");
    assert_eq!(*state.mode(), SnippetMode::Browse);
}

#[test]
fn test_enter_create_mode_clears_description_textarea() {
    let mut state = SnippetState::new();
    state.enter_create_mode(".test");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    state.description_textarea_mut().insert_str("Old desc");
    state.cancel_create();

    state.enter_create_mode(".new");
    state.next_field(); // Name -> Query
    state.next_field(); // Query -> Description
    assert_eq!(state.description_input(), "");
}

#[test]
fn test_query_edit_during_create() {
    let mut state = SnippetState::new_without_persistence();
    state.enter_create_mode(".original");
    state.name_textarea_mut().insert_str("Test Snippet");
    state.next_field(); // Name -> Query

    // Query textarea should be populated from pending_query
    state.query_textarea_mut().select_all();
    state.query_textarea_mut().cut();
    state.query_textarea_mut().insert_str(".modified | keys");

    state.next_field(); // Query -> Description (should update pending_query)
    state.next_field(); // Description -> Name (cycle)
    state.next_field(); // Name -> Query

    // Now save
    state.next_field(); // Query -> Description
    state.save_new_snippet().unwrap();

    assert_eq!(state.snippets()[0].query, ".modified | keys");
}
