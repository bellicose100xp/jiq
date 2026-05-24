---
title: Clipboard & paste
parent: Features
nav_order: 8
description: Three input paths — file, stdin, clipboard with paste-box fallback. Plus copy-out shortcuts.
---

# Clipboard & paste

## Launch

```bash
jiq data.json          # 1. file
... | jiq              # 2. piped stdin
jiq                    # 3. clipboard, with paste-box fallback
```

With no file and no piped stdin, jiq reads from the clipboard. If the clipboard is empty, unreadable, or not valid JSON / JSONL, the **paste-recovery** view opens instead of failing.

The clipboard read tries the OS clipboard first; if that fails — typical for SSH sessions without X11/Wayland forwarding — it falls back to **OSC 52** with a 1-second timeout. OSC 52 picks up content copied inside the remote session (tmux selection, peer-app writes); host-clipboard contents usually don't round-trip because terminals refuse to forward those reads back through SSH.

<div class="tui-mockup with-title" data-title="Paste-recovery view">
<pre>┌─ No JSON loaded ─────────────────────────────┐
│ Clipboard does not contain valid JSON.       │
└──────────────────────────────────────────────┘
┌─ Paste here, then Enter ─────────────────────┐
│ {                                            │
│   "users": [                                 │
│     { "name": "alice", "active": true }      │
│   ]                                          │
│ }                                            │
└── Esc Normal · Ctrl+X Clear · Enter Load ───┘</pre>
</div>

Paste with your terminal's normal shortcut. <kbd>Enter</kbd> validates and loads. On invalid input, the top block updates to `Invalid JSON: <detail>` and a red toast nudges your eye to the change.

The textarea has every VIM binding the query input has — operators (`dd`, `cc`, `D`, `dw`), text objects (`ci"`, `da[`, `ci|`), char-search (`f`, `t`, `;`), undo/redo. `j` `k` `g` `G` move between lines.

| Key | Action |
|---|---|
| <kbd>Enter</kbd> | Validate and load |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Clear the textarea |
| <kbd>Esc</kbd> | Toggle Normal / Insert |
| All VIM motions | Edit the paste |
{: .shortcuts }

If your terminal doesn't forward bracketed paste (Cloud Desktop, plain tmux, mosh), pasted line breaks arrive as <kbd>Ctrl</kbd>+<kbd>J</kbd>; jiq intercepts those and inserts real newlines so the paste lands intact.

The paste cap is 16 MiB.

## Copying out

| Key | What it copies |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Whatever is focused — query if input, results if results |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Results (regardless of focus) |
| <kbd>yy</kbd> in NORMAL | Same as <kbd>Ctrl</kbd>+<kbd>Y</kbd> |
{: .shortcuts }

In visual mode, <kbd>Ctrl</kbd>+<kbd>Y</kbd> / <kbd>Ctrl</kbd>+<kbd>O</kbd> copy only the selected lines.

The copy backend is configurable in `~/.config/jiq/config.toml`:

```toml
[clipboard]
# auto:    OS clipboard, fall back to OSC 52  (default)
# system:  OS clipboard only
# osc52:   OSC 52 only (best on remote SSH)
backend = "auto"
```
