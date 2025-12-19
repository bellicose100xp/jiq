//! Tests for Anthropic provider configuration validation

use super::*;

// =========================================================================
// Property-Based Tests for AsyncAiProvider (Anthropic)
// =========================================================================

// **Feature: ai-assistant, Property 3: Missing API key produces error state**
// *For any* AiConfig with `enabled = true` but missing or empty `api_key`,
// attempting to create an AsyncAiProvider should return an error.
// **Validates: Requirements 1.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_missing_api_key_produces_error(
        model in "[a-z0-9-]{5,30}",
    ) {
        // Config with enabled=true but no API key
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: None,
                model: Some(model),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating provider with missing API key should fail"
        );

        if let Err(AiError::NotConfigured { message, .. }) = result {
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
    fn prop_empty_api_key_produces_error(
        model in "[a-z0-9-]{5,30}",
        // Generate empty or whitespace-only strings
        empty_key in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: Some(empty_key),
                model: Some(model),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating provider with empty API key should fail"
        );

        if let Err(AiError::NotConfigured { message, .. }) = result {
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
    fn prop_valid_api_key_creates_provider(
        model in "[a-z0-9-]{5,30}",
        // Generate non-empty API keys
        api_key in "[a-zA-Z0-9_-]{10,50}",
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: Some(api_key),
                model: Some(model),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_ok(),
            "Creating provider with valid API key should succeed"
        );
    }

    #[test]
    fn prop_disabled_config_produces_error(
        model in "[a-z0-9-]{5,30}",
        api_key in "[a-zA-Z0-9_-]{10,50}",
    ) {
        // Config with enabled=false (even with valid API key)
        let config = AiConfig {
            enabled: false,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: Some(api_key),
                model: Some(model),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating provider with disabled config should fail"
        );

        if let Err(AiError::NotConfigured { message, .. }) = result {
            prop_assert!(
                message.contains("disabled"),
                "Error message should mention disabled: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }
}

// =========================================================================
// Unit Tests for AsyncAiProvider (Anthropic)
// =========================================================================

#[test]
fn test_async_from_config_missing_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: None,
            ..Default::default()
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_empty_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("".to_string()),
            ..Default::default()
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_whitespace_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("   ".to_string()),
            ..Default::default()
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_valid_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("sk-ant-test-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_async_from_config_disabled() {
    let config = AiConfig {
        enabled: false,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("sk-ant-test-key".to_string()),
            ..Default::default()
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}
