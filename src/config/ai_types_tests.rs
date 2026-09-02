//! Tests for ai_types

use super::*;
use crate::config::Config;

use super::BedrockConfig;
use super::GeminiConfig;
use super::OpenAiConfig;

#[test]
fn test_ai_config_default_values() {
    let config = AiConfig::default();
    assert!(!config.enabled);
    assert!(
        config.provider.is_none(),
        "AiConfig::default() should have provider = None"
    );
    assert!(config.anthropic.api_key.is_none());
    assert!(config.anthropic.model.is_none());
    assert_eq!(config.anthropic.max_tokens, 512);
    assert_eq!(
        config.max_context_length, 100_000,
        "Default must match the serde field default"
    );
    assert_eq!(
        config.request_timeout_secs, 120,
        "Default must match the serde field default"
    );
}

#[test]
fn test_anthropic_config_default_values() {
    let config = AnthropicConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
    assert_eq!(config.max_tokens, 512);
}

#[test]
fn test_bedrock_config_default_values() {
    let config = BedrockConfig::default();
    assert!(config.region.is_none());
    assert!(config.model.is_none());
    assert!(config.profile.is_none());
    assert!(config.effort.is_none());
    assert!(!config.context_1m);
}

#[test]
fn test_openai_config_default_values() {
    let config = OpenAiConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
    assert!(config.effort.is_none());
}

#[test]
fn test_effort_as_str_maps_every_variant() {
    assert_eq!(AiEffort::Minimal.as_str(), "minimal");
    assert_eq!(AiEffort::Low.as_str(), "low");
    assert_eq!(AiEffort::Medium.as_str(), "medium");
    assert_eq!(AiEffort::High.as_str(), "high");
    assert_eq!(AiEffort::Xhigh.as_str(), "xhigh");
    assert_eq!(AiEffort::Max.as_str(), "max");
}

#[test]
fn test_parse_bedrock_effort_and_context_1m() {
    let toml = r#"
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-east-1"
model = "global.anthropic.claude-sonnet-4-6-v1:0"
effort = "xhigh"
context_1m = true
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.bedrock.effort, Some(AiEffort::Xhigh));
    assert!(config.ai.bedrock.context_1m);
}

#[test]
fn test_parse_openai_effort() {
    let toml = r#"
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "bedrock-api-key"
model = "openai.gpt-5.6-terra"
base_url = "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1"
effort = "high"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.openai.effort, Some(AiEffort::High));
}

#[test]
fn test_parse_all_effort_levels() {
    for (raw, expected) in [
        ("minimal", AiEffort::Minimal),
        ("low", AiEffort::Low),
        ("medium", AiEffort::Medium),
        ("high", AiEffort::High),
        ("xhigh", AiEffort::Xhigh),
        ("max", AiEffort::Max),
    ] {
        let toml =
            format!("[ai.bedrock]\nregion = \"us-east-1\"\nmodel = \"m\"\neffort = \"{raw}\"\n");
        let config: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.ai.bedrock.effort, Some(expected));
    }
}

#[test]
fn test_parse_request_timeout_secs() {
    let toml = r#"
[ai]
enabled = true
request_timeout_secs = 30
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.request_timeout_secs, 30);
}

#[test]
fn test_request_timeout_secs_defaults_to_120() {
    let config: Config = toml::from_str("[ai]\nenabled = true\n").unwrap();
    assert_eq!(config.ai.request_timeout_secs, 120);
}

#[test]
fn test_parse_extra_instructions() {
    let toml = r#"
[ai]
enabled = true
extra_instructions = "Prefer map() over .[] pipelines."
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(
        config.ai.extra_instructions.as_deref(),
        Some("Prefer map() over .[] pipelines.")
    );
}

#[test]
fn test_extra_instructions_omitted_defaults_to_none() {
    let toml = r#"
[ai]
enabled = true
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.extra_instructions.is_none());
}

#[test]
fn test_invalid_effort_fails_parse() {
    let toml = r#"
[ai.bedrock]
region = "us-east-1"
model = "m"
effort = "turbo"
"#;
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Invalid effort value should fail to parse");
}

