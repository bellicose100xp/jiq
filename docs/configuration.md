---
title: Configuration
nav_order: 5
description: config.toml reference — clipboard backend, autocomplete depth, AI providers (Anthropic, OpenAI, Gemini, AWS Bedrock, OpenAI-compatible).
---

# Configuration
{: .no_toc }

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{: toc }
</details>

---

## Location

| OS | Path |
|:---|:---|
| Linux | `~/.config/jiq/config.toml` |
| macOS | `~/Library/Application Support/jiq/config.toml` |
| Windows | `%APPDATA%\jiq\config.toml` |

Optional. Defaults are reasonable; only AI requires config.

---

## Clipboard

```toml
[clipboard]
# Clipboard backend: "auto" (default), "system", or "osc52"
# - auto: tries system clipboard first, falls back to OSC 52
# - system: use only OS clipboard (may not work in SSH/tmux)
# - osc52: use terminal escape sequences (works in most modern terminals over SSH)
backend = "auto"
```

| Value | Use when |
|:---|:---|
| `auto` (default) | OS clipboard first, OSC 52 fallback. |
| `system` | Local desktop only. |
| `osc52` | SSH / tmux / mosh on a terminal that forwards OSC 52 (Ghostty, kitty, WezTerm, foot). |

See [Clipboard & paste recovery](./features/clipboard).

---

## Autocomplete

```toml
[autocomplete]
# Number of array elements sampled to discover field suggestions for arrays
# where fields differ across elements. Increasing this may improve suggestions
# but adds a performance cost.
# Range: 1 - 1000 (default: 10)
array_sample_size = 10
```

For field suggestions inside array elements, jiq samples up to `array_sample_size` elements and unions their keys. Bump this for heterogeneous arrays at a small startup cost per trigger.

---

## AI

```toml
[ai]
# Enable AI assistant
enabled = true
# Provider: "anthropic", "openai", "gemini", or "bedrock"
provider = "anthropic"
# Character limit at which JSON schema and output samples are truncated (default: 100000)
# Larger values send more context to AI but increase token usage/costs
max_context_length = 100000
```

Lightweight models are faster:

| Provider | Recommended model |
|:---|:---|
| Anthropic | `claude-haiku-4-5-20251001` |
| OpenAI | `gpt-4o-mini` |
| Gemini | `gemini-3-flash` |

See [AI assistant](./features/ai-assistant).

### Anthropic

```toml
[ai.anthropic]
# https://console.anthropic.com/settings/keys
api_key = "your-api-key-here"
model = "claude-haiku-4-5-20251001"
```

### OpenAI

```toml
[ai.openai]
# https://platform.openai.com/api-keys
api_key = "sk-proj-..."
model = "gpt-4o-mini"
```

### OpenAI-compatible (Ollama, LM Studio, x.ai Grok, others)

Set `provider = "openai"` and override `base_url`.

```toml
[ai.openai]
base_url = "https://your-api-endpoint/v1"
api_key  = "your-api-key"      # optional if the provider doesn't require one
model    = "model-name"
```

Ollama:

```toml
[ai.openai]
base_url = "http://localhost:11434/v1"
model    = "llama3"
```

LM Studio:

```toml
[ai.openai]
base_url = "http://localhost:1234/v1"
model    = "local-model"
```

x.ai Grok:

```toml
[ai.openai]
api_key  = "your-xai-api-key"
base_url = "https://api.x.ai/v1"
model    = "grok-4-fast-non-reasoning"
```

### Gemini

```toml
[ai.gemini]
# https://aistudio.google.com/apikey
api_key = "AIza..."
model   = "gemini-3-flash-preview"
```

### AWS Bedrock

```toml
[ai.bedrock]
region  = "us-east-1"
model   = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"   # optional — uses default credential chain if omitted
```

{: .warning }
> AI providers receive your query and a portion of the JSON schema/sample. For sensitive payloads, use a local model via Ollama or LM Studio.

---

## Full example

```toml
[clipboard]
backend = "auto"

[autocomplete]
array_sample_size = 25

[ai]
enabled            = true
provider           = "anthropic"
max_context_length = 80000

[ai.anthropic]
api_key = "sk-ant-..."
model   = "claude-haiku-4-5-20251001"
```

---

## Environment overrides

| Variable | Effect |
|:---|:---|
| `JIQ_DEBUG=1` | Same as `--debug`: write debug logs to `/tmp/jiq-debug.log`. |

See [Troubleshooting](./troubleshooting).
