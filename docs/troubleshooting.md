---
title: Troubleshooting
nav_order: 6
description: Debug logging, known limitations, and tips for diagnosing jiq issues.
---

# Troubleshooting
{: .no_toc }

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{: toc }
</details>

---

## Debug logging

```bash
jiq --debug data.json
# or
JIQ_DEBUG=1 jiq data.json
```

Logs go to `/tmp/jiq-debug.log` (file only, never stdout/stderr). Covers:

- Session lifecycle (init, terminal setup/restore)
- Config loading
- File I/O and JSONL detection
- Every jq invocation (input, query, exit code, timing)
- Query dispatch (debounce, cancellation)
- AI requests (provider, model, error/parse failures)
- Clipboard operations (backend tried, result)

### Filing a bug

Open a [GitHub issue](https://github.com/bellicose100xp/jiq/issues/new) with:

1. `/tmp/jiq-debug.log` (metadata + small samples only, not your full JSON).
2. `jiq --version` output.
3. OS + terminal emulator.
4. Steps to reproduce.

---

## Known limitations

### Autocomplete

- **Mid-query edits** fall back to root-level suggestions. Work at the end of the path for accurate field completion.
- **Heterogeneous arrays**: jiq samples up to `array_sample_size` elements (default 10). Bump it in `[autocomplete]` config — see [Configuration](./configuration#autocomplete).

### Syntax highlighting

Keyword-based only; no tree-sitter style scope analysis. Use the error overlay (<kbd>Ctrl</kbd>+<kbd>E</kbd>) for the full jq error — see [Tooltip & overlays](./features/tooltip).

### Non-ASCII keys

jq's `.field` shorthand is ASCII-only. For CJK, emoji, accented Latin, hyphens, spaces, or digit-starts, use bracket notation:

```text
.["名前"]      ✓ works (bracket notation, what jiq's autocomplete emits)
."名前"        ✓ works (quoted-dot, valid alternative)
.名前           ✗ jq syntax error
```

Autocomplete and the AI assistant emit bracket notation automatically. Manual edits must follow jq's syntax. See [Autocomplete](./features/autocomplete#non-ascii-keys).

### Clipboard over SSH

OSC 52 read with a 1-second timeout when the OS clipboard is unreachable.

- Modern terminals (Ghostty, kitty, WezTerm, foot) forward OSC 52; older ones may not.
- Host-workstation copies generally don't round-trip — terminals refuse to forward host-clipboard reads back through SSH.
- Remote-session copies (tmux buffer, OSC 52 writes from peer apps) do round-trip.

If clipboard auto-load fails, the [paste recovery view](./features/clipboard#paste-recovery-view) opens — paste, <kbd>Enter</kbd>.

---

## Common questions

### Pressed Enter, got no output

jiq writes filtered JSON to **stdout** on <kbd>Enter</kbd>; output prints after the TUI tears down. If piping (`| jiq | wc -l`), confirm the downstream command reads stdout.

### SSH session can't read the clipboard

Use a terminal that forwards OSC 52 (Ghostty, kitty, WezTerm, foot), force `[clipboard] backend = "osc52"`, or pipe: `cat data.json | jiq`. See [Clipboard & paste recovery](./features/clipboard).

### AI suggestions return errors / fail to parse

Usually network or quota. Check `/tmp/jiq-debug.log`. See [AI assistant](./features/ai-assistant) for the non-ASCII auto-sanitizer.

### TUI looks corrupted after quit

Run `reset` or `tput reset`. jiq restores the terminal on clean exit; corruption means the process was killed mid-render.

### Clear query history

Delete the on-disk file:

| OS | Path |
|:---|:---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

Or use <kbd>Ctrl</kbd>+<kbd>D</kbd> in the history popup. See [Query history](./features/history).

---

## Still stuck?

- [GitHub Issues](https://github.com/bellicose100xp/jiq/issues) — bug reports.
- [GitHub Discussions](https://github.com/bellicose100xp/jiq/discussions) — feature requests, usage questions.
- [Changelog](./changelog) — recent fixes and behavior changes.
