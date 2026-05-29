// Performance instrumentation. RAII stopwatches feed an in-memory tally
// during the session; a single summary is dumped to the debug log on exit.
//
// Activation is gated by the same conditions that enable the debug logger
// (--debug flag, JIQ_DEBUG=1 env var, or debug build). When inactive, every
// stopwatch call is a single AtomicBool load and an early return.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(false);
static SUMMARY_DUMPED: AtomicBool = AtomicBool::new(false);
static TALLY: Mutex<Option<HashMap<&'static str, Vec<u64>>>> = Mutex::new(None);

/// Enable perf instrumentation. Called once at startup from `init_logger`
/// when the debug logger is also being enabled.
pub fn enable() {
    if let Ok(mut guard) = TALLY.lock() {
        *guard = Some(HashMap::new());
    }
    ENABLED.store(true, Ordering::Release);
}

/// True if perf instrumentation is active. Cheap to call repeatedly.
#[inline]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Record a single duration sample under `name`. Called by `Stopwatch::drop`.
fn record(name: &'static str, nanos: u64) {
    if !is_enabled() {
        return;
    }
    if let Ok(mut guard) = TALLY.lock()
        && let Some(map) = guard.as_mut()
    {
        map.entry(name).or_default().push(nanos);
    }
}

/// RAII timer. Records elapsed time on drop under the given name. Use the
/// `time!` macro at call sites rather than constructing this directly.
pub struct Stopwatch {
    name: &'static str,
    start: Instant,
}

impl Stopwatch {
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for Stopwatch {
    fn drop(&mut self) {
        if !is_enabled() {
            return;
        }
        let elapsed = self.start.elapsed().as_nanos();
        // Cap at u64::MAX to handle the (impossible) overflow case cleanly.
        let nanos = u64::try_from(elapsed).unwrap_or(u64::MAX);
        record(self.name, nanos);
    }
}

/// Time a block of code under a static name. The stopwatch is dropped
/// when the block exits, recording the duration.
///
/// Two forms:
///   time!("name", { ... block ... })   — times a block, returns block's value
///   let _g = time_guard!("name");      — times the rest of the enclosing scope
#[macro_export]
macro_rules! time {
    ($name:expr, $body:block) => {{
        let _stopwatch = $crate::perf::Stopwatch::new($name);
        $body
    }};
}

/// Create a stopwatch tied to a local variable. Times until the variable
/// is dropped (i.e. end of enclosing scope).
#[macro_export]
macro_rules! time_guard {
    ($name:expr) => {
        $crate::perf::Stopwatch::new($name)
    };
}

/// Dump a percentile summary of all recorded operations to the debug log.
/// Idempotent — repeat calls after the first are no-ops, so wiring into
/// multiple exit paths (clean exit, restore_terminal, panic hook) is safe.
///
/// Snapshots the tally via clone instead of consuming it. Samples
/// recorded after the first dump call are preserved in TALLY for any
/// post-mortem inspection but are not re-logged (the SUMMARY_DUMPED gate
/// makes the printing side strictly idempotent). The bench-script runner
/// pairs this with a drain phase that ensures the worker thread has gone
/// quiescent before the dump fires, so late samples shouldn't occur on
/// that path. The clone-not-take semantics are defensive insurance.
pub fn dump_summary() {
    if !is_enabled() {
        return;
    }
    if SUMMARY_DUMPED.swap(true, Ordering::AcqRel) {
        return;
    }

    let snapshot: HashMap<&'static str, Vec<u64>> = match TALLY.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(map) => map.clone(),
            None => return,
        },
        Err(_) => return,
    };

    if snapshot.is_empty() {
        log::debug!("=== PERF SUMMARY (no samples recorded) ===");
        return;
    }

    let mut entries: Vec<(&'static str, Vec<u64>)> = snapshot.into_iter().collect();
    entries.sort_by_key(|(name, _)| *name);

    log::debug!("=== PERF SUMMARY ===");
    log::debug!(
        "{:<32} {:>8} {:>12} {:>12} {:>12} {:>14}",
        "operation",
        "count",
        "p50",
        "p95",
        "p99",
        "total"
    );
    for (name, mut samples) in entries {
        samples.sort_unstable();
        let stats = Stats::from_sorted(&samples);
        log::debug!(
            "{:<32} {:>8} {:>12} {:>12} {:>12} {:>14}",
            name,
            stats.count,
            format_duration(stats.p50),
            format_duration(stats.p95),
            format_duration(stats.p99),
            format_duration(stats.total)
        );
    }
    log::debug!("=== END PERF SUMMARY ===");
}

struct Stats {
    count: usize,
    p50: u64,
    p95: u64,
    p99: u64,
    total: u64,
}

impl Stats {
    fn from_sorted(sorted: &[u64]) -> Self {
        let count = sorted.len();
        let total: u64 = sorted.iter().sum();
        Self {
            count,
            p50: percentile(sorted, 50),
            p95: percentile(sorted, 95),
            p99: percentile(sorted, 99),
            total,
        }
    }
}

fn percentile(sorted: &[u64], p: u64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    // nearest-rank: index = ceil(p/100 * n) - 1, clamped to [0, n-1]
    let n = sorted.len();
    let idx = (p as usize * n).div_ceil(100).saturating_sub(1).min(n - 1);
    sorted[idx]
}

fn format_duration(nanos: u64) -> String {
    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2}us", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.3}s", nanos as f64 / 1_000_000_000.0)
    }
}
