---
title: Clipboard & paste
parent: Features
nav_order: 8
description: Smart source picker on launch, paste editor for manual input, and clipboard copy from results.
---

# Clipboard and paste

When you run `jiq` with no file argument and no piped stdin, jiq peeks the clipboard once at launch and lets you confirm what to load — no surprise auto-loads.

## The source picker

If the clipboard contains a JSON object or array, jiq shows a small picker banner with the cached payload previewed below:

```
┌─ Choose JSON input source ───────────────────────────────────┐
│  ▶ Clipboard                                                 │
│    Paste                                                     │
└─ Enter Load • ↑/↓ Switch • Esc Quit ─────────────────────────┘
┌─ Clipboard preview ──────────────────────────────────────────┐
│  {                                                           │
│    "users": [                                                │
│      { "name": "alice", "active": true },                    │
│      …                                                       │
│  }                                                           │
│  … (47 more lines, 11.8 KB more)                             │
└──────────────────────────────────────────────────────────────┘
```

- `Enter` loads the highlighted source. The bottom-border hint text adapts: it reads "Enter Load" when Clipboard is highlighted, "Enter Open paste editor" when Paste is highlighted.
- `↑` / `↓` (or `Tab`, `j`/`k`, `h`/`l`) toggle between the two options.
- `Esc` quits jiq immediately.

## Smart fallback when the clipboard isn't queryable

If the clipboard is empty, isn't JSON, holds a primitive value, or can't be read at all, jiq skips the picker entirely and drops straight into the paste editor with a one-line "Info" box describing what was on the clipboard. Pick from the message and paste manually:

| What jiq saw | Info-box message |
|---|---|
| Clipboard read failed | `Couldn't read the clipboard` |
| Empty buffer | `Clipboard is empty` |
| Not valid JSON | `Clipboard contents aren't valid JSON` |
| A primitive (`42`, `"x"`, `true`, `null`) | `Clipboard JSON is a primitive (e.g. 42, "x", true) — needs an object or array` |

The paste editor has its own title and placeholder telling you how to operate it; the Info box only carries the diagnosis.

## Skip the picker with a flag

| Flag | Behavior |
|---|---|
| `jiq --clipboard` | Force the clipboard auto-load (skips the picker; uses the legacy "load directly" path). |
| `jiq --paste` | Open the paste editor immediately, without touching the clipboard. The editor uses a calm cyan border and no Info box — its title and placeholder say everything. |

The two flags are mutually exclusive. Combining either with piped stdin or a file argument hard-errors with a colored "ambiguous input source" message that lists every valid invocation form.

## The paste editor

A full-screen text area with the same Vim-style editing as the query input. Paste, edit, and press `Enter` to load.

| Key | Action |
|---|---|
| Paste (Cmd+V / Ctrl+Shift+V) | Insert JSON into the box |
| `Enter` | Validate and load the pasted JSON |
| `Ctrl+X` | Clear the text area |
| `Esc` | Switch to navigation mode |
| `i` | Switch back to typing mode |
| Vim shortcuts | Same editing keys as the query input (see [Vim editing](./vim-editing)) |
| `Ctrl+C` | Quit |

Manual paste rejects bare primitives the same way the clipboard does — `42`, `"hello"`, `true`, and `null` all return "Input must be a JSON object or array, not a primitive value." so the same rule applies regardless of how JSON entered jiq.

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
