//! Tests for Gemini provider configuration validation

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

// =========================================================================
// Property-Based Tests for Gemini Provider
// =========================================================================

// **Feature: gemini-provider, Property 4: Whitespace API key rejection**
// *For any* string composed entirely of whitespace characters, the system should treat it
// as missing and return a NotConfigured error.
// **Validates: Requirements 1.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_gemini_whitespace_api_key_rejection(
        model in "[a-z0-9-]{5,30}",
        // Generate whitespace-only strings (spaces, tabs, newlines)
        whitespace_key in prop::string::string_regex("[ \t\n\r]+").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Gemini),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig {
                api_key: Some(whitespace_key),
                model: Some(model),
            },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating Gemini provider with whitespace-only API key should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "Gemini",
                "Error provider should be 'Gemini'"
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
}

// **Feature: gemini-provider, Property 7: Network error propagation**
// *For any* network error, the system should return AiError::Network with provider "Gemini"
// and the error message.
// **Validates: Requirements 4.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_gemini_network_error_propagation(
        message in "[a-zA-Z0-9 .,!?_-]{1,100}",
    ) {
        let err = AiError::Network {
            provider: "Gemini".to_string(),
            message: message.clone(),
        };

        // Verify the error contains the correct provider
        if let AiError::Network { provider, message: msg } = &err {
            prop_assert_eq!(provider, "Gemini", "Network error should have provider 'Gemini'");
            prop_assert_eq!(msg, &message, "Network error should preserve the message");
        } else {
            prop_assert!(false, "Expected Network error variant");
        }

        // Verify display format includes provider in brackets
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with("[Gemini]"),
            "Network error display should start with [Gemini], got: {}",
            display
        );
        prop_assert!(
            display.contains(&message),
            "Network error display should contain the message: {}",
            display
        );
    }
}

// **Feature: gemini-provider, Property 8: API error propagation**
// *For any* HTTP error status code and message, the system should return AiError::Api
// with provider "Gemini", the status code, and the message.
// **Validates: Requirements 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_gemini_api_error_propagation(
        code in 400u16..600u16,
        message in "[a-zA-Z0-9 .,!?_-]{1,100}",
    ) {
        let err = AiError::Api {
            provider: "Gemini".to_string(),
            code,
            message: message.clone(),
        };

        // Verify the error contains the correct provider and code
        if let AiError::Api { provider, code: c, message: msg } = &err {
            prop_assert_eq!(provider, "Gemini", "API error should have provider 'Gemini'");
            prop_assert_eq!(*c, code, "API error should preserve the status code");
            prop_assert_eq!(msg, &message, "API error should preserve the message");
        } else {
            prop_assert!(false, "Expected Api error variant");
        }

        // Verify display format includes provider in brackets and code
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with("[Gemini]"),
            "API error display should start with [Gemini], got: {}",
            display
        );
        prop_assert!(
            display.contains(&code.to_string()),
            "API error display should contain the status code: {}",
            display
        );
        prop_assert!(
            display.contains(&message),
            "API error display should contain the message: {}",
            display
        );
    }
}

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
