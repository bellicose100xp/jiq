//! Tests for AI provider abstraction

use super::*;
use crate::config::ai_types::{AiConfig, AiProviderType, AnthropicConfig};
use proptest::prelude::*;

// =========================================================================
// Property-Based Tests for AsyncAiProvider
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
        max_tokens in 256u32..4096u32,
    ) {
        // Config with enabled=true but no API key
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: None,
                model: Some(model),
                max_tokens,
            },
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
        max_tokens in 256u32..4096u32,
        // Generate empty or whitespace-only strings
        empty_key in prop::string::string_regex("[ \t]*").unwrap(),
    ) {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some(empty_key),
                model: Some(model),
                max_tokens,
            },
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
        max_tokens in 256u32..4096u32,
        // Generate non-empty API keys
        api_key in "[a-zA-Z0-9_-]{10,50}",
    ) {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some(api_key),
                model: Some(model),
                max_tokens,
            },
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
        max_tokens in 256u32..4096u32,
        api_key in "[a-zA-Z0-9_-]{10,50}",
    ) {
        // Config with enabled=false (even with valid API key)
        let config = AiConfig {
            enabled: false,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some(api_key),
                model: Some(model),
                max_tokens,
            },
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
// Unit Tests for AsyncAiProvider
// =========================================================================

#[test]
fn test_async_from_config_missing_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: AiProviderType::Anthropic,
        anthropic: AnthropicConfig {
            api_key: None,
            ..Default::default()
        },
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_empty_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: AiProviderType::Anthropic,
        anthropic: AnthropicConfig {
            api_key: Some("".to_string()),
            ..Default::default()
        },
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_whitespace_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: AiProviderType::Anthropic,
        anthropic: AnthropicConfig {
            api_key: Some("   ".to_string()),
            ..Default::default()
        },
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
}

#[test]
fn test_async_from_config_valid_api_key() {
    let config = AiConfig {
        enabled: true,
        provider: AiProviderType::Anthropic,
        anthropic: AnthropicConfig {
            api_key: Some("sk-ant-test-key".to_string()),
            model: Some("claude-3-haiku".to_string()),
            max_tokens: 1024,
        },
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_async_from_config_disabled() {
    let config = AiConfig {
        enabled: false,
        provider: AiProviderType::Anthropic,
        anthropic: AnthropicConfig {
            api_key: Some("sk-ant-test-key".to_string()),
            ..Default::default()
        },
    };

    let result = AsyncAiProvider::from_config(&config);
    assert!(result.is_err());
    assert!(matches!(result, Err(AiError::NotConfigured { .. })));
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
        max_tokens in 256u32..4096u32,
    ) {
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some(api_key),
                model: Some(model),
                max_tokens,
            },
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
        max_tokens in 256u32..4096u32,
    ) {
        // Test 1: Missing API key should produce error with correct provider
        let config = AiConfig {
            enabled: true,
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: None,
                model: Some(model.clone()),
                max_tokens,
            },
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
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("valid-key".to_string()),
                model: Some(model.clone()),
                max_tokens,
            },
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
            provider: AiProviderType::Anthropic,
            anthropic: AnthropicConfig {
                api_key: Some("".to_string()),
                model: Some(model),
                max_tokens,
            },
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
