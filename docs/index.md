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

```bash
brew install bellicose100xp/tap/jiq
# or
cargo install jiq
# or
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

```bash
jiq data.json              # from a file
curl -s api/data | jiq     # from stdin
jiq                        # from clipboard, with paste-box fallback
```

## Features

![jiq demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-v3.20.gif)

<ul class="feature-list" markdown="0">
  <li><a href="./features/results-pane/">Results pane</a> <span class="desc">— cursor, drill chords <code>&gt;</code> <code>&lt;</code> <code>*</code> <code>^</code> <code>}</code>, visual selection</span></li>
  <li><a href="./features/autocomplete/">Autocomplete</a> <span class="desc">— schema-aware, with type hints</span></li>
  <li><a href="./features/ai-assistant/">AI assistant</a> <span class="desc">— Claude, GPT, Gemini, Bedrock, or local models</span></li>
  <li><a href="./features/snippets/">Snippets</a> <span class="desc">— save and reuse jq queries</span></li>
  <li><a href="./features/search/">Search</a> <span class="desc">— find and step through matches</span></li>
  <li><a href="./features/history/">History</a> <span class="desc">— searchable query history</span></li>
  <li><a href="./features/vim-editing/">VIM editing</a> <span class="desc">— motions, operators, text objects</span></li>
  <li><a href="./features/clipboard/">Clipboard &amp; paste</a> <span class="desc">— auto-load on launch, paste-box fallback</span></li>
  <li><a href="./features/mouse/">Mouse</a> <span class="desc">— click, scroll, drag-select</span></li>
</ul>

[All features →](./features/)
