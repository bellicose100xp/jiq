# Performance Fixtures

JSON inputs for benchmarking. Gitignored — only the generator is tracked.

## Shapes

- **wide** — flat array of objects. Stresses scroll, search, render.
- **deep** — recursively nested tree (~10-12 levels). Stresses path-at-cursor and json_path traversal.
- **keys** — single object with many distinct field names. Stresses autocomplete caches.

## Sizes

| Size   | Lines     | File size | Notes                                     |
|--------|-----------|-----------|-------------------------------------------|
| small  | 10,000    | ~200 KB   | Fast iteration                            |
| medium | 100,000   | ~2.3 MB   | Typical workload                          |
| large  | 1,000,000 | ~23 MB    | Real-world stress (default for benches)   |

`keys` scales by field count (1,000 / 10,000 / 100,000) instead of lines.
The `large` variant exceeds the autocomplete cache cap so cap-handling is
exercised.

## Generate

```sh
cargo build --bin gen_perf_fixture
./target/debug/gen_perf_fixture <shape> --size <small|medium|large>
```

Output: `tests/perf_fixtures/<shape>_<size>.json`. Deterministic — same
args produce byte-identical output across runs.

## Run a benchmark

Always release build. Debug is 5-10× slower with different bottlenecks.

**Manual TUI session (primary):**

```sh
cargo build --release
./target/release/jiq --debug < tests/perf_fixtures/wide_medium.json
# interact, exit with Ctrl+C
grep -A 20 "PERF SUMMARY" /tmp/jiq-debug.log
```

`--debug` enables both the log and the perf timers; the summary is
appended at exit.

**Automated bench-script run** (deterministic, repeatable — scripts in
[`tests/perf_scripts/`](../perf_scripts/README.md)):

```sh
script -q -c "./target/release/jiq --debug \
  --bench-script tests/perf_scripts/typical.script \
  < tests/perf_fixtures/wide_medium.json" /dev/null
```

`script(1)` provides a PTY (jiq needs a TTY for raw mode). On macOS:
`script -q /dev/null <command>` (positional). Run each scenario 3× and
compare medians to wash out noise.
