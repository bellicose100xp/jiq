---
title: Getting started
nav_order: 2
description: Install jiq, load your first JSON file, and learn the core workflow in a few minutes.
---

# Getting started

jiq is an interactive terminal tool for exploring and querying JSON. You type a jq query and see the results update live — no need to re-run a command after every change.

## Before you begin

Install [`jq`](https://jqlang.org/download/) and make sure it's on your `PATH`. jiq runs your queries through jq.

## Install jiq

**macOS**
```bash
brew install bellicose100xp/tap/jiq
```

**macOS / Linux**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

**Any platform with Rust**
```bash
cargo install jiq
```

**Windows / others:** [pre-built binaries](https://github.com/bellicose100xp/jiq/releases/latest)

## Load a JSON file

Run jiq with a filename:

```bash
jiq data.json
```

You can also pipe JSON into jiq:

```bash
curl -s https://api.example.com/users | jiq
```

Or run it with no arguments — jiq reads JSON directly from your clipboard:

```bash
jiq
```

If the clipboard is empty or doesn't contain valid JSON, an interactive paste editor opens where you can paste or correct the input.

## Write your first query

jiq starts with the query `.`, which shows the entire document. The query input is at the top; the results appear in the pane below.

Start typing to change the query. Results update on every keystroke — you don't need to press Enter to run the query.

**Step 1.** Type `.users` to navigate to the users array. The results pane shows the array.

**Step 2.** Type `[]` to expand it — `.users[]` — and jiq fans out all elements, one per result.

**Step 3.** Add `| select(.active)` to filter. Only users where `active` is true remain.

**Step 4.** Add `| .email` to extract a single field. You now have a list of email addresses.

<div class="tui-mockup with-title" data-title="Building the query step by step">
<pre>Step 1  .users
Step 2  .users[]
Step 3  .users[] | select(.active)
Step 4  .users[] | select(.active) | .email

Result after step 4:
  "alice@example.com"
  "carol@example.com"</pre>
</div>

## Get the result out

When you have the output you want:

- Press **Enter** to exit jiq and print the filtered JSON to stdout.
- Press **Ctrl+Q** to exit and print the query string itself (useful for scripts).
- Press **Ctrl+Y** to copy the result to your clipboard without exiting.

## What to explore next

- [Navigate the output](./features/results-pane) — zoom into nested values without typing paths
- [Autocomplete](./features/autocomplete) — get field name suggestions from your actual data
- [Quick reference](./quick-reference) — all keyboard shortcuts on one page
