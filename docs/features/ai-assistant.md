---
title: AI assistant
parent: Features
nav_order: 3
description: Get context-aware jq query suggestions from an AI that sees your data, your query, and the error, then ask it follow-up questions in the same popup.
---

# AI assistant

The AI assistant fixes broken queries for you — it sees what went wrong, understands your data shape, and offers working alternatives you apply with a single keystroke. The same popup is a conversation: type a question, get an answer plus applyable queries, and ask follow-ups that build on what came before.

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

The AI sends your current query, the error message, and a sample of your JSON to the configured provider. It returns up to 5 suggestions ranked by relevance. The entire round trip typically takes 1-3 seconds.

## Get a fix for a failing query

1. Write a query that produces an error (the `Syntax Error` banner appears).
2. Press **Ctrl+A** to open the AI popup.
3. Wait for suggestions to appear (a loading indicator shows progress).
4. Press **Alt+1** through **Alt+5** to apply a suggestion directly — or use **Alt+j**/**Alt+k** to navigate, then **Enter** to apply.

Opening the popup puts the cursor in its chat input. Press **Esc** to go back to editing the query; the popup stays open and keeps suggesting as you type.

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

The AI assistant is not limited to fixing errors. Even when your query works, press **Ctrl+A** and the AI may suggest optimizations — a more concise form, a more idiomatic construct, or a more robust one. When the query is already as good as it gets, the popup says **No suggestions**.

## Ask a question

The bottom row of the popup is a chat input. Type there instead of in the query box when you want to talk about the data or the query rather than edit it.

1. Press **Ctrl+A**. The popup opens with the chat input focused.
2. Type a question and press **Enter**. Anything goes: "why is this empty?", "how do I group these by role?", "explain suggestion 2", "what's the difference between map and .[]?".
3. The answer appears as a short prose reply, followed by numbered queries when a query answers the question. Apply one with **Alt+1**..**Alt+5** exactly like a fix.
4. Ask a follow-up. The AI remembers the conversation, so "now only the active ones" or "make that a single object" works without restating the goal.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">AI popup, conversation</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">❯ how do I count users per role?</span></div>
    <div class="term-line"><span class="term-dim">group_by collects users sharing a role; length counts each group.</span></div>
    <div class="term-line"><span class="term-dim">  [Query] .users | group_by(.role) | map({role: .[0].role, count: length})</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-highlight">❯ only active users</span></div>
    <div class="term-line"><span class="term-output">Filter before grouping so inactive users never reach the count.</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-highlight"> 1.</span> <span class="term-output">[Query] .users | map(select(.active)) | group_by(.role) | map({role: .[0].role, count: length})</span></div>
    <div class="term-line"><span class="term-dim">   Keeps active users, then counts per role</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">──────────────────────────────────────────────────</span></div>
    <div class="term-line"><span class="term-highlight">› </span><span class="term-dim">Ask about this query or data…</span></div>
  </div>
</div>

Every request carries the current query, its output or error, a sample of the data, the suggestions on screen, and the earlier questions and answers. Suggestions triggered by editing the query use the same conversation, so once you have explained what you are after, the fixes and optimizations it proposes take that into account.

Earlier exchanges stay visible above the current one, dimmed. Scroll them with **↑**/**↓** or **PgUp**/**PgDn** while the chat input is focused, or with the mouse wheel. **Ctrl+L** forgets the conversation and starts fresh. The last 12 exchanges are sent with each request; older ones drop off.

## What the popup tells you

| Popup state | What it means |
|---|---|
| A numbered list of suggestions | The AI returned up to 5 jq queries you can apply. `[Fix]` corrects an error, `[Optimize]` improves a working query, `[Query]` does what you asked for in chat. |
| Prose above the list | The answer to your question. |
| **No suggestions** | The AI ran successfully but had nothing useful to add for this query (common for the bare `.` identity query). This is normal, not an error. |
| **Could not parse AI response** | The provider returned a response jiq could not read. Re-run with `--debug` and check `/tmp/jiq-debug.log` to see the raw response. |

## Navigate and dismiss suggestions

| Action | Key |
|---|---|
| Move between suggestions | **Alt+Up** / **Alt+Down** or **Alt+j** / **Alt+k** |
| Apply the highlighted suggestion | **Enter** (after navigating with Alt+Up/Down) |
| Apply suggestion N directly | **Alt+1** through **Alt+5** |
| Send the typed question | **Enter** (chat input focused) |
| Back to the query box, popup stays open | **Esc** |
| Focus the chat input again | click its row, or **Ctrl+A** twice |
| Clear the conversation | **Ctrl+L** (chat input focused) |
| Close the popup | **Ctrl+A** |

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
effort = "high"      # optional: low | medium | high | xhigh | max (Claude 4.6+)
context_1m = false   # optional: 1M-token context window beta (Claude Sonnet 4/4.5)
```

`effort` sets the reasoning depth on Claude models that support it. `context_1m` opts into the 1M-token context window — raise [`max_context_length`](#tuning-context-size) too, or jiq still sends the same small sample.

### OpenAI

```toml
[ai.openai]
api_key = "sk-proj-..."
model = "gpt-4o-mini"
effort = "medium"    # optional: minimal | low | medium | high | xhigh | max (reasoning models)
```

`effort` maps to the OpenAI `reasoning_effort` field and is only sent when set.

### Gemini

```toml
[ai.gemini]
api_key = "AIza..."
model = "gemini-3-flash-preview"
effort = "medium"    # optional: minimal | low | medium | high (Gemini 3+; xhigh/max clamp to high)
```

`effort` maps to Gemini's `thinkingLevel`. Gemini 2.5-series models use a different mechanism and reject it — omit `effort` on those.

### AWS Bedrock

```toml
[ai.bedrock]
region = "us-east-1"
model = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"  # optional: uses default credential chain if omitted
effort = "high"      # optional: low | medium | high | xhigh | max (Claude Sonnet/Opus 4.6+)
context_1m = false   # optional: enable the 1M-token context window (Claude Sonnet 4/4.5)
```

`effort` sets the reasoning depth for Claude models that support it (Sonnet/Opus 4.6 and newer). Omit it to use the model default. Setting it on a model without reasoning support makes Bedrock reject the request.

`context_1m` opts into the 1M-token context window (Claude Sonnet 4 and 4.5; newer Sonnet models already default to 1M). On its own it only lifts the ceiling — raise [`max_context_length`](#tuning-context-size) too, or jiq still sends the same small sample. Prompts over 200K tokens are billed at a higher rate.

### OpenAI models on Bedrock (gpt-oss, GPT-5.x)

Two routes, depending on how you want to authenticate:

**AWS credentials / profile (Converse).** OpenAI models on Bedrock — gpt-oss and GPT-5.6 (Sol/Terra/Luna) — support the Converse API, so the regular `bedrock` provider works with the same region/profile setup as Claude. jiq sends `effort` as the OpenAI `reasoning_effort` field automatically when the model ID contains `openai.`:

```toml
[ai.bedrock]
region = "us-east-1"
profile = "my-profile"
model = "us.openai.gpt-5.6-sol"   # or "openai.gpt-oss-120b-1:0"
effort = "low"       # gpt-oss: low | medium | high; GPT-5.6 also takes minimal/xhigh/max
```

**Bedrock API key (OpenAI-compatible endpoint).** GPT-5.x models are served through Bedrock's OpenAI-compatible endpoint. Point the `openai` provider at it with a Bedrock API key (generate one in the Bedrock console under API keys, or mint a short-term one from AWS credentials with the `aws-bedrock-token-generator` package):

```toml
[ai]
enabled = true
provider = "openai"

[ai.openai]
api_key = "your-bedrock-api-key"
base_url = "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1"
model = "openai.gpt-5.6-sol"
effort = "medium"    # optional: minimal | low | medium | high | xhigh | max
```

`effort` maps to the OpenAI `reasoning_effort` field. Omit it to use the model default.

### Extra instructions

`extra_instructions` appends your own guidance to every AI prompt — style preferences, house conventions, favored jq idioms, the tone of chat answers. It never replaces the built-in prompt: the output-format contract jiq's suggestion parser depends on always takes precedence.

```toml
[ai]
extra_instructions = "Prefer map() over .[] pipelines. Keep suggestions POSIX-shell safe."
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

These endpoints share the `[ai.openai]` options, including `effort` — jiq only sends `reasoning_effort` when you set it, so servers that don't support the field are unaffected. Set it only for models that take it (e.g. gpt-oss on Ollama).

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
| `Ctrl+A` | Open the popup with the chat input focused / close it |
| `Enter` | Send the typed question (chat input focused) |
| `Esc` | Back to the query box; popup stays open |
| `Ctrl+L` | Clear the conversation (chat input focused) |
| `↑` / `↓`, `PgUp` / `PgDn` | Scroll the conversation (chat input focused) |
| `Alt+1`..`Alt+5` | Apply suggestion 1-5 directly |
| `Alt+Up` / `Alt+Down` | Navigate suggestions |
| `Alt+j` / `Alt+k` | Navigate suggestions (vim style) |
| `Enter` | Apply selected suggestion (after Alt+Up/Down) |
