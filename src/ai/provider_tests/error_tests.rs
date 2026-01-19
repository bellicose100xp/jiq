//! Cross-cutting error handling tests for AI provider abstraction

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

#[test]
fn test_from_config_returns_error_when_provider_is_none() {
    let config = AiConfig {
        enabled: true,
        provider: None,
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("valid-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);

    assert!(result.is_err(), "Expected error when provider is None");

    match result {
        Err(AiError::NotConfigured { provider, message }) => {
            assert_eq!(provider, "None", "Provider should be 'None'");
            assert!(
                message.contains("No AI provider configured"),
                "Message should indicate no provider: {}",
                message
            );
            assert!(
                message.contains("https://github.com/bellicose100xp/jiq#configuration"),
                "Message should contain README URL: {}",
                message
            );
        }
        _ => panic!("Expected NotConfigured error, got {:?}", result),
    }
}

#[test]
fn test_from_config_error_when_provider_none_even_with_all_credentials() {
    let config = AiConfig {
        enabled: true,
        provider: None,
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("anthropic-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig {
            region: Some("us-east-1".to_string()),
            model: Some("anthropic.claude-3-haiku".to_string()),
            profile: None,
        },
        openai: OpenAiConfig {
            api_key: Some("openai-key".to_string()),
            model: Some("gpt-4".to_string()),
            base_url: None,
        },
        gemini: GeminiConfig {
            api_key: Some("gemini-key".to_string()),
            model: Some("gemini-pro".to_string()),
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);

    assert!(
        result.is_err(),
        "Should return error even with all credentials when provider is None"
    );
    assert!(
        matches!(result, Err(AiError::NotConfigured { .. })),
        "Should be NotConfigured error"
    );
}

#[test]
fn test_ai_error_display() {
    let err = AiError::NotConfigured {
        provider: "Anthropic".to_string(),
        message: "test message".to_string(),
    };
    assert_eq!(
        format!("{}", err),
        "[Anthropic] AI not configured: test message"
    );

    let err = AiError::Network {
        provider: "Anthropic".to_string(),
        message: "connection failed".to_string(),
    };
    assert_eq!(
        format!("{}", err),
        "[Anthropic] Network error: connection failed"
    );

    let err = AiError::Api {
        provider: "Anthropic".to_string(),
        code: 429,
        message: "rate limited".to_string(),
    };
    assert_eq!(
        format!("{}", err),
        "[Anthropic] API error (429): rate limited"
    );

    let err = AiError::Parse {
        provider: "Anthropic".to_string(),
        message: "invalid json".to_string(),
    };
    assert_eq!(format!("{}", err), "[Anthropic] Parse error: invalid json");

    let err = AiError::Cancelled;
    assert_eq!(format!("{}", err), "Request cancelled");
}

#[test]
fn test_error_display_starts_with_provider_in_brackets() {
    let err = AiError::NotConfigured {
        provider: "TestProvider".to_string(),
        message: "test".to_string(),
    };
    assert!(format!("{}", err).starts_with("[TestProvider]"));

    let err = AiError::Network {
        provider: "TestProvider".to_string(),
        message: "test".to_string(),
    };
    assert!(format!("{}", err).starts_with("[TestProvider]"));

    let err = AiError::Api {
        provider: "TestProvider".to_string(),
        code: 500,
        message: "test".to_string(),
    };
    assert!(format!("{}", err).starts_with("[TestProvider]"));

    let err = AiError::Parse {
        provider: "TestProvider".to_string(),
        message: "test".to_string(),
    };
    assert!(format!("{}", err).starts_with("[TestProvider]"));
}

#[test]
fn test_cancelled_error_has_no_provider_context() {
    let err = AiError::Cancelled;
    let display = format!("{}", err);

    assert!(!display.contains("Bedrock"));
    assert!(!display.contains("Anthropic"));
    assert!(display.contains("cancelled") || display.contains("Cancelled"));
}

#[test]
fn test_provider_name_returns_correct_identifier() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("test-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let provider = AsyncAiProvider::from_config(&config).unwrap();
    assert_eq!(provider.provider_name(), "Anthropic");
}

#[test]
fn test_config_error_includes_correct_provider_for_missing_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: None,
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, .. }) = result {
        assert_eq!(provider, "Anthropic");
    } else {
        panic!("Expected NotConfigured error");
    }
}

#[test]
fn test_config_error_includes_correct_provider_for_disabled() {
    let config = AiConfig {
        enabled: false,
        provider: Some(AiProviderType::Anthropic),
        anthropic: AnthropicConfig {
            max_tokens: 512,
            api_key: Some("valid-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
        },
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig::default(),
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());

    if let Err(AiError::NotConfigured { provider, .. }) = result {
        assert_eq!(provider, "Anthropic");
    } else {
        panic!("Expected NotConfigured error");
    }
}
