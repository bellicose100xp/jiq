use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 150;
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
#[path = "debouncer_tests.rs"]
mod debouncer_tests;
