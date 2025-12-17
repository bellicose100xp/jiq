//! Tests for syntax_highlight

#[path = "syntax_highlight_tests/unit_tests.rs"]
mod unit_tests;

#[path = "syntax_highlight_tests/snapshot_tests.rs"]
mod snapshot_tests;

// Re-export common test utilities
pub(crate) use super::*;
pub(crate) use ratatui::style::{Color, Modifier};
pub(crate) use ratatui::text::Span;
pub(crate) use serde::Serialize;

// Snapshot helpers for tests
#[derive(Debug, Serialize)]
pub struct SerializableSpan {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
}

impl From<&Span<'_>> for SerializableSpan {
    fn from(span: &Span) -> Self {
        SerializableSpan {
            content: span.content.to_string(),
            fg: span.style.fg.map(color_to_string),
            bg: span.style.bg.map(color_to_string),
            modifiers: modifiers_to_vec(span.style.add_modifier),
        }
    }
}

fn color_to_string(color: Color) -> String {
    match color {
        Color::Reset => "reset".to_string(),
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "dark_gray".to_string(),
        Color::LightRed => "light_red".to_string(),
        Color::LightGreen => "light_green".to_string(),
        Color::LightYellow => "light_yellow".to_string(),
        Color::LightBlue => "light_blue".to_string(),
        Color::LightMagenta => "light_magenta".to_string(),
        Color::LightCyan => "light_cyan".to_string(),
        Color::White => "white".to_string(),
        Color::Rgb(r, g, b) => format!("rgb({},{},{})", r, g, b),
        Color::Indexed(i) => format!("indexed({})", i),
    }
}

fn modifiers_to_vec(modifiers: Modifier) -> Vec<String> {
    let mut result = Vec::new();
    if modifiers.contains(Modifier::BOLD) {
        result.push("bold".to_string());
    }
    if modifiers.contains(Modifier::DIM) {
        result.push("dim".to_string());
    }
    if modifiers.contains(Modifier::ITALIC) {
        result.push("italic".to_string());
    }
    if modifiers.contains(Modifier::UNDERLINED) {
        result.push("underlined".to_string());
    }
    if modifiers.contains(Modifier::REVERSED) {
        result.push("reversed".to_string());
    }
    if modifiers.contains(Modifier::SLOW_BLINK) {
        result.push("slow_blink".to_string());
    }
    if modifiers.contains(Modifier::RAPID_BLINK) {
        result.push("rapid_blink".to_string());
    }
    if modifiers.contains(Modifier::HIDDEN) {
        result.push("hidden".to_string());
    }
    if modifiers.contains(Modifier::CROSSED_OUT) {
        result.push("crossed_out".to_string());
    }
    result
}

pub fn serialize_spans(spans: &[Span]) -> Vec<SerializableSpan> {
    spans.iter().map(SerializableSpan::from).collect()
}
