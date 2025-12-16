//! Tests for AI event handling
//!
//! This module contains unit tests and property-based tests for the AI event handlers.
//! Tests are organized into separate submodules for better organization.

// Re-export test modules
#[path = "ai_events_tests/application_tests.rs"]
mod application_tests;
#[path = "ai_events_tests/debounce_tests.rs"]
mod debounce_tests;
#[path = "ai_events_tests/integration_tests.rs"]
mod integration_tests;
#[path = "ai_events_tests/property_tests.rs"]
mod property_tests;
#[path = "ai_events_tests/query_result_tests.rs"]
mod query_result_tests;
#[path = "ai_events_tests/selection_tests.rs"]
mod selection_tests;
#[path = "ai_events_tests/toggle_tests.rs"]
mod toggle_tests;

// Re-export common test utilities for use in submodules
pub(crate) use super::ai_events::*;
pub(crate) use super::ai_state::{AiRequest, AiResponse, AiState};
pub(crate) use proptest::prelude::*;
pub(crate) use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
pub(crate) use std::sync::mpsc;

// Helper to create key events
pub(crate) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(crate) fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}
