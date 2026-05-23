use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{self, Read, Write};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::{Duration, Instant};

use super::backend::{ClipboardError, ClipboardResult};

pub fn copy(text: &str) -> ClipboardResult {
    let sequence = encode_osc52(text);

    io::stdout()
        .write_all(sequence.as_bytes())
        .map_err(|_| ClipboardError::WriteError)?;

    io::stdout().flush().map_err(|_| ClipboardError::WriteError)
}

pub fn encode_osc52(text: &str) -> String {
    let encoded = STANDARD.encode(text);
    format!("\x1b]52;c;{}\x07", encoded)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Osc52ReadError {
    /// No response from the terminal within the timeout. The terminal probably
    /// does not support OSC 52 read, or the response is being intercepted by
    /// a multiplexer that has not been configured to forward it.
    Timeout,
    /// The terminal sent something, but it was not a well-formed OSC 52 c
    /// payload (or `;?` empty-clipboard reply).
    Malformed,
    /// stdin/stdout was not a TTY, or raw mode toggling failed. Carries the
    /// underlying message; surfaced via the `Debug` impl in debug logs only.
    Io(#[allow(dead_code)] String),
}

/// Query the terminal's clipboard via OSC 52 and return the decoded contents.
///
/// Writes `ESC ] 52 ; c ; ? BEL`, then reads bytes from stdin until the
/// matching `ESC ] 52 ; c ; <base64> ST` reply arrives or `timeout` elapses.
/// Briefly enables raw mode so the reply is not swallowed by the terminal
/// line discipline; raw mode is restored on every exit path.
///
/// The byte-by-byte read happens on a worker thread so the timeout is honored
/// even when the terminal never responds. On `Timeout`, that worker is still
/// blocked inside `Stdin::read` holding `io::stdin().lock()`. It does not exit
/// until the *next* stdin byte arrives - which, on a quiet terminal, is the
/// user's first keystroke after launch. That keystroke is then consumed by the
/// dying worker (which sees `Err` on the dropped channel and returns) instead
/// of crossterm's event loop, so the very first key press after a timed-out
/// OSC 52 read is silently dropped. We accept this trade-off because it only
/// triggers on the no-input launch path on terminals that already failed both
/// `arboard` and OSC 52 read - i.e. the user is going to see the multi-line
/// "no input" error and quit anyway, not press a meaningful key into the TUI.
pub fn read_with_timeout(timeout: Duration) -> Result<String, Osc52ReadError> {
    use crossterm::terminal;

    let raw_was_already_on = terminal::is_raw_mode_enabled().unwrap_or(false);
    if !raw_was_already_on {
        terminal::enable_raw_mode().map_err(|e| Osc52ReadError::Io(e.to_string()))?;
    }

    let result = read_inner(timeout);

    if !raw_was_already_on {
        let _ = terminal::disable_raw_mode();
    }

    result
}

fn read_inner(timeout: Duration) -> Result<String, Osc52ReadError> {
    {
        let mut stdout = io::stdout();
        stdout
            .write_all(b"\x1b]52;c;?\x07")
            .map_err(|e| Osc52ReadError::Io(e.to_string()))?;
        stdout
            .flush()
            .map_err(|e| Osc52ReadError::Io(e.to_string()))?;
    }

    let (tx, rx) = channel::<u8>();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut byte = [0u8; 1];
        while handle.read(&mut byte).map(|n| n > 0).unwrap_or(false) {
            if tx.send(byte[0]).is_err() {
                return;
            }
        }
    });

    let mut buffer: Vec<u8> = Vec::with_capacity(256);
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Osc52ReadError::Timeout);
        }

        match rx.recv_timeout(remaining) {
            Ok(byte) => buffer.push(byte),
            Err(RecvTimeoutError::Timeout) => return Err(Osc52ReadError::Timeout),
            Err(RecvTimeoutError::Disconnected) => return Err(Osc52ReadError::Malformed),
        }

        if let Some(text) = parse_response(&buffer)? {
            return Ok(text);
        }
    }
}

/// Try to extract the clipboard payload from buffered stdin bytes.
///
/// Returns `Ok(Some(text))` once a complete `ESC ] 52 ; c ; <base64> ST` reply
/// has been buffered, `Ok(None)` while still waiting for more bytes, and
/// `Err(Malformed)` when the buffer contains a syntactically invalid reply
/// or the terminal returned `;?` (no clipboard data).
pub(crate) fn parse_response(buffer: &[u8]) -> Result<Option<String>, Osc52ReadError> {
    let start = match find_subseq(buffer, b"\x1b]52;") {
        Some(idx) => idx,
        None => {
            // Bail early on a clearly-unrelated response so the caller can
            // surface a friendlier "no clipboard" message instead of waiting
            // out the full timeout on a terminal that already replied.
            if buffer.len() > 64 && !buffer.contains(&b'\x1b') {
                return Err(Osc52ReadError::Malformed);
            }
            return Ok(None);
        }
    };

    let payload_start = start + b"\x1b]52;".len();
    if buffer.len() <= payload_start {
        return Ok(None);
    }

    // Skip selection char ('c', 'p', etc.) and the separating ';'.
    let mut cursor = payload_start;
    while cursor < buffer.len() && buffer[cursor] != b';' {
        cursor += 1;
    }
    if cursor >= buffer.len() {
        return Ok(None);
    }
    cursor += 1;

    let body_start = cursor;
    let mut body_end = None;
    while cursor < buffer.len() {
        if buffer[cursor] == 0x07 {
            body_end = Some(cursor);
            break;
        }
        if buffer[cursor] == 0x1b && cursor + 1 < buffer.len() && buffer[cursor + 1] == b'\\' {
            body_end = Some(cursor);
            break;
        }
        cursor += 1;
    }
    let body_end = match body_end {
        Some(e) => e,
        None => return Ok(None),
    };

    let payload = &buffer[body_start..body_end];

    if payload == b"?" {
        return Err(Osc52ReadError::Malformed);
    }

    let decoded = STANDARD
        .decode(payload)
        .map_err(|_| Osc52ReadError::Malformed)?;
    String::from_utf8(decoded)
        .map(Some)
        .map_err(|_| Osc52ReadError::Malformed)
}

fn find_subseq(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[path = "osc52_tests.rs"]
mod osc52_tests;
