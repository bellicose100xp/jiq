//! Tests for Gemini provider configuration validation

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

// =========================================================================
// Unit Tests for Gemini Provider from_config Validation
// =========================================================================

#[test]
fn test_gemini_from_config_missing_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Gemini),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig {
            api_key: None,
            model: Some("gemini-2.0-flash".to_string()),
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Gemini");
        assert!(message.contains("API key") || message.contains("api_key"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_gemini_from_config_missing_model() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Gemini),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig {
            api_key: Some("AIzaSyTest123".to_string()),
            model: None,
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Gemini");
        assert!(message.contains("model"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_gemini_from_config_valid_creates_client() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Gemini),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig {
            api_key: Some("AIzaSyTest123".to_string()),
            model: Some("gemini-2.0-flash".to_string()),
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_ok());
    if let Ok(AsyncAiProvider::Gemini(_)) = result {
        // Success - correct variant created
    } else {
        panic!("Expected Gemini variant, got {:?}", result);
    }
}

#[test]
fn test_gemini_provider_name() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Gemini),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig {
            api_key: Some("AIzaSyTest123".to_string()),
            model: Some("gemini-2.0-flash".to_string()),
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let provider = AsyncAiProvider::from_config(&config).unwrap();
    assert_eq!(provider.provider_name(), "Gemini");
}
