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

When something is wrong, capture debug logs and attach them to your bug report.

```bash
jiq --debug data.json
# or
JIQ_DEBUG=1 jiq data.json
```

Logs are written to `/tmp/jiq-debug.log` at DEBUG level. Output goes to file only — never stdout/stderr — so the TUI stays uncorrupted.

The log covers:

- Session lifecycle (init, terminal setup/restore)
- Config loading
- File I/O and JSONL detection
- Every jq invocation (input, query, exit code, timing)
- Query dispatch (debounce, cancellation)
- AI requests (provider, model, error/parse failures)
- Clipboard operations (which backend tried, which succeeded)

### Filing a bug

Open a [GitHub issue](https://github.com/bellicose100xp/jiq/issues/new) with:

1. The full `/tmp/jiq-debug.log` (it does not contain your full JSON — only metadata and small samples).
2. `jiq --version` output.
3. OS + terminal emulator (Ghostty, kitty, iTerm2, Terminal.app, Windows Terminal, etc.).
4. Steps to reproduce.

---

## Known limitations

### Autocomplete

- **Mid-query edits**: editing in the *middle* of a query falls back to root-level suggestions. Suggestions tail-track the cursor — for accurate field completion, work at the end of the path.
- **Heterogeneous arrays**: jiq samples up to `array_sample_size` elements (default 10) to discover fields. Increase this in `[autocomplete]` config if your data mixes shapes — see [Configuration](./configuration#autocomplete).

### Syntax highlighting

Basic keyword-based only. jiq does not parse the query as a tree, so highlighting won't reflect structural correctness or scope. The error overlay (<kbd>Ctrl</kbd>+<kbd>E</kbd>) gives you the full jq error when needed — see [Tooltip & overlays](./features/tooltip).

### Non-ASCII keys

jq's `.field` shorthand accepts only ASCII identifiers. For keys with CJK, emoji, accented Latin, hyphens, spaces, or digit-starts, use bracket notation:

```text
.["名前"]      ✓ works (bracket notation, what jiq's autocomplete emits)
."名前"        ✓ works (quoted-dot, valid alternative)
.名前           ✗ jq syntax error
```

jiq's autocomplete handles this correctly — and the AI assistant sanitizes its responses to use bracket notation when needed. Manual edits, though, must follow jq's syntax. See [Autocomplete](./features/autocomplete#non-ascii-keys).

### Clipboard over SSH

jiq supports OSC 52 read with a 1-second timeout when the OS clipboard isn't reachable (typical SSH session without X11/Wayland). Caveats:

- Modern terminals (Ghostty, kitty, WezTerm, foot) work; older terminals may not forward OSC 52 reads through SSH.
- Content copied **on the host workstation** generally cannot be read back by jiq — most terminals refuse to forward host-clipboard reads through the SSH tunnel for security reasons.
- Content copied **inside the remote session** (tmux selection buffer, OSC 52 writes from peer apps) does round-trip.

If clipboard auto-load fails, the [paste recovery view](./features/clipboard#paste-recovery-view) opens automatically — paste your JSON, press <kbd>Enter</kbd>, you're in.

---

## Common questions

### "I pressed Enter and got no output."

Check stderr — jiq writes the filtered JSON to **stdout** on <kbd>Enter</kbd>. If you ran `jiq data.json` interactively, the output prints right after the TUI tears down. If you piped (`| jiq | wc -l`), make sure the downstream command is reading stdout.

### "My SSH session can't read the clipboard."

Either:

- Use a terminal that supports OSC 52 forwarding (Ghostty, kitty, WezTerm, foot), or
- Force the OSC 52 backend in `config.toml`: `[clipboard] backend = "osc52"`, or
- Pipe the JSON in instead: `cat data.json | jiq`.

See [Clipboard & paste recovery](./features/clipboard).

### "AI suggestions return errors / fail to parse."

Most often due to network or quota issues. Check `/tmp/jiq-debug.log` for the response body. The page also covers the auto-sanitizer that fixes invalid `.X` suggestions for non-ASCII keys — see [AI assistant](./features/ai-assistant).

### "The TUI looks corrupted after I quit."

If your terminal is left in a weird state, run `reset` or `tput reset`. jiq always restores the terminal on a clean exit; this should only happen if the process was killed mid-render.

### "How do I clear my query history?"

Delete the on-disk history file. Paths:

| OS | Path |
|:---|:---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

Or delete entries one at a time from the history popup with <kbd>Ctrl</kbd>+<kbd>D</kbd>. See [Query history](./features/history).

---

## Still stuck?

- [GitHub Issues](https://github.com/bellicose100xp/jiq/issues) — bug reports and questions.
- [GitHub Discussions](https://github.com/bellicose100xp/jiq/discussions) — feature requests, usage questions.
- [Changelog](./changelog) — recent fixes and behavior changes that might explain a difference.
