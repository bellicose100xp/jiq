//! Tests for AI provider abstraction
//!
//! This module contains tests for provider configuration validation, error handling,
//! and factory methods. Tests are organized into submodules by provider.

// Re-export test modules
#[path = "provider_tests/anthropic_tests.rs"]
mod anthropic_tests;
#[path = "provider_tests/bedrock_tests.rs"]
mod bedrock_tests;
#[path = "provider_tests/error_tests.rs"]
mod error_tests;
#[path = "provider_tests/gemini_tests.rs"]
mod gemini_tests;
#[path = "provider_tests/openai_tests.rs"]
mod openai_tests;

// Re-export common imports for use in submodules
pub(crate) use super::*;
pub(crate) use crate::config::ai_types::{
    AiConfig, AiProviderType, AnthropicConfig, BedrockConfig, GeminiConfig, OpenAiConfig,
};
