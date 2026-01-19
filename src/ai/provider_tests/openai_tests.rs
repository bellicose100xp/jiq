//! Tests for OpenAI provider configuration validation

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

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
                base_url: None,
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
                base_url: None,
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
                base_url: None,
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
                base_url: None,
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

#[test]
fn test_openai_provider_name_default() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Openai),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig {
            api_key: Some("sk-test".to_string()),
            model: Some("gpt-4o-mini".to_string()),
            base_url: None,
        },
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let provider = AsyncAiProvider::from_config(&config).unwrap();
    assert_eq!(provider.provider_name(), "OpenAI");
}

#[test]
fn test_openai_provider_name_explicit_openai_url() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Openai),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig {
            api_key: Some("sk-test".to_string()),
            model: Some("gpt-4o-mini".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
        },
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let provider = AsyncAiProvider::from_config(&config).unwrap();
    assert_eq!(provider.provider_name(), "OpenAI");
}

#[test]
fn test_openai_provider_name_custom_endpoint() {
    let config = AiConfig {
        enabled: true,
        provider: Some(AiProviderType::Openai),
        anthropic: AnthropicConfig::default(),
        bedrock: BedrockConfig::default(),
        openai: OpenAiConfig {
            api_key: None,
            model: Some("llama3".to_string()),
            base_url: Some("http://localhost:11434/v1".to_string()),
        },
        gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let provider = AsyncAiProvider::from_config(&config).unwrap();
    assert_eq!(provider.provider_name(), "OpenAI-compatible");
}
