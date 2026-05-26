---
title: AI assistant
parent: Features
nav_order: 3
description: Get context-aware jq query suggestions from an AI that sees your data, your query, and the error.
---

# AI assistant

The AI assistant fixes broken queries for you — it sees what went wrong, understands your data shape, and offers working alternatives you apply with a single keystroke.

<div class="before-after">
  <input type="radio" name="ba-ai" id="ba-ai-before" checked>
  <input type="radio" name="ba-ai" id="ba-ai-after">
  <div class="ba-header">
    <label for="ba-ai-before" class="ba-toggle">Without AI</label>
    <label for="ba-ai-after" class="ba-toggle">With AI</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You get a syntax error. Now you're searching the web, reading jq docs, trying variations one by one.</p>
    <div class="ba-terminal">$ jiq data.json
Query: .users | group_by .role
       ^^^^^^^^^^^^^^^^^^^^^^^^^^
       Syntax Error

# Tab to browser...
# Search: "jq group_by syntax"
# Read docs... try group_by(.role)... no wait...
# Try 5 more variations...
# 10 minutes later: finally works</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Press Ctrl+A. The AI sees your query, the error, and your data — then offers working fixes.</p>
    <div class="ba-terminal">Query: .users | group_by .role
       Syntax Error

# Press Ctrl+A...

AI Suggestions:
  1. .users | group_by(.role)
  2. [.users[] | group_by(.role)]
  3. .users | group_by(.role) | map({key: .[0].role, val: .})

# Press Alt+1 — done. 3 seconds.</div>
  </div>
</div>

## How it works

<div class="step-flow">
  <div class="step-item done">
    <div class="step-circle">1</div>
    <div class="step-text">Error appears</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item done">
    <div class="step-circle">2</div>
    <div class="step-text">Press Ctrl+A</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item active">
    <div class="step-circle">3</div>
    <div class="step-text">AI analyzes context</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item">
    <div class="step-circle">4</div>
    <div class="step-text">Pick a suggestion</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item">
    <div class="step-circle">5</div>
    <div class="step-text">Applied</div>
  </div>
</div>

The AI sends your current query, the error message, and a sample of your JSON to the configured provider. It returns 2-5 suggestions ranked by relevance. The entire round trip typically takes 1-3 seconds.

## Get a fix for a failing query

1. Write a query that produces an error (the `Syntax Error` banner appears).
2. Press **Ctrl+A** to open the AI popup.
3. Wait for suggestions to appear (a loading indicator shows progress).
4. Press **Alt+1** through **Alt+5** to apply a suggestion directly — or use **Alt+j**/**Alt+k** to navigate, then **Enter** to apply.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">AI suggestions popup</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-error">.items | map(select(.price > 100) | .name, .price)</span></div>
    <div class="term-line"><span class="term-error">Syntax Error: unexpected ',' at line 1</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">AI Suggestions:</span></div>
    <div class="term-line"><span class="term-highlight"> 1.</span> <span class="term-output">.items[] | select(.price > 100) | {name, price}</span></div>
    <div class="term-line"><span class="term-highlight"> 2.</span> <span class="term-output">[.items[] | select(.price > 100) | {name: .name, price: .price}]</span></div>
    <div class="term-line"><span class="term-highlight"> 3.</span> <span class="term-output">.items | map(select(.price > 100) | {name, price})</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Alt+1..3 Apply  |  Alt+j/k Navigate  |  Enter Apply selected</span></div>
  </div>
</div>

## Ask for help with a working query

The AI assistant is not limited to fixing errors. Even when your query works, press **Ctrl+A** and the AI may suggest improvements — a more concise form, a different approach, or natural language interpretation of what you typed.

## Navigate and dismiss suggestions

| Action | Key |
|---|---|
| Move between suggestions | **Alt+Up** / **Alt+Down** or **Alt+j** / **Alt+k** |
| Apply the highlighted suggestion | **Enter** |
| Apply suggestion N directly | **Alt+1** through **Alt+5** |
| Close without applying | **Ctrl+A** or **Esc** |

## Configure the AI provider

The AI assistant requires a provider configuration in `~/.config/jiq/config.toml`. jiq supports Anthropic, OpenAI, Gemini, AWS Bedrock, and any OpenAI-compatible API.

```toml
[ai]
enabled = true
provider = "anthropic"    # "anthropic", "openai", "gemini", or "bedrock"
max_context_length = 100000  # characters of JSON context sent to AI (default 100k)
```

### Anthropic

```toml
[ai.anthropic]
api_key = "sk-ant-..."
model = "claude-haiku-4-5-20251001"
```

### OpenAI

```toml
[ai.openai]
api_key = "sk-proj-..."
model = "gpt-4o-mini"
```

### Gemini

```toml
[ai.gemini]
api_key = "AIza..."
model = "gemini-3-flash-preview"
```

### AWS Bedrock

```toml
[ai.bedrock]
region = "us-east-1"
model = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"  # optional: uses default credential chain if omitted
```

### OpenAI-compatible APIs (Ollama, LM Studio, x.ai)

Any API that follows the OpenAI chat completions format works by setting `provider = "openai"` with a custom `base_url`:

```toml
# Ollama (local)
[ai.openai]
base_url = "http://localhost:11434/v1"
model = "llama3"

# LM Studio (local)
[ai.openai]
base_url = "http://localhost:1234/v1"
model = "local-model"

# x.ai Grok
[ai.openai]
api_key = "your-xai-api-key"
base_url = "https://api.x.ai/v1"
model = "grok-4-fast-non-reasoning"
```

For local providers that don't require authentication, omit the `api_key` field entirely.

### Tuning context size

The `max_context_length` setting controls how much of your JSON data is sent to the AI. Larger values give the AI more context for better suggestions but increase token usage and cost. Smaller values reduce cost and latency.

```toml
[ai]
max_context_length = 50000   # send less context (faster, cheaper)
max_context_length = 200000  # send more context (better suggestions for large files)
```

For sensitive data, a local model via Ollama or LM Studio keeps everything on your machine.

## All keys

| Key | Action |
|---|---|
| `Ctrl+A` | Toggle AI assistant popup |
| `Alt+1`..`Alt+5` | Apply suggestion 1-5 directly |
| `Alt+Up` / `Alt+Down` | Navigate suggestions |
| `Alt+j` / `Alt+k` | Navigate suggestions (vim style) |
| `Enter` | Apply selected suggestion |
| `Ctrl+A` / `Esc` | Close popup |
