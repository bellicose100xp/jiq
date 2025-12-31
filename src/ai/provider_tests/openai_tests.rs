//! Tests for OpenAI provider configuration validation

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

// =========================================================================
// Property-Based Tests for OpenAI Provider
// =========================================================================

// **Feature: openai-provider, Property 1: Provider selection from configuration**
// *For any* valid configuration with `provider = "openai"`, calling `AsyncAiProvider::from_config`
// should return an `Openai` variant containing an `AsyncOpenAiClient`.
// **Validates: Requirements 1.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_openai_provider_selection_from_config(
        api_key in "[a-zA-Z0-9_-]{10,50}",
        model in "[a-z0-9-]{5,30}",
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some(api_key),
                model: Some(model),
            },
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_ok(),
            "Creating OpenAI provider with valid config should succeed: {:?}",
            result
        );

        if let Ok(AsyncAiProvider::Openai(_)) = result {
            // Success - correct variant created
        } else {
            prop_assert!(false, "Expected Openai variant, got {:?}", result);
        }
    }

    #[test]
    fn prop_openai_missing_api_key_produces_error(
        model in "[a-z0-9-]{5,30}",
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: None,
                model: Some(model),
            },
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating OpenAI provider with missing API key should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "OpenAI",
                "Error provider should be 'OpenAI'"
            );
            prop_assert!(
                message.contains("API key") || message.contains("api_key"),
                "Error message should mention API key: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }

    #[test]
    fn prop_openai_empty_api_key_produces_error(
        model in "[a-z0-9-]{5,30}",
        // Generate empty or whitespace-only strings
        empty_key in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some(empty_key),
                model: Some(model),
            },
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating OpenAI provider with empty API key should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "OpenAI",
                "Error provider should be 'OpenAI'"
            );
            prop_assert!(
                message.contains("API key") || message.contains("api_key"),
                "Error message should mention API key: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }

    #[test]
    fn prop_openai_missing_model_produces_error(
        api_key in "[a-zA-Z0-9_-]{10,50}",
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some(api_key),
                model: None,
            },
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating OpenAI provider with missing model should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "OpenAI",
                "Error provider should be 'OpenAI'"
            );
            prop_assert!(
                message.contains("model"),
                "Error message should mention model: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }

    #[test]
    fn prop_openai_empty_model_produces_error(
        api_key in "[a-zA-Z0-9_-]{10,50}",
        // Generate empty or whitespace-only strings
        empty_model in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some(api_key),
                model: Some(empty_model),
            },
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating OpenAI provider with empty model should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "OpenAI",
                "Error provider should be 'OpenAI'"
            );
            prop_assert!(
                message.contains("model"),
                "Error message should mention model: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }
}

// =========================================================================
// Snapshot Tests for OpenAI Error Messages
// =========================================================================

#[cfg(test)]
mod openai_error_snapshots {
    use super::*;
    use insta::assert_snapshot;

    // Subtask 9.2: Write snapshot tests for error messages
    // Snapshot of NotConfigured error for missing API key
    #[test]
    fn snapshot_openai_missing_api_key_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: None,
                model: Some("gpt-4o-mini".to_string()),
            },
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        let error_message = format!("{}", result.unwrap_err());
        assert_snapshot!(error_message);
    }

    // Snapshot of NotConfigured error for missing model
    #[test]
    fn snapshot_openai_missing_model_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some("sk-proj-test123".to_string()),
                model: None,
            },
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        let error_message = format!("{}", result.unwrap_err());
        assert_snapshot!(error_message);
    }

    // Snapshot of NotConfigured error for whitespace-only API key
    #[test]
    fn snapshot_openai_whitespace_api_key_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some("   ".to_string()),
                model: Some("gpt-4o-mini".to_string()),
            },
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        let error_message = format!("{}", result.unwrap_err());
        assert_snapshot!(error_message);
    }

    // Snapshot of NotConfigured error for whitespace-only model
    #[test]
    fn snapshot_openai_whitespace_model_error() {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig {
                api_key: Some("sk-proj-test123".to_string()),
                model: Some("   ".to_string()),
            },
            gemini: GeminiConfig::default(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        let error_message = format!("{}", result.unwrap_err());
        assert_snapshot!(error_message);
    }

    // Snapshot of Api error messages
    #[test]
    fn snapshot_openai_api_error() {
        let error = AiError::Api {
            provider: "OpenAI".to_string(),
            code: 401,
            message: "Invalid API key provided".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }

    #[test]
    fn snapshot_openai_api_error_rate_limit() {
        let error = AiError::Api {
            provider: "OpenAI".to_string(),
            code: 429,
            message: "Rate limit exceeded".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }

    // Snapshot of Network error messages
    #[test]
    fn snapshot_openai_network_error() {
        let error = AiError::Network {
            provider: "OpenAI".to_string(),
            message: "Connection timeout".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }

    #[test]
    fn snapshot_openai_network_error_dns() {
        let error = AiError::Network {
            provider: "OpenAI".to_string(),
            message: "DNS resolution failed".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }

    // Snapshot of Parse error messages
    #[test]
    fn snapshot_openai_parse_error() {
        let error = AiError::Parse {
            provider: "OpenAI".to_string(),
            message: "Invalid JSON in SSE event".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }

    #[test]
    fn snapshot_openai_parse_error_request() {
        let error = AiError::Parse {
            provider: "OpenAI".to_string(),
            message: "Failed to serialize request body".to_string(),
        };
        let error_message = format!("{}", error);
        assert_snapshot!(error_message);
    }
}
