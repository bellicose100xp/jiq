use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 50;
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
    pub fn new() -> Self {
        Self {
            last_input_time: None,
            pending_execution: false,
        }
    }

    pub fn schedule_execution(&mut self) {
        self.last_input_time = Some(Instant::now());
        self.pending_execution = true;
    }

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

    pub fn mark_executed(&mut self) {
        self.pending_execution = false;
        self.last_input_time = None;
    }

    pub fn has_pending(&self) -> bool {
        self.pending_execution
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::thread;

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
        assert!(!debouncer.should_execute());
    }

    #[test]
    fn test_should_execute_true_after_debounce_period() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();
        thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));
        assert!(debouncer.should_execute());
    }

    #[test]
    fn test_mark_executed_clears_state() {
        let mut debouncer = Debouncer::new();
        debouncer.schedule_execution();
        thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));
        assert!(debouncer.should_execute());

        debouncer.mark_executed();
        assert!(!debouncer.has_pending());
        assert!(!debouncer.should_execute());
    }

    #[test]
    fn test_schedule_resets_timer() {
        let mut debouncer = Debouncer::new();

        debouncer.schedule_execution();
        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2));
        debouncer.schedule_execution();
        assert!(!debouncer.should_execute());

        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2));
        assert!(!debouncer.should_execute());

        thread::sleep(Duration::from_millis(DEBOUNCE_MS / 2 + 10));
        assert!(debouncer.should_execute());
    }

    #[test]
    fn test_default_impl() {
        let debouncer = Debouncer::default();
        assert!(!debouncer.has_pending());
        assert!(!debouncer.should_execute());
    }

    // Feature: performance, Property 2: Debounce timer reset on input
    // *For any* sequence of keystrokes where each keystroke occurs within 50ms
    // of the previous one, the debouncer should reset its timer on each keystroke
    // and not trigger execution until 50ms after the final keystroke.
    // **Validates: Requirements 2.1, 2.2, 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_debounce_timer_reset_on_input(num_inputs in 2usize..=10) {
            let mut debouncer = Debouncer::new();

            for _ in 0..num_inputs {
                debouncer.schedule_execution();
                thread::sleep(Duration::from_millis(5));
            }

            prop_assert!(
                !debouncer.should_execute(),
                "Should not execute immediately after rapid inputs"
            );

            prop_assert!(
                debouncer.has_pending(),
                "Should have pending execution after scheduling"
            );

            thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));

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
        fn prop_debounce_state_consistency(num_cycles in 1usize..=5) {
            let mut debouncer = Debouncer::new();

            for _ in 0..num_cycles {
                debouncer.schedule_execution();

                prop_assert!(
                    debouncer.has_pending(),
                    "has_pending should be true after schedule_execution"
                );

                thread::sleep(Duration::from_millis(DEBOUNCE_MS + 10));

                prop_assert!(
                    debouncer.should_execute(),
                    "should_execute should be true after debounce period"
                );

                debouncer.mark_executed();

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
