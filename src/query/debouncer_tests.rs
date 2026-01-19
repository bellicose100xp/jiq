//! Tests for debouncer

use super::*;
use proptest::prelude::*;

#[test]
fn test_new_debouncer_has_no_pending() {
    let debouncer = Debouncer::new();
    assert!(!debouncer.has_pending());
    assert!(!debouncer.should_execute_at(0));
}

#[test]
fn test_schedule_execution_sets_pending() {
    let mut debouncer = Debouncer::new();
    debouncer.schedule_execution_at(0);
    assert!(debouncer.has_pending());
}

#[test]
fn test_should_execute_false_immediately_after_schedule() {
    let mut debouncer = Debouncer::new();
    debouncer.schedule_execution_at(0);
    assert!(!debouncer.should_execute_at(0));
}

#[test]
fn test_should_execute_true_after_debounce_period() {
    let mut debouncer = Debouncer::new();
    debouncer.schedule_execution_at(0);
    assert!(debouncer.should_execute_at(TEST_DEBOUNCE_MS + 10));
}

#[test]
fn test_mark_executed_clears_state() {
    let mut debouncer = Debouncer::new();
    debouncer.schedule_execution_at(0);
    assert!(debouncer.should_execute_at(TEST_DEBOUNCE_MS + 10));

    debouncer.mark_executed();
    assert!(!debouncer.has_pending());
    assert!(!debouncer.should_execute_at(TEST_DEBOUNCE_MS + 10));
}

#[test]
fn test_schedule_resets_timer() {
    let mut debouncer = Debouncer::new();

    // Schedule at time 0
    debouncer.schedule_execution_at(0);
    // At time DEBOUNCE_MS/2, should not execute yet
    assert!(!debouncer.should_execute_at(TEST_DEBOUNCE_MS / 2));

    // Reschedule at time DEBOUNCE_MS/2
    debouncer.schedule_execution_at(TEST_DEBOUNCE_MS / 2);
    // At time DEBOUNCE_MS, should not execute (only half the debounce period since reschedule)
    assert!(!debouncer.should_execute_at(TEST_DEBOUNCE_MS));

    // At time DEBOUNCE_MS + DEBOUNCE_MS/2 + 10, should execute
    assert!(debouncer.should_execute_at(TEST_DEBOUNCE_MS + TEST_DEBOUNCE_MS / 2 + 10));
}

#[test]
fn test_default_impl() {
    let debouncer = Debouncer::default();
    assert!(!debouncer.has_pending());
    assert!(!debouncer.should_execute_at(0));
}

// Feature: performance, Property 2: Debounce timer reset on input
// *For any* sequence of keystrokes where each keystroke occurs within 150ms
// of the previous one, the debouncer should reset its timer on each keystroke
// and not trigger execution until 150ms after the final keystroke.
// **Validates: Requirements 2.1, 2.2, 2.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_debounce_timer_reset_on_input(num_inputs in 2usize..=10) {
        let mut debouncer = Debouncer::new();
        let mut current_time: u64 = 0;

        // Simulate rapid inputs, each 5ms apart
        for _ in 0..num_inputs {
            debouncer.schedule_execution_at(current_time);
            current_time += 5;
        }

        // Immediately after rapid inputs, should not execute
        prop_assert!(
            !debouncer.should_execute_at(current_time),
            "Should not execute immediately after rapid inputs"
        );

        prop_assert!(
            debouncer.has_pending(),
            "Should have pending execution after scheduling"
        );

        // After debounce period elapses from last input, should execute
        let final_check_time = current_time + TEST_DEBOUNCE_MS + 10;
        prop_assert!(
            debouncer.should_execute_at(final_check_time),
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
        let mut current_time: u64 = 0;

        for _ in 0..num_cycles {
            debouncer.schedule_execution_at(current_time);

            prop_assert!(
                debouncer.has_pending(),
                "has_pending should be true after schedule_execution"
            );

            // Advance time past debounce period
            current_time += TEST_DEBOUNCE_MS + 10;

            prop_assert!(
                debouncer.should_execute_at(current_time),
                "should_execute should be true after debounce period"
            );

            debouncer.mark_executed();

            prop_assert!(
                !debouncer.has_pending(),
                "has_pending should be false after mark_executed"
            );
            prop_assert!(
                !debouncer.should_execute_at(current_time),
                "should_execute should be false after mark_executed"
            );

            // Advance time for next cycle
            current_time += 10;
        }
    }
}
