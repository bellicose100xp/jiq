//! Tests for AI event handling
//!
//! This module contains unit tests and property-based tests for the AI event handlers.
//! Tests are organized into separate submodules for better organization.

// Re-export test modules
#[path = "ai_events_tests/ai_flow_tests.rs"]
mod ai_flow_tests;
#[path = "ai_events_tests/application_tests.rs"]
mod application_tests;
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
pub(crate) use proptest::prelude::*;
pub(crate) use std::sync::mpsc;
