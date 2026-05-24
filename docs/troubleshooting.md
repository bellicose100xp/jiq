---
title: Troubleshooting
nav_order: 6
description: Debug logging and known limitations.
---

# Troubleshooting

## Debug logging

```bash
jiq --debug data.json
# or
JIQ_DEBUG=1 jiq data.json
```

Logs to `/tmp/jiq-debug.log` (file only, never stdout/stderr).

### Filing a bug

Open a [GitHub issue](https://github.com/bellicose100xp/jiq/issues/new) with:

1. `/tmp/jiq-debug.log`.
2. `jiq --version`.
3. OS + terminal emulator.
4. Steps to reproduce.

## Known limitations

- **Autocomplete mid-query**: editing in the middle of a query falls back to root-level suggestions. Work at the end of the path.
- **Syntax highlighting**: keyword-based only, no scope analysis. Use the error overlay (<kbd>Ctrl</kbd>+<kbd>E</kbd>) for the full jq error.
- **Non-ASCII keys**: jq's `.field` shorthand is ASCII-only. Use bracket notation for CJK, emoji, hyphens, spaces, digit-starts:

  ```text
  .["名前"]   ✓ bracket notation (what autocomplete emits)
  ."名前"     ✓ quoted-dot
  .名前        ✗ jq syntax error
  ```

- **Clipboard over SSH**: OSC 52 read works on modern terminals (Ghostty, kitty, WezTerm, foot) for content copied inside the session. Host-workstation copies don't round-trip.
