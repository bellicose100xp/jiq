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
    <p class="feature-card-desc">Move the cursor to any row and press one key to zoom into that value — jiq rewrites the query for you. Step back out the same way.</p>
  </a>
  <a class="feature-card" href="./features/autocomplete/">
    <span class="feature-card-title">Autocomplete</span>
    <p class="feature-card-desc">Suggestions appear as you type a field path, pulled from your actual JSON. Each field shows its type. Tab to accept.</p>
  </a>
  <a class="feature-card" href="./features/ai-assistant/">
    <span class="feature-card-title">AI assistant</span>
    <p class="feature-card-desc">Press one shortcut and the AI returns 2–5 query suggestions based on your data and current error. Apply any of them instantly.</p>
  </a>
  <a class="feature-card" href="./features/snippets/">
    <span class="feature-card-title">Snippet library</span>
    <p class="feature-card-desc">Save your most-used queries by name. Open the library, type to filter the list, press Enter to run. Persists across sessions.</p>
  </a>
  <a class="feature-card" href="./features/search/">
    <span class="feature-card-title">Search in results</span>
    <p class="feature-card-desc">Search the rendered output as you type. Matches highlight in real time; n/N steps between them. Works across thousands of lines.</p>
  </a>
  <a class="feature-card" href="./features/history/">
    <span class="feature-card-title">Query history</span>
    <p class="feature-card-desc">Every query that produced output is saved automatically. Cycle through recent ones inline, or open a popup to search your full history.</p>
  </a>
  <a class="feature-card" href="./features/vim-editing/">
    <span class="feature-card-title">Vim editing</span>
    <p class="feature-card-desc">The query input supports Vim keybindings — move by word, delete with operators, undo and redo. INSERT mode works like a normal text field if you skip this.</p>
  </a>
  <a class="feature-card" href="./features/clipboard/">
    <span class="feature-card-title">Clipboard &amp; paste</span>
    <p class="feature-card-desc">Run <code>jiq</code> with no file and it reads JSON straight from your clipboard. If the clipboard is empty or invalid, a paste box opens instead of failing.</p>
  </a>
  <a class="feature-card" href="./features/mouse/">
    <span class="feature-card-title">Mouse support</span>
    <p class="feature-card-desc">Click to focus any pane, scroll to navigate, drag to select lines in the results. Click a suggestion or history entry to apply it.</p>
  </a>
  <a class="feature-card" href="./features/tooltip/">
    <span class="feature-card-title">Tooltip &amp; overlays</span>
    <p class="feature-card-desc">Cursor on a jq function? A tooltip shows its signature and examples. Ctrl+E shows the full error message. F1 opens the keybind reference.</p>
  </a>
</div>
