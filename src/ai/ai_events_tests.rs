//! Tests for AI event handling
//!
//! This module contains unit tests and property-based tests for the AI event handlers.
//! Tests are organized into separate submodules for better organization.

// Re-export test modules
#[path = "ai_events_tests/ai_flow_tests.rs"]
mod ai_flow_tests;
#[path = "ai_events_tests/application_tests.rs"]
mod application_tests;
#[path = "ai_events_tests/chat_question_tests.rs"]
mod chat_question_tests;
#[path = "ai_events_tests/debounce_tests.rs"]
mod debounce_tests;
#[path = "ai_events_tests/property_tests.rs"]
mod property_tests;
#[path = "ai_events_tests/query_result_tests.rs"]
mod query_result_tests;
#[path = "ai_events_tests/selection_tests.rs"]
mod selection_tests;

// Re-export common test utilities for use in submodules
pub(crate) use super::ai_events::*;
pub(crate) use super::ai_state::{AiRequest, AiResponse, AiState};
pub(crate) use super::chat::AiPrompt;
pub(crate) use proptest::prelude::*;
pub(crate) use std::sync::mpsc;

/// The final user turn of a prompt: the part that carries the current query,
/// its output or error, and the task the model is asked to do.
pub(crate) fn last_user_turn(prompt: &AiPrompt) -> &str {
    let last = prompt
        .messages
        .last()
        .expect("prompt has at least one turn");
    assert_eq!(
        last.role,
        super::chat::ChatRole::User,
        "prompts always end with a user turn"
    );
    &last.content
}
