//! AI Assistant module for jiq
//!
//! Provides AI-powered help for jq queries: fixes when a query errors,
//! optimizations when it runs, and a conversation in the popup where the
//! user asks questions and gets answers plus applyable queries. The
//! conversation is replayed to the provider on every request.

pub mod ai_events;
pub mod ai_render;
pub mod ai_state; // Made public for integration tests
pub mod chat;
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
