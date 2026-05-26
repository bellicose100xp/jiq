---
title: Clipboard & paste
parent: Features
nav_order: 8
description: Load JSON from your clipboard, use the paste box when clipboard fails, and copy results out.
---

# Clipboard and paste

Run `jiq` with no arguments and it loads JSON directly from your clipboard — no file needed.

<div class="before-after">
  <input type="radio" name="ba-clipboard" id="ba-clipboard-before" checked>
  <input type="radio" name="ba-clipboard" id="ba-clipboard-after">
  <div class="ba-header">
    <label for="ba-clipboard-before" class="ba-toggle">Without clipboard loading</label>
    <label for="ba-clipboard-after" class="ba-toggle">With jiq</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You copied JSON from a browser or API tool. Now you need to explore it.</p>
    <div class="ba-terminal">$ pbpaste > /tmp/data.json
$ jiq /tmp/data.json
# or: pbpaste | jiq</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Just run <code>jiq</code>. It reads your clipboard automatically.</p>
    <div class="ba-terminal">$ jiq
# Clipboard JSON loaded directly — start querying</div>
  </div>
</div>

## How clipboard loading works

When you run `jiq` with no file argument and no piped stdin:

1. jiq reads your system clipboard.
2. If the clipboard contains valid JSON, it loads immediately.
3. If the clipboard is empty or invalid, the **paste recovery box** opens.

## Use the paste recovery box

When clipboard auto-load fails, jiq shows a full-screen text area where you can paste manually:

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Paste recovery</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Paste your JSON below (Cmd+V or Ctrl+Shift+V):</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-output">{"users": [{"name": "alice"}, {"name": "bob"}]}</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Enter: validate and load  |  Ctrl+X: clear  |  Esc: toggle mode</span></div>
  </div>
</div>

The paste editor supports the same keyboard shortcuts as the query input, including Vim-style editing if you use it (press Esc for navigation mode, `i` to return to typing).

## Copy results to your clipboard

| What you want | Press |
|---|---|
| Copy results (from any focus) | <kbd>Ctrl</kbd>+<kbd>O</kbd> |
| Copy the focused pane (query if input, results if results) | <kbd>Ctrl</kbd>+<kbd>Y</kbd> |
| Copy specific lines | <kbd>v</kbd> to select, <kbd>y</kbd> to copy |

## Configure the clipboard backend

In `~/.config/jiq/config.toml`:

```toml
[clipboard]
backend = "auto"   # "auto", "system", or "osc52"
```

| Backend | When to use |
|---|---|
| `auto` | Tries system clipboard first, falls back to OSC 52 |
| `system` | Force OS clipboard only (may not work over SSH) |
| `osc52` | Terminal escape sequences — works in most modern terminals over SSH/tmux |

## All keys (paste recovery)

| Key | Action |
|---|---|
| Paste (Cmd+V / Ctrl+Shift+V) | Insert JSON into the box |
| `Enter` | Validate and load the pasted JSON |
| `Ctrl+X` | Clear the text area |
| `Esc` | Switch to navigation mode (move cursor without typing) |
| `i` | Switch back to typing mode |
| Vim shortcuts | Same editing keys as the query input (see [Vim editing](./vim-editing)) |
| `Ctrl+C` | Quit |
