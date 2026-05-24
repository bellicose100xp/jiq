---
title: Clipboard & paste recovery
parent: Features
nav_order: 8
description: Auto-load JSON from the clipboard on launch, fall back to OSC 52 over SSH, recover via in-app paste box when the clipboard read fails.
---

# Clipboard & paste recovery
{: .no_toc }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## Input sources

<div class="tui-mockup with-title" data-title="How jiq picks its input">
<pre>
$ jiq &lt;file&gt;        → loads from file
$ ... | jiq         → loads from piped stdin
$ jiq               → loads from clipboard
                      └─ on failure: in-app paste recovery box
</pre>
</div>

---

## Clipboard auto-load on launch

With no file and no piped stdin, jiq reads from the clipboard. If the clipboard is empty or not valid JSON, the in-app paste box opens — paste, press <kbd>Enter</kbd>. JSONL (newline-delimited JSON) is auto-detected.

```bash
curl -s https://api.example.com/data | jiq
```

{: .note }
> **OSC 52 fallback for SSH.** On remote sessions where `arboard` can't reach a real clipboard, jiq falls back to OSC 52 read with a 1-second timeout. Modern terminals (Ghostty, kitty, WezTerm, foot) hand the clipboard back through the SSH tunnel.
>
> OSC 52 picks up content copied **inside the remote session** (tmux buffers, OSC 52 writes from peer apps). Content copied on the **host workstation** generally won't round-trip — most terminals refuse to forward host-clipboard reads through the SSH tunnel for security. Copy on the remote side, or use the paste box.

---

## Paste recovery view

When clipboard auto-load fails, jiq shows an in-app paste box instead of exiting:

<div class="tui-mockup" markdown="0">
<pre>
╭─ No JSON loaded ─────────────────────────────────────────╮
│ Clipboard does not contain valid JSON.                   │
│                                                          │
│ Paste your JSON below and press Enter to load.           │
╰──────────────────────────────────────────────────────────╯
╭─ Paste ─ INSERT ─────────────────────────────────────────╮
│ {                                                        │
│   "users": [                                             │
│     ...                                                  │
│   ]                                                      │
│ }                                                        │
╰── Enter Load · Ctrl+X Clear · i Insert · Esc Normal ─────╯
</pre>
</div>

The diagnosis line indicates why auto-load failed:

| Diagnosis line | What happened |
|---|---|
| `Could not read the system clipboard.` | Backend unavailable — both `arboard` and OSC 52 read failed. |
| `Clipboard is empty.` | Backend reachable, no content. |
| `Clipboard does not contain valid JSON.` | Backend returned text, but it didn't parse. |

<kbd>Enter</kbd> validates and loads. On parse failure, the diagnosis updates to `Invalid JSON: <details>` with a red toast. <kbd>Ctrl</kbd>+<kbd>X</kbd> clears the textarea.

### Full VIM editing in the paste box

The paste box uses the same textarea infrastructure as the query input — every motion, operator, and text object works: `dd`, `cc`, `D`, `C`, `dw`, `ci"`, `f`/`F`/`t`/`T`, `;`/`,`, <kbd>u</kbd>, <kbd>Ctrl</kbd>+<kbd>R</kbd>, plus `j`/`k`/`g`/`G` for line navigation. See [VIM editing](./vim-editing).

The bottom border hint shows the opposite mode's toggle: `i Insert` in NORMAL, `Esc Normal` in INSERT.

{: .warning }
> **Terminals without bracketed paste.** On Cloud Desktop, plain tmux without `set-option -g set-clipboard on`, and mosh, pasted newlines arrive as <kbd>Ctrl</kbd>+<kbd>J</kbd>. jiq intercepts these and converts them to real line breaks — without this, `tui-textarea`'s default <kbd>Ctrl</kbd>+<kbd>J</kbd> (delete-line-by-head) would collapse multi-line pastes.

---

## Copying out

| Shortcut | What it copies | Notes |
|---|---|---|
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Focus-aware — query if input focused, results if results focused | Matches the cyan/yellow border cue |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Always results, regardless of focus | Grab rendered JSON without leaving the query box |
| <kbd>yy</kbd> | Focus-aware (same as <kbd>Ctrl</kbd>+<kbd>Y</kbd>) | NORMAL mode only |
| <kbd>v</kbd> / <kbd>V</kbd> then <kbd>y</kbd> | Selected lines from the results pane | See [Results pane](./results-pane#visual-selection) |

All routes use the backend configured in `config.toml`.

---

## Clipboard backend setting

```toml
# ~/.config/jiq/config.toml
[clipboard]
# "auto"   — try OS clipboard, fall back to OSC 52 (default)
# "system" — OS clipboard only (may not work in SSH/tmux)
# "osc52"  — OSC 52 only (skips arboard probe on remote-only setups)
backend = "auto"
```

Full reference: [Configuration → clipboard](../configuration#clipboard).

---

## Shortcut summary
{: .no_toc }

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Focus-aware copy (query or results) |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Copy results regardless of focus |
| <kbd>yy</kbd> *(NORMAL)* | Focus-aware copy |
| <kbd>v</kbd> / <kbd>V</kbd> + <kbd>y</kbd> | Visual-line selection then copy |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> *(paste recovery)* | Clear the paste textarea |
| <kbd>Enter</kbd> *(paste recovery)* | Validate and load pasted JSON |
{: .shortcuts }
