#!/bin/bash
# Streaming perf matrix runner. Runs all 27 cells (3 shapes × 3 sizes × 3
# runs) and streams a one-line summary per cell as it completes.
#
# Usage: tests/perf_fixtures/run_matrix.sh [BASELINE_FILE]
#   BASELINE_FILE defaults to tests/perf_fixtures/baseline_matrix.txt.
#   Pass /dev/null to skip comparison and just capture results.
#
# Output:
#   tests/perf_fixtures/matrix_latest.txt — full per-cell perf block.
#   stdout — streaming `[Xs] shape size run=N inject_key_event_p99=Y ms ...`.
#
# Run from the package root. Cell waits for "PERF SUMMARY" in the debug log
# rather than always sleeping the full timeout cap.

set -u

if [ ! -f Cargo.toml ] || [ ! -d tests/perf_fixtures ]; then
    echo "error: run from the package root (cwd has Cargo.toml + tests/perf_fixtures/)" >&2
    exit 2
fi

if ! command -v script >/dev/null 2>&1; then
    echo "error: 'script' (PTY wrapper) is required. macOS ships it; on Linux install bsdmainutils/util-linux." >&2
    exit 2
fi

OUT=tests/perf_fixtures/matrix_latest.txt
BASELINE="${1:-tests/perf_fixtures/baseline_matrix.txt}"
> "$OUT"

MAX_TIMEOUT_SMALL=30
MAX_TIMEOUT_LARGE=120
POLL_INTERVAL=0.25

extract_p99() {
    local file="$1" shape="$2" size="$3" run="$4"
    local header="^===== shape=${shape} size=${size} run=${run} "
    awk -v h="$header" '
        $0 ~ h { in_block=1; next }
        /^===== / && in_block { in_block=0 }
        in_block && /inject_key_event/ {
            for (i=1;i<=NF;i++) {
                if ($i == "inject_key_event") {
                    raw = $(i+4)
                    if (match(raw, /^[0-9.]+/)) {
                        val = substr(raw, 1, RLENGTH)
                        unit = substr(raw, RLENGTH+1)
                        if (unit == "ms")              { print val; exit }
                        if (unit == "us" || unit == "µs") { printf "%.3f\n", val/1000; exit }
                        if (unit == "ns")              { printf "%.6f\n", val/1000000; exit }
                        if (unit == "s")               { printf "%.0f\n", val*1000; exit }
                    }
                }
            }
        }
    ' "$file" 2>/dev/null
}

delta_tag() {
    local base="$1" cur="$2"
    if [ -z "$base" ] || [ -z "$cur" ]; then echo "(no baseline)"; return; fi
    awk -v b="$base" -v c="$cur" 'BEGIN {
        if (b == 0) { print "(baseline=0)"; exit }
        d = (c - b) / b * 100
        tag = "flat"
        if (d >= 20)  tag = "REGRESSION"
        if (d <= -20) tag = "WIN"
        printf "[%s %+.1f%%] base=%.2fms cur=%.2fms\n", tag, d, b, c
    }'
}

run_cell() {
    local shape="$1" size="$2" script="$3" run="$4"
    local fixture="tests/perf_fixtures/${shape}_${size}.json"
    local timeout=$MAX_TIMEOUT_SMALL
    if [ "$size" = "large" ]; then timeout=$MAX_TIMEOUT_LARGE; fi

    truncate -s 0 /tmp/jiq-debug.log

    local start_ts=$(date +%s)
    if [ "$(uname)" = "Darwin" ]; then
        script -q /dev/null sh -c "./target/release/jiq --debug --bench-script ${script} < ${fixture}" > /dev/null 2>&1 &
    else
        script -q -c "./target/release/jiq --debug --bench-script ${script} < ${fixture}" /dev/null > /dev/null 2>&1 &
    fi
    local sp=$!

    local elapsed=0
    while [ "$elapsed" -lt "$timeout" ]; do
        if grep -q "PERF SUMMARY" /tmp/jiq-debug.log 2>/dev/null; then
            sleep 0.1
            break
        fi
        sleep "$POLL_INTERVAL"
        elapsed=$(($(date +%s) - start_ts))
    done
    kill "$sp" 2>/dev/null
    wait "$sp" 2>/dev/null

    local end_ts=$(date +%s)
    local cell_secs=$((end_ts - start_ts))

    echo "===== shape=${shape} size=${size} run=${run} script=${script} =====" >> "$OUT"
    grep -A 25 "PERF SUMMARY" /tmp/jiq-debug.log | head -25 >> "$OUT"
    echo "" >> "$OUT"

    local cur_p99 base_p99
    cur_p99=$(extract_p99 "$OUT" "$shape" "$size" "$run")
    base_p99=""
    if [ -f "$BASELINE" ] && [ "$BASELINE" != "/dev/null" ]; then
        base_p99=$(extract_p99 "$BASELINE" "$shape" "$size" "$run")
    fi
    local tag
    tag=$(delta_tag "$base_p99" "$cur_p99")
    printf "[%2ds] %-6s %-6s run=%d  inject_key_event_p99=%s ms  %s\n" \
        "$cell_secs" "$shape" "$size" "$run" "${cur_p99:-?}" "$tag"
}

if [ ! -x ./target/release/jiq ]; then
    echo "error: ./target/release/jiq not found. Build first: cargo build --release" >&2
    exit 2
fi

echo "Streaming matrix run starting. Baseline: $BASELINE"
echo "Output: $OUT"
echo ""

for size in small medium large; do
    for shape_pair in \
        "wide   tests/perf_scripts/typical.script" \
        "deep   tests/perf_scripts/deep_drilling.script" \
        "keys   tests/perf_scripts/autocomplete_burst.script"; do
        shape="${shape_pair%% *}"
        script="${shape_pair##* }"
        for run in 1 2 3; do
            run_cell "$shape" "$size" "$script" "$run"
        done
    done
done

echo ""
echo "FULL MATRIX RUN COMPLETE" | tee -a "$OUT"
