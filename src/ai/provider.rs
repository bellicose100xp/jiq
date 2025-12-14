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
mod tests {
    use super::*;
    use crate::config::ai_types::{AiConfig, AiProviderType, AnthropicConfig};
    use proptest::prelude::*;

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant, Property 3: Missing API key produces error state**
    // *For any* AiConfig with `enabled = true` but missing or empty `api_key`,
    // attempting to create an AiProvider should return an error.
    // **Validates: Requirements 1.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_missing_api_key_produces_error(
            debounce_ms in 100u64..5000u64,
            auto_show_on_error in prop::bool::ANY,
            model in "[a-z0-9-]{5,30}",
            max_tokens in 256u32..4096u32,
        ) {
            // Config with enabled=true but no API key
            let config = AiConfig {
                enabled: true,
                provider: AiProviderType::Anthropic,
                debounce_ms,
                auto_show_on_error,
                anthropic: AnthropicConfig {
                    api_key: None,
                    model: Some(model),
                    max_tokens,
                },
            };

            let result = AiProvider::from_config(&config);

            prop_assert!(
                result.is_err(),
                "Creating provider with missing API key should fail"
            );

            if let Err(AiError::NotConfigured(msg)) = result {
                prop_assert!(
                    msg.contains("API key") || msg.contains("api_key"),
                    "Error message should mention API key: {}",
                    msg
                );
            } else {
                prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
            }
        }

        #[test]
        fn prop_empty_api_key_produces_error(
            debounce_ms in 100u64..5000u64,
            auto_show_on_error in prop::bool::ANY,
            model in "[a-z0-9-]{5,30}",
            max_tokens in 256u32..4096u32,
            // Generate empty or whitespace-only strings
            empty_key in prop::string::string_regex("[ \t]*").unwrap(),
        ) {
            let config = AiConfig {
                enabled: true,
                provider: AiProviderType::Anthropic,
                debounce_ms,
                auto_show_on_error,
                anthropic: AnthropicConfig {
                    api_key: Some(empty_key),
                    model: Some(model),
                    max_tokens,
                },
            };

            let result = AiProvider::from_config(&config);

            prop_assert!(
                result.is_err(),
                "Creating provider with empty API key should fail"
            );

            if let Err(AiError::NotConfigured(msg)) = result {
                prop_assert!(
                    msg.contains("API key") || msg.contains("api_key"),
                    "Error message should mention API key: {}",
                    msg
                );
            } else {
                prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
            }
        }

        #[test]
        fn prop_valid_api_key_creates_provider(
            debounce_ms in 100u64..5000u64,
            auto_show_on_error in prop::bool::ANY,
            model in "[a-z0-9-]{5,30}",
            max_tokens in 256u32..4096u32,
            // Generate non-empty API keys
            api_key in "[a-zA-Z0-9_-]{10,50}",
        ) {
            let config = AiConfig {
                enabled: true,
                provider: AiProviderType::Anthropic,
                debounce_ms,
                auto_show_on_error,
                anthropic: AnthropicConfig {
                    api_key: Some(api_key),
                    model: Some(model),
                    max_tokens,
                },
            };

            let result = AiProvider::from_config(&config);

            prop_assert!(
                result.is_ok(),
                "Creating provider with valid API key should succeed"
            );
        }

        #[test]
        fn prop_disabled_config_produces_error(
            debounce_ms in 100u64..5000u64,
            auto_show_on_error in prop::bool::ANY,
            model in "[a-z0-9-]{5,30}",
            max_tokens in 256u32..4096u32,
            api_key in "[a-zA-Z0-9_-]{10,50}",
        ) {
            // Config with enabled=false (even with valid API key)
            let config = AiConfig {
                enabled: false,
                provider: AiProviderType::Anthropic,
                debounce_ms,
                auto_show_on_error,
                anthropic: AnthropicConfig {
                    api_key: Some(api_key),
                    model: Some(model),
                    max_tokens,
                },
            };

            let result = AiProvider::from_config(&config);

            prop_assert!(
                result.is_err(),
                "Creating provider with disabled config should fail"
            );

            if let Err(AiError::NotConfigured(msg)) = result {
                prop_assert!(
                    msg.contains("disabled"),
                    "Error message should mention disabled: {}",
                    msg
                );
            } else {
                prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
            }
        }
    }

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_from_config_missing_api_key() {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: None,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = AiProvider::from_config(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(AiError::NotConfigured(_))));
    }

    #[test]
    fn test_from_config_empty_api_key() {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = AiProvider::from_config(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(AiError::NotConfigured(_))));
    }

    #[test]
    fn test_from_config_whitespace_api_key() {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("   ".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = AiProvider::from_config(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(AiError::NotConfigured(_))));
    }

    #[test]
    fn test_from_config_valid_api_key() {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("sk-ant-test-key".to_string()),
                model: Some("claude-3-haiku".to_string()),
                max_tokens: 1024,
            },
            ..Default::default()
        };

        let result = AiProvider::from_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_config_disabled() {
        let config = AiConfig {
            enabled: false,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("sk-ant-test-key".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = AiProvider::from_config(&config);
        assert!(result.is_err());
        assert!(matches!(result, Err(AiError::NotConfigured(_))));
    }

    #[test]
    fn test_ai_error_display() {
        let err = AiError::NotConfigured("test message".to_string());
        assert_eq!(format!("{}", err), "AI not configured: test message");

        let err = AiError::Network("connection failed".to_string());
        assert_eq!(format!("{}", err), "Network error: connection failed");

        let err = AiError::Api {
            code: 429,
            message: "rate limited".to_string(),
        };
        assert_eq!(format!("{}", err), "API error (429): rate limited");

        let err = AiError::Parse("invalid json".to_string());
        assert_eq!(format!("{}", err), "Parse error: invalid json");

        let err = AiError::Cancelled;
        assert_eq!(format!("{}", err), "Request cancelled");
    }
}
