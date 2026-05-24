---
title: Home
layout: home
nav_order: 1
description: jiq — a TUI for jq. Type a query, see results live.
permalink: /
---

# jiq

<p class="tagline">A TUI for <code>jq</code>. Type a query, see results live, drill into nested values with a single keystroke.</p>

[Get started](./getting-started){: .btn .btn-primary .mr-2 } [Quick reference](./quick-reference){: .btn .mr-2 } [GitHub](https://github.com/bellicose100xp/jiq){: .btn }

---

## Quick start

Requires [`jq`](https://jqlang.org/download/) on `$PATH`.

| Platform | Install |
|---|---|
| **macOS** | `brew install bellicose100xp/tap/jiq` |
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh \| sh` |
| **Any (with Rust)** | `cargo install jiq` |
| **Windows / others** | [Pre-built binaries](https://github.com/bellicose100xp/jiq/releases/latest) |

```bash
jiq data.json        # file
curl -s api | jiq    # stdin
jiq                  # clipboard, with paste-box fallback
```

## See it in action

![jiq demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-v3.20.gif)

## Features

<div class="feature-grid" markdown="0">
  <a class="feature-card" href="./features/results-pane/">
    <span class="feature-card-title">Results pane</span>
    <p class="feature-card-desc">Cursor navigation, drill chords <code>&gt; &lt; * ^ }</code>, and visual line selection across jq output.</p>
  </a>
  <a class="feature-card" href="./features/autocomplete/">
    <span class="feature-card-title">Autocomplete</span>
    <p class="feature-card-desc">Schema-aware field and function suggestions with JSON type hints — drawn from your actual data.</p>
  </a>
  <a class="feature-card" href="./features/ai-assistant/">
    <span class="feature-card-title">AI assistant</span>
    <p class="feature-card-desc">Fix errors, optimize queries, or ask for next steps. Works with Claude, GPT, Gemini, Bedrock, or local models.</p>
  </a>
  <a class="feature-card" href="./features/snippets/">
    <span class="feature-card-title">Snippet library</span>
    <p class="feature-card-desc">Save and reuse named jq queries across sessions. Fuzzy search to find and apply instantly.</p>
  </a>
  <a class="feature-card" href="./features/search/">
    <span class="feature-card-title">Search in results</span>
    <p class="feature-card-desc">Case-insensitive full-text search across rendered output, with live highlighting and match counter.</p>
  </a>
  <a class="feature-card" href="./features/history/">
    <span class="feature-card-title">Query history</span>
    <p class="feature-card-desc">Every successful query is saved and searchable. Cycle inline or browse a fuzzy-filtered popup.</p>
  </a>
  <a class="feature-card" href="./features/vim-editing/">
    <span class="feature-card-title">VIM editing</span>
    <p class="feature-card-desc">Full motions, operators, text objects, and undo/redo in the query input — including a jq-aware pipe segment.</p>
  </a>
  <a class="feature-card" href="./features/clipboard/">
    <span class="feature-card-title">Clipboard &amp; paste</span>
    <p class="feature-card-desc">Auto-loads JSON from clipboard on launch. Paste-box fallback when clipboard is empty or invalid.</p>
  </a>
  <a class="feature-card" href="./features/mouse/">
    <span class="feature-card-title">Mouse</span>
    <p class="feature-card-desc">Click to focus, scroll any pane, drag-select in results, click suggestions and history entries.</p>
  </a>
  <a class="feature-card" href="./features/tooltip/">
    <span class="feature-card-title">Tooltip &amp; overlays</span>
    <p class="feature-card-desc">Function tooltip, full error overlay, and a tabbed help popup covering every keybind.</p>
  </a>
</div>
