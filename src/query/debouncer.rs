//! Query execution debouncer
//!
//! Delays query execution during fast typing to reduce unnecessary jq process spawns.
//! Uses a 50ms debounce delay that is imperceptible to users but effectively batches
//! rapid keystrokes into a single execution.

use std::time::{Duration, Instant};

/// Debounce delay in milliseconds.
/// 50ms is imperceptible to users but batches rapid keystrokes effectively.
const DEBOUNCE_MS: u64 = 50;

/// Manages debounced query execution timing.
///
/// Tracks when the last input occurred and whether there's a pending execution.
/// The debouncer delays execution until `DEBOUNCE_MS` has elapsed since the last input,
/// allowing rapid keystrokes to be batched into a single query execution.
#[derive(Debug)]
pub struct Debouncer {
    /// Timestamp of the last input that triggered a debounce
    last_input_time: Option<Instant>,
    /// Whether there's a pending query execution waiting for debounce to expire
    pending_execution: bool,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new()
    }
}

impl Debouncer {
    /// Creates a new debouncer with no pending execution.
    pub fn new() -> Self {
        Self {
            last_input_time: None,
            pending_execution: false,
        }
    }

    /// Schedule a query execution after the debounce delay.
    ///
    /// Records the current time and marks execution as pending.
    /// If called multiple times in rapid succession, each call resets the timer.
    pub fn schedule_execution(&mut self) {
        self.last_input_time = Some(Instant::now());
        self.pending_execution = true;
    }

    /// Check if the debounce period has elapsed and execution should proceed.
    ///
    /// Returns `true` if:
    /// - There is a pending execution, AND
    /// - At least `DEBOUNCE_MS` milliseconds have elapsed since the last input
    pub fn should_execute(&self) -> bool {
        if !self.pending_execution {
            return false;
        }

        match self.last_input_time {
            Some(last_time) => {
                let elapsed = last_time.elapsed();
                elapsed >= Duration::from_millis(DEBOUNCE_MS)
            }
            None => false,
        }
    }

    /// Mark the pending execution as complete.
    ///
    /// Clears the pending flag and resets the timer.
    /// Should be called after query execution completes.
    pub fn mark_executed(&mut self) {
        self.pending_execution = false;
        self.last_input_time = None;
    }

    /// Check if there's a pending execution.
    ///
    /// Returns `true` if `schedule_execution()` was called and
    /// `mark_executed()` has not been called since.
    pub fn has_pending(&self) -> bool {
        self.pending_execution
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::thread;

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_new_debouncer_has_no_pending() {
        let debouncer = Debouncer::new();
        assert!(!debouncer.has_pending());
        assert!(!debouncer.should_execute());
    }

    #[test]
    fn test_schedule_execution_sets_pending() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();
        assert!(debouncer.has_pending());
    }

    #[test]
    fn test_should_execute_false_immediately_after_schedule() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();
        // Immediately after scheduling, should_execute should be false
        // (debounce period hasn't elapsed)
        assert!(!debouncer.should_execute());
    }

    #[test]
    fn test_should_execute_true_after_debounce_period() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();

        // Wait for debounce period to elapse
        thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));

        assert!(debouncer.should_execute());
    }

    #[test]
    fn test_mark_executed_clears_state() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();

        // Wait for debounce period
        thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));
        assert!(debouncer.should_execute());

        // Mark as executed
        debouncer.mark_executed();

        // State should be cleared
        assert!(!debouncer.has_pending());
        assert!(!debouncer.should_execute());
    }

    #[test]
    fn test_schedule_resets_timer() {
        let mut debouncer = Debouncer::new();

        // First schedule
        debouncer.schedule_execution();

        // Wait half the debounce period
        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2));

        // Schedule again (should reset timer)
        debouncer.schedule_execution();

        // Should not execute yet (timer was reset)
        assert!(!debouncer.should_execute());

        // Wait another half period (total 1.5x from first, but only 0.5x from second)
        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2));

        // Still should not execute (only ~0.5x debounce period since last schedule)
        assert!(!debouncer.should_execute());

        // Wait remaining time plus buffer
        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2 + 10));

        // Now should execute
        assert!(debouncer.should_execute());
    }

    #[test]
    fn test_default_impl() {
        let debouncer = Debouncer::default();
        assert!(!debouncer.has_pending());
        assert!(!debouncer.should_execute());
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // Feature: performance, Property 2: Debounce timer reset on input
    // *For any* sequence of keystrokes where each keystroke occurs within 50ms
    // of the previous one, the debouncer should reset its timer on each keystroke
    // and not trigger execution until 50ms after the final keystroke.
    // **Validates: Requirements 2.1, 2.2, 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_debounce_timer_reset_on_input(
            // Generate 2-10 rapid inputs
            num_inputs in 2usize..=10,
        ) {
            let mut debouncer = Debouncer::new();

            // Simulate rapid inputs (each within debounce period)
            for _ in 0..num_inputs {
                debouncer.schedule_execution();
                // Small delay but less than debounce period
                thread::sleep(Duration::from_millis(5));
            }

            // Immediately after rapid inputs, should NOT execute
            // (debounce timer was reset on each input)
            prop_assert!(
                !debouncer.should_execute(),
                "Should not execute immediately after rapid inputs"
            );

            // Should still have pending execution
            prop_assert!(
                debouncer.has_pending(),
                "Should have pending execution after scheduling"
            );

            // Wait for full debounce period after last input
            thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));

            // Now should execute
            prop_assert!(
                debouncer.should_execute(),
                "Should execute after debounce period elapses"
            );
        }
    }

    // Feature: performance, Property 3: Debounce state consistency
    // *For any* debouncer state, if `pending_execution` is true and `should_execute()`
    // returns true, then after calling `mark_executed()`, `pending_execution` should
    // be false and `should_execute()` should return false.
    // **Validates: Requirements 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_debounce_state_consistency(
            // Generate random number of schedule/execute cycles
            num_cycles in 1usize..=5,
        ) {
            let mut debouncer = Debouncer::new();

            for _ in 0..num_cycles {
                // Schedule execution
                debouncer.schedule_execution();

                // Verify pending is set
                prop_assert!(
                    debouncer.has_pending(),
                    "has_pending should be true after schedule_execution"
                );

                // Wait for debounce period
                thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));

                // Verify should_execute is true
                prop_assert!(
                    debouncer.should_execute(),
                    "should_execute should be true after debounce period"
                );

                // Mark as executed
                debouncer.mark_executed();

                // Verify state is cleared
                prop_assert!(
                    !debouncer.has_pending(),
                    "has_pending should be false after mark_executed"
                );
                prop_assert!(
                    !debouncer.should_execute(),
                    "should_execute should be false after mark_executed"
                );
            }
        }
    }
}
