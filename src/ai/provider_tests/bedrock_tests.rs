//! Tests for Bedrock provider configuration validation and error handling

use super::*;

// =========================================================================
// Property-Based Tests for Bedrock Provider Validation
// =========================================================================

// **Feature: bedrock-provider, Property 2: Model validation rejects invalid values**
// *For any* Bedrock configuration where `model` is None, empty, or contains only whitespace
// characters, `from_config` SHALL return a `NotConfigured` error with provider "Bedrock".
// **Validates: Requirements 1.3, 1.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_bedrock_missing_model_produces_error(
        region in "[a-z]{2}-[a-z]+-[0-9]",
        profile in proptest::option::of("[a-zA-Z0-9_-]{3,20}"),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig {
                region: Some(region),
                model: None,
                profile,
            },
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating Bedrock provider with missing model should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "Bedrock",
                "Error provider should be 'Bedrock'"
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
    fn prop_bedrock_empty_model_produces_error(
        region in "[a-z]{2}-[a-z]+-[0-9]",
        profile in proptest::option::of("[a-zA-Z0-9_-]{3,20}"),
        // Generate empty or whitespace-only strings
        empty_model in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig {
                region: Some(region),
                model: Some(empty_model),
                profile,
            },
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating Bedrock provider with empty model should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "Bedrock",
                "Error provider should be 'Bedrock'"
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

// **Feature: bedrock-provider, Property 3: Region is required**
// *For any* Bedrock configuration where `region` is None or empty, `from_config` SHALL
// return a `NotConfigured` error with provider "Bedrock".
// **Validates: Requirements 2.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_bedrock_missing_region_produces_error(
        model in "[a-z0-9.-]{10,50}",
        profile in proptest::option::of("[a-zA-Z0-9_-]{3,20}"),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig {
                region: None,
                model: Some(model),
                profile,
            },
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating Bedrock provider with missing region should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "Bedrock",
                "Error provider should be 'Bedrock'"
            );
            prop_assert!(
                message.contains("region"),
                "Error message should mention region: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }

    #[test]
    fn prop_bedrock_empty_region_produces_error(
        model in "[a-z0-9.-]{10,50}",
        profile in proptest::option::of("[a-zA-Z0-9_-]{3,20}"),
        // Generate empty or whitespace-only strings
        empty_region in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig {
                region: Some(empty_region),
                model: Some(model),
                profile,
            },
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_err(),
            "Creating Bedrock provider with empty region should fail"
        );

        if let Err(AiError::NotConfigured { provider, message }) = result {
            prop_assert_eq!(
                provider, "Bedrock",
                "Error provider should be 'Bedrock'"
            );
            prop_assert!(
                message.contains("region"),
                "Error message should mention region: {}",
                message
            );
        } else {
            prop_assert!(false, "Expected NotConfigured error, got {:?}", result);
        }
    }

    #[test]
    fn prop_bedrock_valid_config_creates_provider(
        region in "[a-z]{2}-[a-z]+-[0-9]",
        model in "[a-z0-9.-]{10,50}",
        profile in proptest::option::of("[a-zA-Z0-9_-]{3,20}"),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig {
                region: Some(region),
                model: Some(model),
                profile,
            },
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
        };

        let result = AsyncAiProvider::from_config(&config);

        prop_assert!(
            result.is_ok(),
            "Creating Bedrock provider with valid config should succeed: {:?}",
            result
        );

        if let Ok(AsyncAiProvider::Bedrock(_)) = result {
            // Success - correct variant created
        } else {
            prop_assert!(false, "Expected Bedrock variant, got {:?}", result);
        }
    }
}

// =========================================================================
// Property-Based Tests for Bedrock Error Handling
// =========================================================================

// **Feature: bedrock-provider, Property 5: Non-cancelled errors include Bedrock provider context**
// *For any* error returned from Bedrock operations (except `Cancelled`), the error message
// SHALL contain "Bedrock" as the provider identifier.
// **Validates: Requirements 4.4, 6.1, 6.2, 6.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_bedrock_errors_include_provider_context(
        message in "[a-zA-Z0-9 .,!?_-]{1,100}",
        code in 100u16..600u16,
    ) {
        // Test NotConfigured with Bedrock provider
        let err = AiError::NotConfigured {
            provider: "Bedrock".to_string(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.contains("Bedrock"),
            "NotConfigured error should contain 'Bedrock': {}",
            display
        );

        // Test Network with Bedrock provider
        let err = AiError::Network {
            provider: "Bedrock".to_string(),
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.contains("Bedrock"),
            "Network error should contain 'Bedrock': {}",
            display
        );

        // Test Api with Bedrock provider
        let err = AiError::Api {
            provider: "Bedrock".to_string(),
            code,
            message: message.clone(),
        };
        let display = format!("{}", err);
        prop_assert!(
            display.contains("Bedrock"),
            "Api error should contain 'Bedrock': {}",
            display
        );

        // Test AwsSdk (Bedrock-specific error variant)
        let err = AiError::AwsSdk(message.clone());
        let display = format!("{}", err);
        prop_assert!(
            display.contains("Bedrock"),
            "AwsSdk error should contain 'Bedrock': {}",
            display
        );
    }
}
