---
title: Home
layout: home
nav_order: 1
description: "jiq — Interactive JSON query tool with real-time output. Type a jq query, see results live, drill into nested values with a single keystroke."
permalink: /
---

<div class="hero" markdown="0">
  <span class="hero-eyebrow">v3.26 · TUI</span>
  <h1 class="hero-title">Build jq queries, live.</h1>
  <p class="hero-tagline">Type a query. See results as you type. Drill into nested values with a single keystroke.</p>
  <div class="hero-actions">
    <a class="btn btn-primary fs-5" href="./getting-started/">Get started</a>
    <a class="btn fs-5" href="./quick-reference/">Quick reference</a>
    <a class="btn fs-5" href="https://github.com/bellicose100xp/jiq">GitHub</a>
  </div>
</div>

## Features

<div class="feature-grid" markdown="1">

<div class="feature-card" markdown="1">
### [Path-at-cursor](./features/path-at-cursor)
Drill in, step back, iterate, walk parents with `>` `<` `*` `^` `}`.
</div>

<div class="feature-card" markdown="1">
### [Autocomplete](./features/autocomplete)
Schema-aware fields and functions with type hints.
</div>

<div class="feature-card" markdown="1">
### [AI assistant](./features/ai-assistant)
Natural-language queries and error fixes from Claude, GPT, Gemini, Bedrock, or local models.
</div>

<div class="feature-card" markdown="1">
### [Snippets](./features/snippets)
Save and reuse queries across sessions.
</div>

<div class="feature-card" markdown="1">
### [Search](./features/search)
Find and step through matches in the output pane.
</div>

<div class="feature-card" markdown="1">
### [History](./features/history)
Searchable history of every successful query.
</div>

<div class="feature-card" markdown="1">
### [Mouse](./features/mouse)
Click, scroll, drag-select, click suggestions.
</div>

<div class="feature-card" markdown="1">
### [Clipboard & paste](./features/clipboard)
Auto-load from the clipboard; in-app paste box on failure.
</div>

<div class="feature-card" markdown="1">
### [VIM editing](./features/vim-editing)
Motions, operators, text objects, undo/redo.
</div>

<div class="feature-card" markdown="1">
### [Results pane](./features/results-pane)
Cursor, scroll, visual line selection.
</div>

<div class="feature-card" markdown="1">
### [Tooltip & overlays](./features/tooltip)
Function help, error overlay, in-app keybind reference.
</div>

<div class="feature-card" markdown="1">
### [Configuration](./configuration)
Clipboard backend, AI providers, autocomplete depth.
</div>

</div>

## Launch

```bash
jiq data.json                     # from a file
curl https://api.example.com | jiq # from stdin
jiq                                # from clipboard, falls back to paste box
```

When run with no file and no piped input, jiq reads JSON from the clipboard. If the clipboard is empty or not valid JSON, it opens an in-app paste box — paste your JSON, press <kbd>Enter</kbd>, and you're in.

[Install →](./getting-started#install)
