//! AI-specific debouncer for API requests
//!
//! Delays API calls until user input stabilizes to reduce unnecessary requests.
//! Uses a configurable delay (default 1000ms) that is longer than the query debouncer
//! to account for the cost of API calls.

use std::time::{Duration, Instant};

/// Manages debounced AI API request timing.
///
/// Tracks when the last input occurred and whether there's a pending request.
/// The debouncer delays requests until the configured period has elapsed since
/// the last input, allowing rapid keystrokes to be batched into a single API call.
#[derive(Debug)]
pub struct AiDebouncer {
    /// Debounce delay in milliseconds
    delay_ms: u64,
    /// Timestamp of the last input that triggered a debounce
    last_input_time: Option<Instant>,
    /// Whether there's a pending request waiting for debounce to expire
    pending: bool,
}

// TODO: Remove #[allow(dead_code)] when debouncer is integrated in future phases
#[allow(dead_code)] // Phase 1: Scaffolded for future use
impl AiDebouncer {
    /// Creates a new debouncer with the specified delay.
    ///
    /// # Arguments
    /// * `delay_ms` - Debounce delay in milliseconds (default 1000ms)
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            last_input_time: None,
            pending: false,
        }
    }

    /// Schedule an API request after the debounce delay.
    ///
    /// Records the current time and marks a request as pending.
    /// If called multiple times in rapid succession, each call resets the timer.
    pub fn schedule(&mut self) {
        self.last_input_time = Some(Instant::now());
        self.pending = true;
    }

    /// Cancel any pending request.
    ///
    /// Clears the pending flag and resets the timer.
    pub fn cancel(&mut self) {
        self.pending = false;
        self.last_input_time = None;
    }

    /// Check if the debounce period has elapsed and a request should be made.
    ///
    /// Returns `true` if:
    /// - There is a pending request, AND
    /// - At least `delay_ms` milliseconds have elapsed since the last input
    pub fn is_ready(&self) -> bool {
        if !self.pending {
            return false;
        }

        match self.last_input_time {
            Some(last_time) => {
                let elapsed = last_time.elapsed();
                elapsed >= Duration::from_millis(self.delay_ms)
            }
            None => false,
        }
    }

    /// Reset the debounce timer without changing pending state.
    ///
    /// This is called when new input arrives during the debounce period.
    pub fn reset(&mut self) {
        if self.pending {
            self.last_input_time = Some(Instant::now());
        }
    }

    /// Mark the pending request as complete.
    ///
    /// Clears the pending flag and resets the timer.
    /// Should be called after the API request is made.
    pub fn mark_complete(&mut self) {
        self.pending = false;
        self.last_input_time = None;
    }

    /// Check if there's a pending request.
    pub fn has_pending(&self) -> bool {
        self.pending
    }

    /// Get the configured delay in milliseconds.
    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }
}

