---
title: Clipboard & paste
parent: Features
nav_order: 8
description: Load JSON from your clipboard, use the paste box when clipboard fails, and copy results out.
---

# Clipboard & paste

## Load JSON from your clipboard

Run jiq with no arguments:

```bash
jiq
```

jiq reads your clipboard and loads whatever JSON it finds. You can then start querying immediately.

If your terminal is connected over SSH and the OS clipboard isn't available, jiq falls back to OSC 52 (a terminal protocol that works over SSH). This picks up content copied inside the remote terminal session.

## Use the paste box when the clipboard isn't available

If jiq can't read the clipboard, or what it reads isn't valid JSON, the paste box opens automatically instead of failing.

<div class="tui-mockup with-title" data-title="Paste box">
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

1. Paste your JSON with your terminal's paste shortcut.
2. Press **Enter** to validate and load it.

If the JSON is invalid, the top message updates to show the error. Fix it in the box and press **Enter** again.

The paste box supports the same Vim editing as the query input — you can use `dd`, `ciw`, `da[`, undo, and so on to edit what you pasted.

| Key | Action |
|---|---|
| `Enter` | Validate and load |
| `Ctrl+X` | Clear the textarea |
| `Esc` | Toggle NORMAL / INSERT mode |
| All Vim motions | Edit the pasted content |

## Copy the result out

To copy jiq's output to your clipboard without exiting:

| Key | What it copies |
|---|---|
| `Ctrl+Y` | Whatever is focused — query if input, results if results pane |
| `Ctrl+O` | The results, regardless of which pane is focused |
| `yy` in NORMAL mode | Same as `Ctrl+Y` |

In visual selection mode, `Ctrl+Y` and `Ctrl+O` copy only the selected lines.

## Configure the clipboard backend

```toml
# ~/.config/jiq/config.toml
[clipboard]
# auto:    try OS clipboard first, fall back to OSC 52 (default)
# system:  OS clipboard only — works on local terminals
# osc52:   OSC 52 only — best for SSH sessions
backend = "auto"
```

Use `osc52` if you're always on SSH and the OS clipboard never works for you. Use `system` if you want to disable the OSC 52 fallback entirely.
