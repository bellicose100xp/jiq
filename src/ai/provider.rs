//! AI provider abstraction
//!
//! Defines the AsyncAiProvider enum, AiError types, and factory for creating provider instances.
//! Uses async/await with tokio for non-blocking streaming and CancellationToken for request cancellation.

use std::sync::mpsc::Sender;

use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::ai::ai_state::AiResponse;
use crate::config::ai_types::{AiConfig, AiProviderType};

mod async_anthropic;
mod async_bedrock;

pub use async_anthropic::AsyncAnthropicClient;
pub use async_bedrock::AsyncBedrockClient;

/// Errors that can occur during AI operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AiError {
    /// AI is not configured (missing API key or disabled)
    #[error("[{provider}] AI not configured: {message}")]
    NotConfigured { provider: String, message: String },

    /// Network error during API request
    #[error("[{provider}] Network error: {message}")]
    Network { provider: String, message: String },

    /// API returned an error response
    #[error("[{provider}] API error ({code}): {message}")]
    Api {
        provider: String,
        code: u16,
        message: String,
    },

    /// Failed to parse API response
    #[error("[{provider}] Parse error: {message}")]
    Parse { provider: String, message: String },

    /// AWS SDK error (Bedrock-specific)
    #[error("[Bedrock] AWS SDK error: {0}")]
    AwsSdk(String),

    /// Request was cancelled
    #[error("Request cancelled")]
    Cancelled,
}

/// Async AI provider implementations with cancellation support
///
/// Uses async/await with tokio for non-blocking streaming and
/// CancellationToken for request cancellation.
#[derive(Debug, Clone)]
pub enum AsyncAiProvider {
    /// Anthropic Claude API (async)
    Anthropic(AsyncAnthropicClient),
    /// AWS Bedrock API (async)
    Bedrock(AsyncBedrockClient),
}

impl AsyncAiProvider {
    /// Returns the display name of the provider
    pub fn provider_name(&self) -> &'static str {
        match self {
            AsyncAiProvider::Anthropic(_) => "Anthropic",
            AsyncAiProvider::Bedrock(_) => "Bedrock",
        }
    }

    /// Create an async AI provider from configuration
    ///
    /// Returns an error if the configuration is invalid (e.g., missing API key)
    pub fn from_config(config: &AiConfig) -> Result<Self, AiError> {
        if !config.enabled {
            let provider_name = match config.provider {
                AiProviderType::Anthropic => "Anthropic",
                AiProviderType::Bedrock => "Bedrock",
            };
            return Err(AiError::NotConfigured {
                provider: provider_name.to_string(),
                message: format!(
                    "AI is disabled. Set 'enabled = true' in [ai] section with provider = \"{}\". See https://github.com/bellicose100xp/jiq#configuration for setup instructions.",
                    provider_name.to_lowercase()
                ),
            });
        }

        match config.provider {
            AiProviderType::Anthropic => {
                let api_key = config
                    .anthropic
                    .api_key
                    .as_ref()
                    .filter(|k| !k.trim().is_empty())
                    .ok_or_else(|| AiError::NotConfigured {
                        provider: "Anthropic".to_string(),
                        message: "Missing API key. Add 'api_key' in [ai.anthropic] section. Get your key from https://console.anthropic.com/settings/keys. See https://github.com/bellicose100xp/jiq#configuration for full setup.".to_string(),
                    })?;

                let model = config
                    .anthropic
                    .model
                    .as_ref()
                    .filter(|m| !m.trim().is_empty())
                    .ok_or_else(|| AiError::NotConfigured {
                        provider: "Anthropic".to_string(),
                        message: "Missing model. Add 'model' in [ai.anthropic] section (e.g., 'claude-haiku-4-5-20251001'). See https://github.com/bellicose100xp/jiq#configuration for examples.".to_string(),
                    })?;

                let provider = AsyncAiProvider::Anthropic(AsyncAnthropicClient::new(
                    api_key.clone(),
                    model.clone(),
                    config.anthropic.max_tokens,
                ));

                // Use provider_name to avoid dead code warning
                let _ = provider.provider_name();
                Ok(provider)
            }
            AiProviderType::Bedrock => {
                let region = config
                    .bedrock
                    .region
                    .as_ref()
                    .filter(|r| !r.trim().is_empty())
                    .ok_or_else(|| AiError::NotConfigured {
                        provider: "Bedrock".to_string(),
                        message: "Missing region. Add 'region' in [ai.bedrock] section (e.g., 'us-east-1'). Ensure AWS credentials are configured via environment variables or ~/.aws/credentials. See https://github.com/bellicose100xp/jiq#configuration for setup.".to_string(),
                    })?;

                let model = config
                    .bedrock
                    .model
                    .as_ref()
                    .filter(|m| !m.trim().is_empty())
                    .ok_or_else(|| AiError::NotConfigured {
                        provider: "Bedrock".to_string(),
                        message: "Missing model. Add 'model' in [ai.bedrock] section (e.g., 'anthropic.claude-3-haiku-20240307-v1:0'). See https://github.com/bellicose100xp/jiq#configuration for examples.".to_string(),
                    })?;

                let provider = AsyncAiProvider::Bedrock(AsyncBedrockClient::new(
                    region.clone(),
                    model.clone(),
                    config.bedrock.profile.clone(),
                ));

                // Use provider_name to avoid dead code warning
                let _ = provider.provider_name();
                Ok(provider)
            }
        }
    }

    /// Stream a response from the AI provider with cancellation support
    ///
    /// Uses async streaming and sends chunks via the response channel.
    /// Can be cancelled via the CancellationToken.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the API
    /// * `request_id` - Unique ID for this request
    /// * `cancel_token` - Token to cancel the request
    /// * `response_tx` - Channel to send response chunks
    ///
    /// # Returns
    /// * `Ok(())` - Stream completed successfully
    /// * `Err(AiError::Cancelled)` - Request was cancelled
    /// * `Err(AiError::*)` - Other errors
    pub async fn stream_with_cancel(
        &self,
        prompt: &str,
        request_id: u64,
        cancel_token: CancellationToken,
        response_tx: Sender<AiResponse>,
    ) -> Result<(), AiError> {
        match self {
            AsyncAiProvider::Anthropic(client) => {
                client
                    .stream_with_cancel(prompt, request_id, cancel_token, response_tx)
                    .await
            }
            AsyncAiProvider::Bedrock(client) => {
                client
                    .stream_with_cancel(prompt, request_id, cancel_token, response_tx)
                    .await
            }
        }
    }
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod provider_tests;
