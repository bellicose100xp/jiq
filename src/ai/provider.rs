//! AI provider abstraction
//!
//! Defines the AiProvider enum, AiError types, and factory for creating provider instances.

use thiserror::Error;

use crate::config::ai_types::{AiConfig, AiProviderType};

mod anthropic;

pub use anthropic::AnthropicClient;

/// Errors that can occur during AI operations
#[derive(Debug, Error)]
pub enum AiError {
    /// AI is not configured (missing API key or disabled)
    #[error("AI not configured: {0}")]
    NotConfigured(String),

    /// Network error during API request
    #[error("Network error: {0}")]
    Network(String),

    /// API returned an error response
    #[error("API error ({code}): {message}")]
    Api { code: u16, message: String },

    /// Failed to parse API response
    #[error("Parse error: {0}")]
    Parse(String),

    /// Request was cancelled
    #[error("Request cancelled")]
    // TODO: Remove #[allow(dead_code)] when cancellation is implemented
    #[allow(dead_code)] // Phase 1: Reserved for future cancellation support
    Cancelled,
}

/// AI provider implementations
#[derive(Debug)]
pub enum AiProvider {
    /// Anthropic Claude API
    Anthropic(AnthropicClient),
}

impl AiProvider {
    /// Create an AI provider from configuration
    ///
    /// Returns an error if the configuration is invalid (e.g., missing API key)
    pub fn from_config(config: &AiConfig) -> Result<Self, AiError> {
        if !config.enabled {
            return Err(AiError::NotConfigured(
                "AI is disabled in config".to_string(),
            ));
        }

        match config.provider {
            AiProviderType::Anthropic => {
                let api_key = config
                    .anthropic
                    .api_key
                    .as_ref()
                    .filter(|k| !k.trim().is_empty())
                    .ok_or_else(|| {
                        AiError::NotConfigured(
                            "Missing or empty API key in [ai.anthropic] config".to_string(),
                        )
                    })?;

                let model = config
                    .anthropic
                    .model
                    .as_ref()
                    .filter(|m| !m.trim().is_empty())
                    .ok_or_else(|| {
                        AiError::NotConfigured(
                            "Missing or empty model in [ai.anthropic] config".to_string(),
                        )
                    })?;

                Ok(AiProvider::Anthropic(AnthropicClient::new(
                    api_key.clone(),
                    model.clone(),
                    config.anthropic.max_tokens,
                )))
            }
        }
    }

    /// Stream a response from the AI provider
    ///
    /// Returns an iterator that yields text chunks as they arrive
    pub fn stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Iterator<Item = Result<String, AiError>> + '_>, AiError> {
        match self {
            AiProvider::Anthropic(client) => client.stream(prompt),
        }
    }
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod provider_tests;
