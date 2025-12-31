//! Cross-cutting error handling tests for AI provider abstraction

use super::*;
use crate::config::ai_types::TEST_MAX_CONTEXT_LENGTH;

// =========================================================================
// Unit Tests for No Default Provider
// =========================================================================

#[test]
fn test_from_config_returns_error_when_provider_is_none() {
    // Create config with provider = None
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

    // Should return NotConfigured error
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
    // Even with valid credentials for all providers, should fail when provider is None
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
        },
        gemini: GeminiConfig {
            api_key: Some("gemini-key".to_string()),
            model: Some("gemini-pro".to_string()),
        },
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
    };

    let result = AsyncAiProvider::from_config(&config);

    // Should still return error - no default fallback
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

// =========================================================================
// Property-Based Tests for Error Variants Provider Field
// =========================================================================

// **Feature: error-refactoring, Property 1: Error variants contain provider field**
// *For any* `AiError` variant (except `Cancelled`), the error SHALL contain a `provider`
// field that is a non-empty string.
// **Validates: Requirements 1.1, 1.2, 1.3, 1.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_error_variants_contain_provider_field(
        provider in "[A-Za-z][A-Za-z0-9_-]{2,20}",
        message in "[a-zA-Z0-9 .,!?_-]{1,100}",
        code in 100u16..600u16,
    ) {
        // Test NotConfigured - verify provider field exists and is non-empty
        let err = AiError::NotConfigured {
            provider: provider.clone(),
            message: message.clone(),
        };
        if let AiError::NotConfigured { provider: p, .. } = &err {
            prop_assert!(!p.is_empty(), "NotConfigured provider field should not be empty");
            prop_assert_eq!(p, &provider, "NotConfigured provider field should match input");
        } else {
            prop_assert!(false, "Expected NotConfigured variant");
        }

        // Test Network - verify provider field exists and is non-empty
        let err = AiError::Network {
            provider: provider.clone(),
            message: message.clone(),
        };
        if let AiError::Network { provider: p, .. } = &err {
            prop_assert!(!p.is_empty(), "Network provider field should not be empty");
            prop_assert_eq!(p, &provider, "Network provider field should match input");
        } else {
            prop_assert!(false, "Expected Network variant");
        }

        // Test Api - verify provider field exists and is non-empty
        let err = AiError::Api {
            provider: provider.clone(),
            code,
            message: message.clone(),
        };
        if let AiError::Api { provider: p, .. } = &err {
            prop_assert!(!p.is_empty(), "Api provider field should not be empty");
            prop_assert_eq!(p, &provider, "Api provider field should match input");
        } else {
            prop_assert!(false, "Expected Api variant");
        }

        // Test Parse - verify provider field exists and is non-empty
        let err = AiError::Parse {
            provider: provider.clone(),
            message: message.clone(),
        };
        if let AiError::Parse { provider: p, .. } = &err {
            prop_assert!(!p.is_empty(), "Parse provider field should not be empty");
            prop_assert_eq!(p, &provider, "Parse provider field should match input");
        } else {
            prop_assert!(false, "Expected Parse variant");
        }
    }
}

// =========================================================================
// Property-Based Tests for Error Display Format
// =========================================================================

// **Feature: error-refactoring, Property 2: Error display includes provider in brackets**
// *For any* `AiError` variant (except `Cancelled`) with any provider name and message,
// the `Display` output SHALL contain the provider name enclosed in square brackets at the start.
// **Validates: Requirements 1.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_error_display_includes_provider_in_brackets(
        provider in "[A-Za-z][A-Za-z0-9_-]{2,20}",
        message in "[a-zA-Z0-9 .,!?_-]{1,100}",
        code in 100u16..600u16,
    ) {
        // Test NotConfigured
        let err = AiError::NotConfigured {
            provider: provider.clone(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with(&format!("[{}]", provider)),
            "NotConfigured display should start with [{}], got: {}",
            provider,
            display
        );

        // Test Network
        let err = AiError::Network {
            provider: provider.clone(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with(&format!("[{}]", provider)),
            "Network display should start with [{}], got: {}",
            provider,
            display
        );

        // Test Api
        let err = AiError::Api {
            provider: provider.clone(),
            code,
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with(&format!("[{}]", provider)),
            "Api display should start with [{}], got: {}",
            provider,
            display
        );

        // Test Parse
        let err = AiError::Parse {
            provider: provider.clone(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.starts_with(&format!("[{}]", provider)),
            "Parse display should start with [{}], got: {}",
            provider,
            display
        );
    }
}

// =========================================================================
// Property-Based Tests for Provider Name Method
// =========================================================================

// **Feature: error-refactoring, Property 3: Provider name method returns correct identifier**
// *For any* `AsyncAiProvider` variant, calling `provider_name()` SHALL return a non-empty
// static string that matches the expected provider identifier.
// **Validates: Requirements 5.1, 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_provider_name_returns_correct_identifier(
        api_key in "[a-zA-Z0-9_-]{10,50}",
        model in "[a-z0-9-]{5,30}",
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
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let provider = AsyncAiProvider::from_config(&config).unwrap();
        let name = provider.provider_name();

        // Verify non-empty
        prop_assert!(!name.is_empty(), "Provider name should not be empty");

        // Verify correct identifier for Anthropic
        match provider {
            AsyncAiProvider::Anthropic(_) => {
                prop_assert_eq!(name, "Anthropic", "Anthropic provider should return 'Anthropic'");
            }
            AsyncAiProvider::Bedrock(_) => {
                prop_assert_eq!(name, "Bedrock", "Bedrock provider should return 'Bedrock'");
            }
            AsyncAiProvider::Openai(_) => {
                prop_assert_eq!(name, "OpenAI", "OpenAI provider should return 'OpenAI'");
            }
            AsyncAiProvider::Gemini(_) => {
                prop_assert_eq!(name, "Gemini", "Gemini provider should return 'Gemini'");
            }
        }
    }
}

