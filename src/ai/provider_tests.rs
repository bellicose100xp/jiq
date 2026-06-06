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

#[cfg(test)]
mod round2_coverage_tests {
    use super::*;
    use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;
    use std::sync::mpsc;

    /// Build a config with `enabled: false` for the given provider, populating that
    /// provider's credentials so the disabled-mode branch is the only thing that can fail.
    fn disabled_config_for(provider: AiProviderType) -> AiConfig {
        let mut config = AiConfig {
            enabled: false,
            provider: Some(provider),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };
        match provider {
            AiProviderType::Anthropic => {
                config.anthropic = AnthropicConfig {
                    api_key: Some("sk-ant-test".to_string()),
                    model: Some("claude-3-haiku".to_string()),
                    max_tokens: 512,
                };
            }
            AiProviderType::Bedrock => {
                config.bedrock = BedrockConfig {
                    region: Some("us-east-1".to_string()),
                    model: Some("anthropic.claude-3-haiku".to_string()),
                    profile: None,
                };
            }
            AiProviderType::Openai => {
                config.openai = OpenAiConfig {
                    api_key: Some("sk-openai-test".to_string()),
                    model: Some("gpt-4o-mini".to_string()),
                    base_url: None,
                };
            }
            AiProviderType::Gemini => {
                config.gemini = GeminiConfig {
                    api_key: Some("AIzaSyTest".to_string()),
                    model: Some("gemini-2.0-flash".to_string()),
                };
            }
        }
        config
    }

    /// Build a valid (enabled, fully-credentialed) config for the given provider so
    /// `from_config` succeeds and yields the matching enum variant.
    fn valid_config_for(provider: AiProviderType) -> AiConfig {
        let mut config = disabled_config_for(provider);
        config.enabled = true;
        config
    }

    // Drives the per-provider name arms inside the `!config.enabled` block (lines 106-108).
    // The existing disabled tests only use Anthropic (line 105), so Bedrock/OpenAI/Gemini
    // name arms — and the lowercased remediation hint — were untested.
    #[test]
    fn test_async_from_config_disabled_bedrock_openai_gemini() {
        let cases = [
            (AiProviderType::Bedrock, "Bedrock"),
            (AiProviderType::Openai, "OpenAI"),
            (AiProviderType::Gemini, "Gemini"),
        ];

        for (provider_type, expected_name) in cases {
            let config = disabled_config_for(provider_type);
            let result = AsyncAiProvider::from_config(&config);

            match result {
                Err(AiError::NotConfigured { provider, message }) => {
                    assert_eq!(
                        provider, expected_name,
                        "disabled-mode provider name for {:?}",
                        provider_type
                    );
                    assert!(
                        message.contains("disabled"),
                        "message should mention disabled: {}",
                        message
                    );
                    assert!(
                        message.contains(&expected_name.to_lowercase()),
                        "message should embed lowercased provider name '{}': {}",
                        expected_name.to_lowercase(),
                        message
                    );
                }
                other => panic!(
                    "Expected NotConfigured for disabled {:?}, got {:?}",
                    provider_type, other
                ),
            }
        }
    }

    // Reaches the Anthropic missing-model ok_or_else closure (lines 137-139): a valid api_key
    // but no model. Every existing Anthropic test either supplies both fields (succeeds) or
    // fails earlier at the api_key check, so this required-field arm was uncovered.
    #[test]
    fn test_anthropic_from_config_missing_model_produces_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig {
                api_key: Some("sk-ant-test".to_string()),
                model: None,
                max_tokens: 512,
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        match result {
            Err(AiError::NotConfigured { provider, message }) => {
                assert_eq!(provider, "Anthropic");
                assert!(
                    message.contains("model"),
                    "message should mention model: {}",
                    message
                );
            }
            other => panic!("Expected NotConfigured for missing model, got {:?}", other),
        }
    }

    // Same arm via a whitespace-only model: the `.filter(|m| !m.trim().is_empty())` rejects it,
    // so the missing-model closure still fires.
    #[test]
    fn test_anthropic_from_config_whitespace_model_produces_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig {
                api_key: Some("sk-ant-test".to_string()),
                model: Some("   ".to_string()),
                max_tokens: 512,
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        match result {
            Err(AiError::NotConfigured { provider, message }) => {
                assert_eq!(provider, "Anthropic");
                assert!(message.contains("model"));
            }
            other => panic!(
                "Expected NotConfigured for whitespace model, got {:?}",
                other
            ),
        }
    }

    // Covers the enum-level AsyncAiProvider::stream_with_cancel method (lines 275-304) and all
    // four delegating match arms. The round-1 cancel tests call client.stream_with_cancel
    // directly on concrete structs, bypassing this enum dispatch. Each client's early
    // is_cancelled guard makes a pre-cancelled token return Cancelled with zero network I/O,
    // so this protects the dispatch wiring for every provider variant.
    #[test]
    fn test_provider_stream_with_cancel_returns_cancelled_for_all_variants() {
        let providers = [
            AiProviderType::Anthropic,
            AiProviderType::Bedrock,
            AiProviderType::Openai,
            AiProviderType::Gemini,
        ];

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        for provider_type in providers {
            let config = valid_config_for(provider_type);
            let provider = AsyncAiProvider::from_config(&config)
                .unwrap_or_else(|e| panic!("from_config failed for {:?}: {:?}", provider_type, e));

            let token = CancellationToken::new();
            token.cancel();
            let (tx, _rx) = mpsc::channel();

            let result = rt
                .block_on(async move { provider.stream_with_cancel("prompt", 1, token, tx).await });

            assert!(
                matches!(result, Err(AiError::Cancelled)),
                "enum dispatch for {:?} should return Cancelled, got {:?}",
                provider_type,
                result
            );
        }
    }
}
