//! Tests for Anthropic provider configuration validation

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}
