---
title: Configuration
nav_order: 5
description: config.toml reference — clipboard backend, autocomplete depth, AI providers.
---

# Configuration

`config.toml` reference covering clipboard, autocomplete, and AI.

## Location

Optional. Defaults are reasonable; only AI requires config.

| OS | Path |
|:---|:---|
| Linux | `~/.config/jiq/config.toml` |
| macOS | `~/Library/Application Support/jiq/config.toml` |
| Windows | `%APPDATA%\jiq\config.toml` |

## Clipboard

```toml
[clipboard]
backend = "auto"
```

| Value | Use when |
|:---|:---|
| `auto` (default) | OS clipboard first, OSC 52 fallback. |
| `system` | Local desktop only. |
| `osc52` | SSH / tmux / mosh on a terminal that forwards OSC 52 (Ghostty, kitty, WezTerm, foot). |

## Autocomplete

```toml
[autocomplete]
array_sample_size = 10
```

For heterogeneous arrays, increase to union more keys (range: 1–1000).

## AI

```toml
[ai]
enabled            = true
provider           = "anthropic"   # "openai" | "gemini" | "bedrock"
max_context_length = 100000
```

| Provider | Recommended model |
|:---|:---|
| Anthropic | `claude-haiku-4-5-20251001` |
| OpenAI | `gpt-4o-mini` |
| Gemini | `gemini-3-flash` |

See [AI assistant](./features/ai-assistant) for per-provider config.

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

## Environment overrides

| Variable | Effect |
|:---|:---|
| `JIQ_DEBUG=1` | Same as `--debug`: write debug logs to `/tmp/jiq-debug.log`. |

See [Troubleshooting](./troubleshooting).