impl Default for AiDebouncer {
    fn default() -> Self {
        Self::new(1000)
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
    fn test_new_debouncer() {
        let debouncer = AiDebouncer::new(1000);
        assert_eq!(debouncer.delay_ms(), 1000);
        assert!(!debouncer.has_pending());
        assert!(!debouncer.is_ready());
    }

    #[test]
    fn test_schedule_sets_pending() {
        let mut debouncer = AiDebouncer::new(1000);
        debouncer.schedule();
        assert!(debouncer.has_pending());
    }

    #[test]
    fn test_is_ready_false_immediately() {
        let mut debouncer = AiDebouncer::new(50);
        debouncer.schedule();
        // Immediately after scheduling, is_ready should be false
        assert!(!debouncer.is_ready());
    }

    #[test]
    fn test_is_ready_true_after_delay() {
        let mut debouncer = AiDebouncer::new(50);
        debouncer.schedule();

        // Wait for debounce period to elapse
        thread::sleep(Duration::from_millis(60));

        assert!(debouncer.is_ready());
    }

    #[test]
    fn test_cancel_clears_state() {
        let mut debouncer = AiDebouncer::new(50);
        debouncer.schedule();
        debouncer.cancel();

        assert!(!debouncer.has_pending());
        assert!(!debouncer.is_ready());
    }

    #[test]
    fn test_mark_complete_clears_state() {
        let mut debouncer = AiDebouncer::new(50);
        debouncer.schedule();

        thread::sleep(Duration::from_millis(60));
        assert!(debouncer.is_ready());

        debouncer.mark_complete();

        assert!(!debouncer.has_pending());
        assert!(!debouncer.is_ready());
    }

    #[test]
    fn test_reset_restarts_timer() {
        let mut debouncer = AiDebouncer::new(50);
        debouncer.schedule();

        // Wait half the debounce period
        thread::sleep(Duration::from_millis(25));

        // Reset the timer
        debouncer.reset();

        // Should not be ready yet (timer was reset)
        assert!(!debouncer.is_ready());

        // Wait another half period
        thread::sleep(Duration::from_millis(30));

        // Still should not be ready
        assert!(!debouncer.is_ready());

        // Wait remaining time
        thread::sleep(Duration::from_millis(30));

        // Now should be ready
        assert!(debouncer.is_ready());
    }

    #[test]
    fn test_default() {
        let debouncer = AiDebouncer::default();
        assert_eq!(debouncer.delay_ms(), 1000);
        assert!(!debouncer.has_pending());
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant, Property 13: Debounce timing**
    // *For any* sequence of inputs within the debounce period, exactly one API request
    // should be made after the debounce period expires.
    // **Validates: Requirements 5.1, 5.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_debounce_timing(
            num_inputs in 1usize..=10,
            delay_ms in 50u64..100u64
        ) {
            let mut debouncer = AiDebouncer::new(delay_ms);

            // Simulate rapid inputs (each within debounce period)
            for _ in 0..num_inputs {
                debouncer.schedule();
                // Small delay but less than debounce period
                thread::sleep(Duration::from_millis(5));
            }

            // Immediately after rapid inputs, should NOT be ready
            prop_assert!(
                !debouncer.is_ready(),
                "Should not be ready immediately after rapid inputs"
            );

            // Should have pending request
            prop_assert!(
                debouncer.has_pending(),
                "Should have pending request after scheduling"
            );

            // Wait for full debounce period after last input
            thread::sleep(Duration::from_millis(delay_ms + 20));

            // Now should be ready
            prop_assert!(
                debouncer.is_ready(),
                "Should be ready after debounce period elapses"
            );

            // Mark complete
            debouncer.mark_complete();

            // Should no longer be ready or pending
            prop_assert!(
                !debouncer.is_ready(),
                "Should not be ready after mark_complete"
            );
            prop_assert!(
                !debouncer.has_pending(),
                "Should not have pending after mark_complete"
            );
        }
    }

    // **Feature: ai-assistant, Property 14: Debounce timer reset**
    // *For any* pending debounce timer, a new input should reset the timer to the
    // full debounce period.
    // **Validates: Requirements 5.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_debounce_timer_reset(
            delay_ms in 50u64..100u64,
            num_resets in 1usize..=5
        ) {
            let mut debouncer = AiDebouncer::new(delay_ms);

            // Initial schedule
            debouncer.schedule();

            for _ in 0..num_resets {
                // Wait less than the debounce period
                thread::sleep(Duration::from_millis(delay_ms / 3));

                // Should not be ready yet
                prop_assert!(
                    !debouncer.is_ready(),
                    "Should not be ready before debounce period"
                );

                // Reset the timer (simulating new input)
                debouncer.reset();
            }

            // After all resets, should still not be ready
            prop_assert!(
                !debouncer.is_ready(),
                "Should not be ready immediately after reset"
            );

            // Wait for full debounce period
            thread::sleep(Duration::from_millis(delay_ms + 20));

            // Now should be ready
            prop_assert!(
                debouncer.is_ready(),
                "Should be ready after full debounce period from last reset"
            );
        }
    }

    // Additional property: State consistency
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_state_consistency(
            delay_ms in 50u64..200u64,
            num_cycles in 1usize..=3
        ) {
            let mut debouncer = AiDebouncer::new(delay_ms);

            for _ in 0..num_cycles {
                // Schedule
                debouncer.schedule();
                prop_assert!(debouncer.has_pending());

                // Wait for ready
                thread::sleep(Duration::from_millis(delay_ms + 20));
                prop_assert!(debouncer.is_ready());

                // Complete
                debouncer.mark_complete();
                prop_assert!(!debouncer.has_pending());
                prop_assert!(!debouncer.is_ready());
            }
        }
    }
}
