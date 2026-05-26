---
title: Home
layout: home
nav_order: 1
description: jiq — a TUI for jq. Type a query, see results live.
permalink: /
---

# jiq

<p class="tagline">An interactive terminal tool for <code>jq</code>. Type a query, see results live, navigate into nested values with a single keystroke.</p>

[Get started](./getting-started){: .btn .btn-primary .mr-2 } [Quick reference](./quick-reference){: .btn .mr-2 } [GitHub](https://github.com/bellicose100xp/jiq){: .btn }

<div class="hero-section">
<div class="animated-terminal hero-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">jiq data.json</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.users[] | select(.active) | .email</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Results:</span></div>
    <div class="term-line"><span class="term-output">"alice@example.com"</span></div>
    <div class="term-line"><span class="term-output">"carol@example.com"</span></div>
    <div class="term-line"><span class="term-output">"dave@example.com"</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-success">3 results</span> <span class="term-dim">in 4ms</span></div>
  </div>
</div>
</div>

---

## What do you want to do?

<div class="goal-grid" markdown="0">
  <a class="goal-card" href="./features/results-pane/">
    <div class="goal-scenario">Explore deeply nested JSON</div>
    <div class="goal-solution">Navigate with <code>j</code>/<code>k</code>, press <code>&gt;</code> to zoom in, <code>&lt;</code> to step back</div>
  </a>
  <a class="goal-card" href="./features/ai-assistant/">
    <div class="goal-scenario">Fix a broken jq query</div>
    <div class="goal-solution">Press <code>Ctrl+A</code> for AI suggestions that see your data and error</div>
  </a>
  <a class="goal-card" href="./features/search/">
    <div class="goal-scenario">Find a specific value in the output</div>
    <div class="goal-solution">Press <code>Ctrl+F</code>, type a term, hit <code>n</code>/<code>N</code> to jump between matches</div>
  </a>
  <a class="goal-card" href="./features/snippets/">
    <div class="goal-scenario">Reuse a query I wrote before</div>
    <div class="goal-solution">Save it once with <code>Ctrl+S</code>, recall instantly by name any time</div>
  </a>
  <a class="goal-card" href="./features/autocomplete/">
    <div class="goal-scenario">I don't know the field names</div>
    <div class="goal-solution">Type <code>.</code> and see every field with its type, pulled from your data</div>
  </a>
  <a class="goal-card" href="./features/vim-editing/">
    <div class="goal-scenario">Edit complex queries efficiently</div>
    <div class="goal-solution">Full Vim motions: <code>ciw</code>, <code>dt|</code>, <code>da"</code> to reshape in seconds</div>
  </a>
  <a class="goal-card" href="./features/results-pane/#select-and-copy-specific-lines">
    <div class="goal-scenario">Copy specific output lines</div>
    <div class="goal-solution">Press <code>v</code> to select lines, <code>y</code> to copy to clipboard</div>
  </a>
  <a class="goal-card" href="./features/clipboard/">
    <div class="goal-scenario">Work with clipboard JSON</div>
    <div class="goal-solution">Just run <code>jiq</code> with no arguments — it reads your clipboard directly</div>
  </a>
</div>

<hr class="accent-divider">

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
jiq                  # reads clipboard; opens a paste editor if needed
```

<hr class="accent-divider">

## All features

<div class="feature-grid" markdown="0">
  <a class="feature-card" href="./features/results-pane/">
    <span class="feature-card-title">Live query execution</span>
    <p class="feature-card-desc">Results update with every keystroke. No edit-run-check loop — you see exactly what your query produces as you type it, so you can iterate in seconds instead of minutes.</p>
  </a>
  <a class="feature-card" href="./features/autocomplete/">
    <span class="feature-card-title">Context-aware autocomplete</span>
    <p class="feature-card-desc">Suggests field names drawn from your actual data, with types shown alongside. No need to memorize your JSON structure — jiq shows you what's available at every level.</p>
  </a>
  <a class="feature-card" href="./features/ai-assistant/">
    <span class="feature-card-title">AI-powered query help</span>
    <p class="feature-card-desc">When a query fails or you're unsure of the syntax, the AI sees your data and the error, then offers working alternatives you can apply instantly.</p>
  </a>
  <a class="feature-card" href="./features/snippets/">
    <span class="feature-card-title">Saved query library</span>
    <p class="feature-card-desc">Store queries you use often and recall them by name. Stop rewriting the same complex filters from memory every time you need them.</p>
  </a>
  <a class="feature-card" href="./features/search/">
    <span class="feature-card-title">Search within output</span>
    <p class="feature-card-desc">Find specific values in large results instantly. Matches highlight in real time and you can step through them one by one — no manual scrolling required.</p>
  </a>
  <a class="feature-card" href="./features/history/">
    <span class="feature-card-title">Persistent query history</span>
    <p class="feature-card-desc">Every successful query is remembered across sessions. Recall any previous query by searching through your history — even ones from days ago.</p>
  </a>
  <a class="feature-card" href="./features/vim-editing/">
    <span class="feature-card-title">Efficient query editing</span>
    <p class="feature-card-desc">Reshape complex queries in a few keystrokes using Vim-style motions and text objects. Or just type normally — advanced editing is there when you need it, invisible when you don't.</p>
  </a>
  <a class="feature-card" href="./features/clipboard/">
    <span class="feature-card-title">Clipboard integration</span>
    <p class="feature-card-desc">Copied JSON from a browser or API response? Just launch jiq — it reads your clipboard directly. No need to save to a file first.</p>
  </a>
  <a class="feature-card" href="./features/mouse/">
    <span class="feature-card-title">Full mouse support</span>
    <p class="feature-card-desc">Point and click wherever you prefer it over the keyboard. Select output lines by dragging, scroll through results, click suggestions to apply them.</p>
  </a>
  <a class="feature-card" href="./features/tooltip/">
    <span class="feature-card-title">Inline documentation</span>
    <p class="feature-card-desc">Forgot how a jq function works? A tooltip shows its signature and usage examples right in the editor — no context switch to a browser needed.</p>
  </a>
</div>
