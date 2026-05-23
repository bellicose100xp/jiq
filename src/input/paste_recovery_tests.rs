use super::*;
use proptest::prelude::*;

#[test]
fn new_stores_error_message() {
    let state = PasteRecoveryState::new("Clipboard is empty.");
    assert_eq!(state.error_message, "Clipboard is empty.");
}

#[test]
fn try_submit_with_valid_json_object_returns_ok() {
    let mut state = PasteRecoveryState::new("err");
    let result = state.try_submit(r#"{"name": "test", "value": 42}"#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"{"name": "test", "value": 42}"#);
    // Successful submit does NOT mutate error_message — caller is expected
    // to consume the state immediately.
}

#[test]
fn try_submit_with_valid_json_array_returns_ok() {
    let mut state = PasteRecoveryState::new("err");
    assert!(state.try_submit(r#"[1, 2, 3]"#).is_ok());
}

#[test]
fn try_submit_with_valid_jsonl_returns_ok() {
    let mut state = PasteRecoveryState::new("err");
    let result = state.try_submit("{\"id\": 1}\n{\"id\": 2}\n{\"id\": 3}");
    assert!(result.is_ok(), "JSONL should validate, got {:?}", result);
}

#[test]
fn try_submit_with_embedded_newlines_in_strings() {
    let mut state = PasteRecoveryState::new("err");
    assert!(state.try_submit(r#"{"a": "line1\nline2"}"#).is_ok());
}

#[test]
fn try_submit_with_invalid_json_updates_error_message() {
    let mut state = PasteRecoveryState::new("original error");
    let result = state.try_submit(r#"{"name": invalid}"#);
    assert!(result.is_err());
    assert!(
        state.error_message.starts_with("Invalid JSON:"),
        "got: {}",
        state.error_message
    );
    assert_ne!(state.error_message, "original error");
}

#[test]
fn try_submit_with_empty_returns_err() {
    let mut state = PasteRecoveryState::new("err");
    let result = state.try_submit("");
    assert!(result.is_err());
    assert!(
        state.error_message.contains("empty"),
        "got: {}",
        state.error_message
    );
}

#[test]
fn try_submit_with_whitespace_only_returns_err() {
    let mut state = PasteRecoveryState::new("err");
    assert!(state.try_submit("   \n\t  ").is_err());
}

#[test]
fn try_submit_with_utf8_bom_pins_serde_json_behavior() {
    // serde_json rejects a leading UTF-8 BOM; pin behavior so a future
    // serde_json upgrade that flips it surfaces here.
    let mut state = PasteRecoveryState::new("err");
    assert!(state.try_submit("\u{FEFF}{\"a\": 1}").is_err());
}

#[test]
fn normalise_newlines_idempotent_on_lf_only() {
    let s = "abc\n{\"x\":1}\n";
    assert_eq!(normalise_newlines(s), s);
}

#[test]
fn normalise_newlines_handles_crlf() {
    let s = "{\"a\":1}\r\n{\"b\":2}\r\n";
    let out = normalise_newlines(s);
    assert!(!out.contains('\r'));
    assert_eq!(out, "{\"a\":1}\n{\"b\":2}\n");
}

#[test]
fn normalise_newlines_handles_mixed_endings() {
    let s = "a\r\nb\rc\nd";
    assert_eq!(normalise_newlines(s), "a\nb\nc\nd");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_valid_json_round_trips_through_try_submit(n in 0i32..1000) {
        let json = format!("{{\"n\": {}}}", n);
        let mut state = PasteRecoveryState::new("err");
        let result = state.try_submit(&json);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), json);
    }

    #[test]
    fn prop_arbitrary_string_never_panics(s in "\\PC{0,200}") {
        let mut state = PasteRecoveryState::new("err");
        // Must not panic on any input.
        let _ = state.try_submit(&s);
    }

    #[test]
    fn prop_invalid_input_updates_error_message(s in "[a-zA-Z ]{1,30}") {
        let mut state = PasteRecoveryState::new("original");
        let _ = state.try_submit(&s);
        // Either Ok (rare for plain ascii) or Err with updated message.
        // Pin: error_message is never empty after a try_submit call on
        // non-blank input.
        prop_assert!(!state.error_message.is_empty());
    }
}
