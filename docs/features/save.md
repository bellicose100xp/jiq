---
title: Save to file
parent: Features
nav_order: 9
description: Press Ctrl+W to save the current result to a JSON file. The popup shows you the resolved path live as you type, and warns if the file already exists.
---

# Save to file

Press <kbd>Ctrl</kbd>+<kbd>W</kbd> to save the current jq result to a file. A small popup appears with a pre-filled path — edit if you like, watch the resolved path update live, press <kbd>Enter</kbd>, and the result lands on disk.

## Activation

Ctrl+W works from anywhere the query and results are visible — input pane (any vim mode), results pane, error overlay. It's suppressed while a popup that owns its own keys is open (snippets, history, search, help): close that popup first, then press Ctrl+W.

If there's no result yet — empty input, jq error, paste-recovery still active — Ctrl+W shows a transient "Nothing to save" notification instead of opening the popup.

## The popup

```
╭ Save Result to file ─────────────────────────────────────╮
│ Path:                                                    │
│ ╭──────────────────────────────────────────────────────╮ │
│ │jiq-20260527-104522.json                              │ │
│ ╰──────────────────────────────────────────────────────╯ │
│ → /home/you/projects/api/jiq-20260527-104522.json        │
│ Enter Save  Esc Cancel                                   │
╰──────────────────────────────────────────────────────────╯
```

The path starts pre-filled with `jiq-{timestamp}.json`, written into the directory you launched jiq from. The line below the input shows the **resolved** path — `~`, env vars, `{cwd}`, and `{timestamp}` are all expanded as you type.

| Key | Action |
|---|---|
| <kbd>Enter</kbd> | Save (or overwrite — the popup tells you which) |
| <kbd>Esc</kbd> | Cancel without writing |

## Path expansion

Anything you can type at a shell prompt works in this field:

| Placeholder | Expands to |
|---|---|
| `{timestamp}` | `YYYYMMDD-HHMMSS` (e.g. `20260527-104522`) |
| `{ext}` | `json` |
| `{cwd}` | Current working directory |
| `~` | Your home directory (only at the start of the path) |
| `$VAR` or `${VAR}` | Environment variable lookup |
| absolute (`/foo/bar.json`) | Used verbatim |
| relative (`out.json`, `./out.json`) | Resolved against `{cwd}` |

The timestamp is captured once when the popup opens and stays stable while you edit, so the resolved path you see is the path that will be written.

## When the file already exists

The popup itself becomes the warning — no extra confirmation step:

```
╭ Save Result to file ─────────────────────────────────────╮
│ Path:                                                    │
│ ╭──────────────────────────────────────────────────────╮ │
│ │results.json                                          │ │
│ ╰──────────────────────────────────────────────────────╯ │
│ ⚠ File exists: /home/you/projects/api/results.json       │
│ Enter Overwrite  Esc Cancel                              │
╰──────────────────────────────────────────────────────────╯
```

The preview line turns yellow with a `⚠`, the popup border picks up the same warning color, and the button label flips from `Save` to `Overwrite`. As soon as you edit the path to something that doesn't collide, everything flips back.

## When the path can't resolve

If your pattern references an unset env var or expands to nothing, the preview shows the error inline — the bad path never gets written:

```
│ ✕ env var UNSET_VAR not set                              │
```

Pressing <kbd>Enter</kbd> while in this state surfaces the same error as a sticky notification.

## Atomic write

jiq writes to a sibling temporary file (`.<filename>.tmp-<pid>`), `fsync`s it, then renames it into place. If your editor or shell tab is sharing the directory, you'll never see a half-written file at the final path — either the new bytes are fully there, or the previous file is unchanged.

If the rename can't be atomic (e.g. across filesystems), jiq falls back to a direct write so saves still succeed.

## Errors

Write errors stay non-fatal — the popup closes after a successful write, but stays open if the write fails.

| Cause | Notification |
|---|---|
| Parent directory doesn't exist | `Save failed: parent directory does not exist: <path>` |
| Permission denied | `Save failed: Permission denied (os error 13)` |
| Disk full / similar OS error | `Save failed: <verbatim OS error>` |
| Unset env var in pattern | `Save failed: env var FOO not set` (also surfaced live in the preview) |
| Empty filename | `Save failed: filename is empty` (also surfaced live) |

## Suppression matrix

| Active surface | Ctrl+W behavior |
|---|---|
| Input pane (any vim mode) | Opens save popup |
| Results pane | Opens save popup |
| Snippets popup open | Suppressed — Esc out first |
| History popup open | Suppressed — Esc out first |
| Search popup open | Suppressed — Esc out first |
| Help popup open | Suppressed — F1 to close first |
| Save popup itself open | Routed to its own handler (Esc closes) |

## Shortcuts

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>W</kbd> | Open save popup |
| <kbd>Enter</kbd> | Save (or overwrite, depending on preview) |
| <kbd>Esc</kbd> | Cancel without writing |
