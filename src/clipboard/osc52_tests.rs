//! Tests for clipboard/osc52

use super::*;
use proptest::prelude::*;

// Feature: clipboard, Property 1: OSC 52 encoding round-trip
// *For any* input text string, encoding it with OSC 52 format and then
// decoding the base64 portion should produce the original text.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_osc52_encoding_roundtrip(text in ".*") {
        let encoded = encode_osc52(&text);

        // Verify the format: \x1b]52;c;{base64}\x07
        assert!(encoded.starts_with("\x1b]52;c;"), "Should start with OSC 52 prefix");
        assert!(encoded.ends_with("\x07"), "Should end with BEL terminator");

        // Extract the base64 portion
        let prefix = "\x1b]52;c;";
        let suffix = "\x07";
        let base64_part = &encoded[prefix.len()..encoded.len() - suffix.len()];

        // Decode and verify round-trip
        let decoded_bytes = STANDARD.decode(base64_part)
            .expect("Base64 decoding should succeed");
        let decoded_text = String::from_utf8(decoded_bytes)
            .expect("Decoded bytes should be valid UTF-8");

        assert_eq!(decoded_text, text, "Round-trip should preserve original text");
    }
}

#[test]
fn test_encode_osc52_simple() {
    let result = encode_osc52("hello");
    assert_eq!(result, "\x1b]52;c;aGVsbG8=\x07");
}

#[test]
fn test_encode_osc52_empty() {
    let result = encode_osc52("");
    assert_eq!(result, "\x1b]52;c;\x07");
}

#[test]
fn test_encode_osc52_unicode() {
    let result = encode_osc52("日本語");
    assert!(result.starts_with("\x1b]52;c;"));
    assert!(result.ends_with("\x07"));

    let base64_part = &result[7..result.len() - 1];
    let decoded = STANDARD.decode(base64_part).unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), "日本語");
}

// =============================================================================
// OSC 52 read response parsing
// =============================================================================

#[test]
fn test_parse_response_complete_with_bel_terminator() {
    let payload = STANDARD.encode("hello");
    let buffer = format!("\x1b]52;c;{}\x07", payload);
    let result = parse_response(buffer.as_bytes()).unwrap();
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn test_parse_response_complete_with_st_terminator() {
    let payload = STANDARD.encode("hello");
    let buffer = format!("\x1b]52;c;{}\x1b\\", payload);
    let result = parse_response(buffer.as_bytes()).unwrap();
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn test_parse_response_unicode_payload() {
    let payload = STANDARD.encode("日本語");
    let buffer = format!("\x1b]52;c;{}\x07", payload);
    let result = parse_response(buffer.as_bytes()).unwrap();
    assert_eq!(result, Some("日本語".to_string()));
}

#[test]
fn test_parse_response_partial_returns_none() {
    // Only the prefix has arrived; we should keep waiting for more bytes.
    let buffer = b"\x1b]52;c;aGVsbG8";
    let result = parse_response(buffer).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_parse_response_without_prefix_returns_none() {
    let buffer = b"\x1b[A";
    let result = parse_response(buffer).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_parse_response_question_mark_reply_is_malformed() {
    // Some terminals reply `\x1b]52;c;?\x07` to refuse a read; treat as malformed.
    let buffer = b"\x1b]52;c;?\x07";
    let result = parse_response(buffer);
    assert!(matches!(result, Err(Osc52ReadError::Malformed)));
}

#[test]
fn test_parse_response_invalid_base64_is_malformed() {
    let buffer = b"\x1b]52;c;not-base-64-!!!\x07";
    let result = parse_response(buffer);
    assert!(matches!(result, Err(Osc52ReadError::Malformed)));
}

#[test]
fn test_parse_response_skips_leading_garbage() {
    // The terminal may have echoed unrelated bytes before the OSC 52 reply.
    let payload = STANDARD.encode("hello");
    let mut buffer = b"some prefix bytes ".to_vec();
    buffer.extend_from_slice(format!("\x1b]52;c;{}\x07", payload).as_bytes());
    let result = parse_response(&buffer).unwrap();
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn test_parse_response_empty_payload_decodes_to_empty_string() {
    let buffer = b"\x1b]52;c;\x07";
    let result = parse_response(buffer).unwrap();
    assert_eq!(result, Some(String::new()));
}

#[test]
fn test_parse_response_long_unrelated_buffer_bails_out() {
    // 65+ bytes with no ESC at all means the terminal isn't going to send a
    // reply; bail early so the loop doesn't sit on the timeout.
    let buffer = vec![b'x'; 100];
    let result = parse_response(&buffer);
    assert!(matches!(result, Err(Osc52ReadError::Malformed)));
}
