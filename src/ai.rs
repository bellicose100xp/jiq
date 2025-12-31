//! AI Assistant module for jiq
//!
//! Provides AI-powered contextual help for jq queries, including error troubleshooting,
//! function explanations, and query optimization suggestions.

pub mod ai_events;
pub mod ai_render;
pub mod ai_state; // Made public for integration tests
pub mod context;
pub mod prompt;
mod provider;
pub mod render;
pub mod selection;
pub mod suggestion;
pub mod worker;

#[cfg(test)]
mod ai_events_tests;

#[cfg(test)]
mod ai_render_tests;

// Note: ai_state_tests is declared in ai_state.rs to avoid duplicate module loading

// Re-export main types (others are internal for Phase 1)
pub use ai_state::AiState;
// TODO: Remove #[allow(unused_imports)] when AiRequest/AiResponse are used externally
#[allow(unused_imports)]
pub use ai_state::{AiRequest, AiResponse};
// Re-export suggestion types
#[allow(unused_imports)]
pub use suggestion::{Suggestion, SuggestionType};
