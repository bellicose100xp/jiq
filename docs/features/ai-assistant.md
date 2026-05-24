---
title: AI assistant
parent: Features
nav_order: 3
description: Get AI-generated query suggestions when you're stuck, have an error, or want to go further.
---

# AI assistant

When you're not sure how to write a query, press **Ctrl+A**. jiq sends your current query, any active error, and a sample of your JSON to the configured model, then shows 2–5 suggestions you can apply instantly.

The assistant works in three situations:

- **Error in the query** — suggests corrected versions (`select(.active = true)` → `select(.active == true)`)
- **Valid query** — proposes shorter or clearer alternatives
- **After a result** — suggests follow-up queries based on what you're looking at

<div class="tui-mockup with-title" data-title="AI assistant — Ctrl+A">
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

## Apply a suggestion

With the popup open:

- Press **Alt+1** through **Alt+5** to apply that numbered suggestion directly.
- Or use **Alt+↑** / **Alt+↓** to highlight a suggestion, then press **Enter** to apply it.

Applying a suggestion replaces your current query and re-runs it immediately.

## Close the popup

Press **Ctrl+A** again or **Esc** to close without applying anything.

## Set up a provider

The AI assistant is off by default. To enable it, add an `[ai]` section and a provider block to `~/.config/jiq/config.toml`.

### Anthropic (Claude)

```toml
[ai]
enabled  = true
provider = "anthropic"

[ai.anthropic]
api_key = "sk-ant-..."
model   = "claude-haiku-4-5-20251001"
```

[Get an Anthropic API key →](https://console.anthropic.com/settings/keys)

### OpenAI

```toml
[ai]
enabled  = true
provider = "openai"

[ai.openai]
api_key = "sk-proj-..."
model   = "gpt-4o-mini"
```

[Get an OpenAI API key →](https://platform.openai.com/api-keys)

### Gemini

```toml
[ai]
enabled  = true
provider = "gemini"

[ai.gemini]
api_key = "AIza..."
model   = "gemini-3-flash-preview"
```

[Get a Gemini API key →](https://aistudio.google.com/apikey)

### AWS Bedrock

```toml
[ai]
enabled  = true
provider = "bedrock"

[ai.bedrock]
region  = "us-east-1"
model   = "anthropic.claude-3-haiku-20240307-v1:0"
profile = "default"   # optional; uses default credential chain if omitted
```

### Local models (Ollama, LM Studio)

Set `provider = "openai"` and point `base_url` at your local server. No API key needed.

```toml
[ai]
enabled  = true
provider = "openai"

# Ollama
[ai.openai]
base_url = "http://localhost:11434/v1"
model    = "llama3"

# LM Studio
[ai.openai]
base_url = "http://localhost:1234/v1"
model    = "local-model"
```

## Limit how much data is sent

By default jiq sends up to 100,000 characters of your JSON as context. Reduce this if you're working with sensitive data or want to lower API costs:

```toml
[ai]
max_context_length = 20000   # characters; default 100000
```

For sensitive data, a local model via Ollama or LM Studio sends nothing outside your machine.

## All keys

| Key | Action |
|---|---|
| `Ctrl+A` | Open or close the popup |
| `Alt+1` … `Alt+5` | Apply suggestion 1–5 |
| `Alt+↑` / `Alt+↓` | Move through the list |
| `Alt+j` / `Alt+k` | Move through the list (Vim) |
| `Enter` | Apply highlighted suggestion |
| `Esc` | Close without applying |
