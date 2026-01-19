//! Tests for Bedrock provider configuration validation and error handling

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

#[test]
fn test_bedrock_missing_model_produces_error() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("us-east-1".to_string()),
            model: None,
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Bedrock");
        assert!(message.contains("model"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_bedrock_empty_model_produces_error() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("us-east-1".to_string()),
            model: Some("".to_string()),
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Bedrock");
        assert!(message.contains("model"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_bedrock_whitespace_model_produces_error() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("us-east-1".to_string()),
            model: Some("   ".to_string()),
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Bedrock");
        assert!(message.contains("model"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_bedrock_missing_region_produces_error() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: None,
            model: Some("anthropic.claude-3-haiku".to_string()),
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Bedrock");
        assert!(message.contains("region"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_bedrock_empty_region_produces_error() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("".to_string()),
            model: Some("anthropic.claude-3-haiku".to_string()),
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, message }) = result {
        assert_eq!(provider, "Bedrock");
        assert!(message.contains("region"));
    } else {
        panic!("Expected NotConfigured error, got {:?}", result);
    }
}

#[test]
fn test_bedrock_valid_config_creates_provider() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("us-east-1".to_string()),
            model: Some("anthropic.claude-3-haiku".to_string()),
            profile: None,
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_ok());

    if let Ok(AsyncAiProvider::Bedrock(_)) = result {
        // Success
    } else {
        panic!("Expected Bedrock variant, got {:?}", result);
    }
}

#[test]
fn test_bedrock_valid_config_with_profile_creates_provider() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Bedrock),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig {
            region: Some("us-west-2".to_string()),
            model: Some("anthropic.claude-3-sonnet".to_string()),
            profile: Some("my-profile".to_string()),
        },
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_bedrock_error_includes_provider_context() {
    let err = AiError::NotConfigured {
        provider: "Bedrock".to_string(),
        message: "test error".to_string(),
    };
    assert!(format!("{}", err).contains("Bedrock"));

    let err = AiError::Network {
        provider: "Bedrock".to_string(),
        message: "connection failed".to_string(),
    };
    assert!(format!("{}", err).contains("Bedrock"));

    let err = AiError::Api {
        provider: "Bedrock".to_string(),
        code: 500,
        message: "server error".to_string(),
    };
    assert!(format!("{}", err).contains("Bedrock"));

    let err = AiError::AwsSdk("SDK error".to_string());
    assert!(format!("{}", err).contains("Bedrock"));
}
