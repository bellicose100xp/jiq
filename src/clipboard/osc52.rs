//! OSC 52 clipboard backend
//!
//! Provides clipboard access via terminal escape sequences,
//! useful for remote sessions (SSH, tmux).

use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{self, Write};

use super::backend::{ClipboardError, ClipboardResult};

/// Copy text to clipboard using OSC 52 escape sequence
///
/// Format: \x1b]52;c;{base64}\x07
///
/// This writes the escape sequence directly to stdout, which terminal
/// emulators that support OSC 52 will interpret as a clipboard operation.
pub fn copy(text: &str) -> ClipboardResult {
    let sequence = encode_osc52(text);

    // Write directly to stdout
    io::stdout()
        .write_all(sequence.as_bytes())
        .map_err(|_| ClipboardError::WriteError)?;

    io::stdout().flush().map_err(|_| ClipboardError::WriteError)
}

/// Encode text for OSC 52 (exposed for testing)
///
/// Format: \x1b]52;c;{base64}\x07
///
/// The sequence consists of:
/// - `\x1b]52;` - OSC 52 introducer
/// - `c;` - clipboard selection (c = clipboard, p = primary)
/// - `{base64}` - base64-encoded content
/// - `\x07` - string terminator (BEL)
pub fn encode_osc52(text: &str) -> String {
    let encoded = STANDARD.encode(text);
    format!("\x1b]52;c;{}\x07", encoded)
}

#[cfg(test)]
mod tests {
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
        // "hello" in base64 is "aGVsbG8="
        assert_eq!(result, "\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn test_encode_osc52_empty() {
        let result = encode_osc52("");
        // Empty string in base64 is ""
        assert_eq!(result, "\x1b]52;c;\x07");
    }

    #[test]
    fn test_encode_osc52_unicode() {
        let result = encode_osc52("日本語");
        // Verify it starts and ends correctly
        assert!(result.starts_with("\x1b]52;c;"));
        assert!(result.ends_with("\x07"));

        // Verify round-trip
        let base64_part = &result[7..result.len() - 1];
        let decoded = STANDARD.decode(base64_part).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "日本語");
    }
}