// =========================================================================
// Property-Based Tests for Config Errors Including Correct Provider
// =========================================================================

// **Feature: error-refactoring, Property 4: Config errors include correct provider**
// *For any* invalid `AiConfig` that causes `from_config()` to fail, the returned `AiError`
// SHALL have a `provider` field matching the configured provider type.
// **Validates: Requirements 3.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_config_errors_include_correct_provider(
        model in "[a-z0-9-]{5,30}",
    ) {
        // Test 1: Missing API key should produce error with correct provider
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: None,
                model: Some(model.clone()),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        prop_assert!(result.is_err(), "Missing API key should produce error");

        if let Err(AiError::NotConfigured { provider, .. }) = result {
            prop_assert_eq!(
                provider, "Anthropic",
                "Error provider should match configured provider type"
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }

        // Test 2: Disabled config should produce error with correct provider
        let config = AiConfig {
            enabled: false,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: Some("valid-key".to_string()),
                model: Some(model.clone()),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        prop_assert!(result.is_err(), "Disabled config should produce error");

        if let Err(AiError::NotConfigured { provider, .. }) = result {
            prop_assert_eq!(
                provider, "Anthropic",
                "Error provider should match configured provider type"
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }

        // Test 3: Empty API key should produce error with correct provider
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig { max_tokens: 512,
                api_key: Some("".to_string()),
                model: Some(model),
            },
            bedrock: BedrockConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);
        prop_assert!(result.is_err(), "Empty API key should produce error");

        if let Err(AiError::NotConfigured { provider, .. }) = result {
            prop_assert_eq!(
                provider, "Anthropic",
                "Error provider should match configured provider type"
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }
}

// =========================================================================
// Property-Based Tests for No Default Provider Fallback
// =========================================================================

// **Feature: no-default-ai-provider, Property 4: No default provider fallback**
// *For any* AiConfig where provider is None and enabled is true, calling
// AsyncAiProvider::from_config SHALL return an AiError::NotConfigured error
// rather than creating a provider.
// **Validates: Requirements 1.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_no_default_provider_fallback(
        // Generate random config values to ensure property holds regardless of other settings
        api_key in "[a-zA-Z0-9_-]{10,50}",
        model in "[a-z0-9-]{5,30}",
        region in "(us-east-1|us-west-2|eu-west-1)",
        max_tokens in 100u32..2000u32,
    ) {
        // Create config with provider = None but enabled = true
        // Even with valid credentials for all providers, it should fail
        let config = AiConfig {
            enabled: true,
            provider: None,  // No provider configured
            anthropic: AnthropicConfig {
                max_tokens,
                api_key: Some(api_key.clone()),
                model: Some(model.clone()),
            },
            bedrock: BedrockConfig {
                region: Some(region),
                model: Some(model.clone()),
                profile: None,
            },
            openai: OpenAiConfig {
                api_key: Some(api_key.clone()),
                model: Some(model.clone()),
            },
            gemini: GeminiConfig {
                api_key: Some(api_key),
                model: Some(model),
            },
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
        };

        let result = AsyncAiProvider::from_config(&config);

        // Should always return an error when provider is None
        prop_assert!(
            result.is_err(),
            "from_config should return error when provider is None"
        );

        // Should be NotConfigured error with provider = "None"
        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "None",
                "Error provider should be 'None' when no provider configured"
            );
            prop_assert!(
                message.contains("No AI provider configured"),
                "Error message should indicate no provider configured: {}",
                message
            );
            prop_assert!(
                message.contains("https://github.com/bellicose100xp/jiq#configuration"),
                "Error message should contain README URL: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }
}

// **Feature: bedrock-provider, Property 6: Cancellation returns error without provider context**
// *For any* cancelled Bedrock request, the returned error SHALL be `AiError::Cancelled`
// without provider-specific context.
// **Validates: Requirements 5.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_cancelled_error_has_no_provider_context(
        // Generate random data to ensure property holds regardless of context
        _dummy in 0u32..1000u32,
    ) {
        let err = AiError::Cancelled;
        let display = format!("{}", err);

        // Cancelled error should NOT contain any provider name
        prop_assert!(
            !display.contains("Bedrock"),
            "Cancelled error should not contain 'Bedrock': {}",
            display
        );
        prop_assert!(
            !display.contains("Anthropic"),
            "Cancelled error should not contain 'Anthropic': {}",
            display
        );

        // Cancelled error should have a simple message
        prop_assert!(
            display.contains("cancelled") || display.contains("Cancelled"),
            "Cancelled error should mention cancellation: {}",
            display
        );

        // Verify it's the Cancelled variant
        prop_assert!(
            matches!(err, AiError::Cancelled),
            "Error should be Cancelled variant"
        );
    }
}
