---
title: Clipboard & paste recovery
parent: Features
nav_order: 8
description: Auto-load JSON from the clipboard on launch, fall back to OSC 52 over SSH, recover via in-app paste box when the clipboard read fails.
---

# Clipboard & paste recovery
{: .no_toc }

[← Features](./){: .btn .btn-outline .fs-3 } [Quick reference](../quick-reference){: .btn .btn-outline .fs-3 } [Configuration](../configuration#clipboard){: .btn .btn-outline .fs-3 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## Three ways to feed jiq JSON

jiq picks its input source from how you launched it. There's no flag to choose — it's the **presence or absence of a file argument and piped stdin**:

<div class="tui-mockup with-title" data-title="Input decision tree">
<pre>
$ jiq &lt;file&gt;        → loads from file
$ ... | jiq         → loads from piped stdin
$ jiq               → loads from clipboard
                      └─ on failure: in-app paste recovery box
</pre>
</div>

The clipboard path is the interesting one. The first two are obvious; the third is the convenience hook that lets you grab JSON from anywhere — a browser, a chat message, an API response in your terminal pager — and just type `jiq`.

---

## Clipboard auto-load on launch

When `jiq` is invoked with no file argument and no piped stdin, it tries the system clipboard. If the content parses as JSON (or JSONL — newline-delimited JSON is auto-detected), jiq loads it and starts normally. You see the result pane populated immediately, exactly as if you'd run `jiq path/to/file.json`.

If the clipboard is empty, unreadable, or contains non-JSON content, jiq drops into [paste recovery](#paste-recovery-view) instead of bailing out.

```bash
# Copy some JSON in your browser, then:
jiq

# Or pull from an API and pipe — equivalent to clipboard for the read,
# but lets jiq watch the stream:
curl -s https://api.example.com/data | jiq
```

{: .note }
> **OSC 52 fallback for SSH.** On remote sessions where `arboard` can't reach a real clipboard (no X11/Wayland forward), jiq falls back to **OSC 52 read** with a 1-second timeout. Modern terminals — Ghostty, kitty, WezTerm, foot — hand the clipboard back through the SSH tunnel.
>
> The OSC 52 path picks up content **copied inside the remote session**: tmux selection buffers, OSC 52 writes from peer apps in the same terminal. Content copied on the **host workstation** generally won't round-trip back to jiq, because most terminals refuse to forward host-clipboard reads through the SSH tunnel for security reasons. If you need that, copy it on the remote side (e.g. `pbcopy` over `ssh`, or paste into a tmux buffer).

---

## Paste recovery view

When the clipboard auto-load fails, jiq doesn't exit — it shows an in-app paste box so you can recover without restarting:

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

The diagnosis line at the top changes based on **why** the auto-load failed, so you know whether to paste, retry, or check your terminal:

| Diagnosis line | What happened |
|---|---|
| `Could not read the system clipboard.` | Backend unavailable — both `arboard` and OSC 52 read failed. |
| `Clipboard is empty.` | Backend reachable, no content. |
| `Clipboard does not contain valid JSON.` | Backend returned text, but it didn't parse. |

Press <kbd>Enter</kbd> to validate and load whatever's in the textarea. On parse failure, the diagnosis updates to `Invalid JSON: <details>` and a red toast nudges your eye to the change. Fix it in place and press <kbd>Enter</kbd> again. <kbd>Ctrl</kbd>+<kbd>X</kbd> clears the textarea so you can paste fresh content without hand-deleting.

### Full VIM editing in the paste box

The paste box reuses the same textarea infrastructure as the main query input, so every motion, operator, and text object you know from there works here too: `dd`, `cc`, `D`, `C`, `dw`, `ci"`, `f`/`F`/`t`/`T`, `;`/`,`, <kbd>u</kbd>, <kbd>Ctrl</kbd>+<kbd>R</kbd>, and `j`/`k`/`g`/`G` for line navigation across the multi-line buffer. See [VIM editing](./vim-editing) for the full set.

The bottom border hint is mode-aware — it shows only the **opposite** mode toggle, so you always see the next thing you'd press: `i Insert` while in NORMAL, `Esc Normal` while in INSERT.

{: .warning }
> **Terminals without bracketed paste.** On Cloud Desktop, plain tmux without `set-option -g set-clipboard on`, and mosh, pasted newlines arrive as <kbd>Ctrl</kbd>+<kbd>J</kbd> keystrokes instead of real line breaks. jiq intercepts these and turns them into actual newlines in the buffer — without the workaround, `tui-textarea`'s default <kbd>Ctrl</kbd>+<kbd>J</kbd> binding (delete-line-by-head) would silently collapse a multi-line paste into one line.

---

## Copying out

Three shortcuts move content **out** of jiq into the system clipboard. They differ in what gets copied and when.

| Shortcut | What it copies | Notes |
|---|---|---|
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | **Focus-aware** — query if input pane focused, results if results pane focused | Matches the visual cue of the cyan/yellow border |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | **Always results**, regardless of focus | Lets you grab the rendered JSON without leaving the query box |
| <kbd>yy</kbd> | Same as <kbd>Ctrl</kbd>+<kbd>Y</kbd> (focus-aware) | NORMAL mode only |
| <kbd>v</kbd> / <kbd>V</kbd> then <kbd>y</kbd> | Selected lines from the results pane | See [Results pane](./results-pane#visual-selection) |

All four routes go through the same backend selected in `config.toml` — see below.

---

## Clipboard backend setting

```toml
# ~/.config/jiq/config.toml
[clipboard]
# "auto"   — try OS clipboard, fall back to OSC 52 (default, recommended)
# "system" — use only OS clipboard (may not work in SSH/tmux)
# "osc52"  — use only OSC 52 (recommended for SSH-only workflows)
backend = "auto"
```

Most users want `auto`. Switch to `osc52` only if you live almost entirely on remote hosts and want to skip the failed `arboard` probe on every launch. Full reference: [Configuration → clipboard](../configuration#clipboard).

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
