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

**macOS**
```bash
brew install bellicose100xp/tap/jiq
```

**macOS / Linux**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

**Any platform with Rust**
```bash
cargo install jiq
```

**Windows / others:** [pre-built binaries](https://github.com/bellicose100xp/jiq/releases/latest)

Then:
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
    <p class="feature-card-desc">Exploring a deeply nested JSON response? Navigate the output with j/k, press &gt; on any row to filter down to just that piece of data. No path typing needed; &lt; takes you back.</p>
  </a>
  <a class="feature-card" href="./features/autocomplete/">
    <span class="feature-card-title">Autocomplete</span>
    <p class="feature-card-desc">Don't know your JSON's field names off the top of your head? jiq suggests them as you type — pulled from your actual data, with the value type shown alongside.</p>
  </a>
  <a class="feature-card" href="./features/ai-assistant/">
    <span class="feature-card-title">AI assistant</span>
    <p class="feature-card-desc">Don't know the right jq syntax? Press Ctrl+A and the AI sees your query, the error, and a sample of your data — then suggests 2–5 fixes you can apply with one key.</p>
  </a>
  <a class="feature-card" href="./features/snippets/">
    <span class="feature-card-title">Snippet library</span>
    <p class="feature-card-desc">Got a jq query you run all the time? Save it by name. One shortcut opens your library; type to filter, Enter to run. Never retype the same query twice.</p>
  </a>
  <a class="feature-card" href="./features/search/">
    <span class="feature-card-title">Search in results</span>
    <p class="feature-card-desc">Looking for a specific value buried in hundreds of lines of output? Ctrl+F highlights every match as you type. n/N jumps between them.</p>
  </a>
  <a class="feature-card" href="./features/history/">
    <span class="feature-card-title">Query history</span>
    <p class="feature-card-desc">Ran the right query ten queries ago and want it back? Every query is saved. Ctrl+P cycles backward; Ctrl+R opens a searchable list of everything you've run.</p>
  </a>
  <a class="feature-card" href="./features/vim-editing/">
    <span class="feature-card-title">Vim editing</span>
    <p class="feature-card-desc">Know Vim? Every motion and operator works in the query input. Don't know Vim? INSERT mode is just a regular text field — nothing to learn.</p>
  </a>
  <a class="feature-card" href="./features/clipboard/">
    <span class="feature-card-title">Clipboard &amp; paste</span>
    <p class="feature-card-desc">Copied some JSON from a browser or terminal? Just run <code>jiq</code> — it reads your clipboard automatically. If the JSON is malformed, a paste box opens so you can fix it first.</p>
  </a>
  <a class="feature-card" href="./features/mouse/">
    <span class="feature-card-title">Mouse support</span>
    <p class="feature-card-desc">Prefer the mouse for some things? Every pane responds — click to focus, scroll to browse, drag to select output lines. Keyboard and mouse work together throughout.</p>
  </a>
  <a class="feature-card" href="./features/tooltip/">
    <span class="feature-card-title">Tooltip &amp; overlays</span>
    <p class="feature-card-desc">Forgotten how <code>group_by</code> works? Put the cursor on any jq function and a tooltip shows its signature and examples inline. Ctrl+E shows the full error when a query fails.</p>
  </a>
</div>
