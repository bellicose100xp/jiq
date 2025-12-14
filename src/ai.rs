//! AI Assistant module for jiq
//!
//! Provides AI-powered contextual help for jq queries, including error troubleshooting,
//! function explanations, and query optimization suggestions.

mod ai_debouncer;
pub mod ai_events;
pub mod ai_render;
mod ai_state;
mod cache;
pub mod context;
pub mod prompt;
mod provider;
pub mod worker;

// Re-export main types (others are internal for Phase 1)
pub use ai_state::AiState;
// TODO: Remove #[allow(unused_imports)] when AiRequest/AiResponse are used externally
#[allow(unused_imports)]
pub use ai_state::{AiRequest, AiResponse};
