---
title: AI assistant
parent: Features
nav_order: 3
description: Natural-language jq queries, error fixes, and follow-ups from Anthropic, OpenAI, Gemini, AWS Bedrock, or any OpenAI-compatible endpoint.
---

# AI assistant

<kbd>Ctrl</kbd>+<kbd>A</kbd> sends the current query, a sample of the loaded JSON, and any active error to the configured model. Returns 3–5 jq suggestions; one keystroke applies any of them.

Three suggestion modes, picked from current state:

- `fix` — when the query has a syntax or runtime error (`select(.active = true)` → `select(.active == true)`).
- `optimize` — when the query is valid; proposes shorter or clearer alternatives.
- `next` — follow-up queries based on the result shape (filter, sort, project a subset, …).

<div class="tui-mockup with-title" data-title="Ctrl+A — AI assistant">
<pre>╭─ AI Assistant ─────────────────────────────────────╮
│ Query: show active users with their emails         │
│                                                    │
│ ▸ 1. .users[] | select(.active) | .email           │
│   2. [.users[] | select(.active) | .email]         │
│   3. .users | map(select(.active) | .email)        │
│   4. .users[] | select(.active == true).email      │
│   5. .users[] | select(.active) | {name, email}    │
│                                                    │
│ Alt+1-5 apply · Alt+↑↓ navigate · Enter apply      │
╰────────────────────────────────────────────────────╯</pre>
</div>

The popup anchors above the input. Suggestions stream in as the model responds.

For non-ASCII keys, suggestions always emit bracket notation (`.["名前"]`), regardless of how the model phrases the query — jiq enforces this client-side so you never end up with a syntactically invalid suggestion.

## Shortcuts
{: .shortcuts }

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Toggle the popup |
| <kbd>Alt</kbd>+<kbd>1</kbd> … <kbd>Alt</kbd>+<kbd>5</kbd> | Apply suggestion 1–5 |
| <kbd>Alt</kbd>+<kbd>↑</kbd> / <kbd>Alt</kbd>+<kbd>↓</kbd> | Navigate |
| <kbd>Alt</kbd>+<kbd>j</kbd> / <kbd>Alt</kbd>+<kbd>k</kbd> | Navigate (vim) |
| <kbd>Enter</kbd> | Apply highlighted |
| <kbd>Esc</kbd> | Close |

## Configuration

Opt-in. Add an `[ai]` section to `~/.config/jiq/config.toml` plus a provider block.

```toml
[ai]
enabled = true
provider = "anthropic"          # or "openai" | "gemini" | "bedrock"
max_context_length = 100000     # JSON sample chars sent per request (default 100000)
```

`max_context_length` caps how much of your data is sent to the model. Larger values give better suggestions on complex shapes; smaller values mean fewer tokens and lower cost.

### Anthropic

```toml
[ai.anthropic]
api_key = "sk-ant-..."
model   = "claude-haiku-4-5-20251001"
# max_tokens = 512   # response cap, default 512
```

[Get an API key](https://console.anthropic.com/settings/keys).

### OpenAI

```toml
[ai.openai]
api_key = "sk-proj-..."
model   = "gpt-4o-mini"
```

[Get an API key](https://platform.openai.com/api-keys).

### OpenAI-compatible (Ollama, LM Studio, x.ai, …)

Set `provider = "openai"` and override `base_url`. API key optional when the endpoint doesn't require one.

```toml
# Ollama (local)
[ai.openai]
base_url = "http://localhost:11434/v1"
model    = "llama3"

# LM Studio (local)
[ai.openai]
base_url = "http://localhost:1234/v1"
model    = "local-model"

# x.ai Grok
[ai.openai]
base_url = "https://api.x.ai/v1"
api_key  = "xai-..."
model    = "grok-4-fast-non-reasoning"
```

### Gemini

```toml
[ai.gemini]
api_key = "AIza..."
model   = "gemini-3-flash-preview"
```

[Get an API key](https://aistudio.google.com/apikey).

### AWS Bedrock

```toml
[ai.bedrock]
region  = "us-east-1"
model   = "anthropic.claude-3-haiku-20240307-v1:0"
profile = "default"   # optional; falls back to default credential chain
```

> Privacy: every request includes your query, an active error message (if any), and a JSON sample truncated at `max_context_length`. For sensitive data, run a local model via Ollama or LM Studio.