#[test]
fn test_bedrock_effort_omitted_defaults_to_none() {
    let toml = r#"
[ai.bedrock]
region = "us-east-1"
model = "m"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.bedrock.effort.is_none());
    assert!(!config.ai.bedrock.context_1m);
}

#[test]
fn test_gemini_config_default_values() {
    let config = GeminiConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
}

#[test]
fn test_parse_ai_enabled() {
    let toml = r#"
[ai]
enabled = true

[ai.anthropic]
api_key = "sk-ant-test"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.anthropic.api_key, Some("sk-ant-test".to_string()));
}

#[test]
fn test_parse_ai_disabled() {
    let toml = r#"
[ai]
enabled = false
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(!config.ai.enabled);
}

#[test]
fn test_empty_ai_section_uses_defaults() {
    let toml = r#"
[ai]
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(!config.ai.enabled);
}

#[test]
fn test_partial_ai_section_uses_defaults() {
    let toml = r#"
[ai]
enabled = true
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert!(config.ai.anthropic.api_key.is_none());
}

#[test]
fn test_parse_anthropic_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "anthropic"

[ai.anthropic]
api_key = "sk-ant-test-key"
model = "claude-3-haiku-20240307"
max_tokens = 1024
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Anthropic));
    assert_eq!(config.ai.anthropic.max_tokens, 1024);
}

#[test]
fn test_parse_openai_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "sk-proj-test-key"
model = "gpt-4o-mini"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Openai));
    assert_eq!(
        config.ai.openai.api_key,
        Some("sk-proj-test-key".to_string())
    );
    assert_eq!(config.ai.openai.model, Some("gpt-4o-mini".to_string()));
}

#[test]
fn test_parse_bedrock_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-east-1"
model = "anthropic.claude-3-haiku-20240307-v1:0"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Bedrock));
    assert_eq!(config.ai.bedrock.region, Some("us-east-1".to_string()));
    assert_eq!(
        config.ai.bedrock.model,
        Some("anthropic.claude-3-haiku-20240307-v1:0".to_string())
    );
    assert!(config.ai.bedrock.profile.is_none());
}

#[test]
fn test_parse_bedrock_provider_with_profile() {
    let toml = r#"
[ai]
enabled = true
provider = "bedrock"

[ai.bedrock]
region = "us-west-2"
model = "anthropic.claude-3-sonnet-20240229-v1:0"
profile = "my-aws-profile"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.provider, Some(AiProviderType::Bedrock));
    assert_eq!(
        config.ai.bedrock.profile,
        Some("my-aws-profile".to_string())
    );
}

#[test]
fn test_parse_gemini_provider() {
    let toml = r#"
[ai]
enabled = true
provider = "gemini"

[ai.gemini]
api_key = "AIza-test-key"
model = "gemini-2.0-flash"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.enabled);
    assert_eq!(config.ai.provider, Some(AiProviderType::Gemini));
    assert_eq!(config.ai.gemini.api_key, Some("AIza-test-key".to_string()));
    assert_eq!(config.ai.gemini.model, Some("gemini-2.0-flash".to_string()));
}

#[test]
fn test_missing_ai_section_defaults_to_disabled() {
    let toml = r#"
[clipboard]
backend = "auto"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(!config.ai.enabled);
    assert!(config.ai.provider.is_none());
}

#[test]
fn test_invalid_provider_fails_parse() {
    let toml = r#"
[ai]
enabled = true
provider = "invalid"
"#;
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Invalid provider should fail to parse");
}

#[test]
fn test_deserialization_without_provider_yields_none() {
    let toml = r#"
[ai]
enabled = true

[ai.anthropic]
api_key = "sk-ant-test-key"
model = "claude-3-haiku-20240307"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.ai.provider.is_none());
    assert!(config.ai.enabled);
}

#[test]
fn test_whitespace_api_key_stored_as_is() {
    let toml = r#"
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "   "
model = "gpt-4o-mini"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.ai.openai.api_key, Some("   ".to_string()));
}
