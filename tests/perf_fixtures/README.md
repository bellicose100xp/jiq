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

## Run the full matrix

Streams a one-line summary per cell as it completes. Each cell waits for
`PERF SUMMARY` to appear in the debug log instead of always sleeping the
full timeout, so a clean 27-cell run takes ~4 minutes rather than ~25.

```sh
cargo build --release
tests/perf_fixtures/run_matrix.sh
```

Cell summaries print to stdout as `[Xs] shape size run=N
inject_key_event_p99=Y ms  [tag ±%] base=A cur=B`. `tag` is `WIN` for
≥20% faster than baseline, `REGRESSION` for ≥20% slower, otherwise
`flat`. Threshold is informational — the runner never stops early.

Full per-cell perf blocks land in `tests/perf_fixtures/matrix_latest.txt`
(gitignored). The committed `tests/perf_fixtures/baseline_matrix.txt` is
the reference baseline against which `run_matrix.sh` compares.

To capture a fresh baseline (no comparison):

```sh
tests/perf_fixtures/run_matrix.sh /dev/null
cp tests/perf_fixtures/matrix_latest.txt tests/perf_fixtures/baseline_matrix.txt
```

Refresh the baseline whenever `perf` lands a real win and we want
subsequent measurements to compare against the new floor.
