use std::time::Instant;

const DEBOUNCE_MS: u64 = 150;

#[cfg(test)]
pub const TEST_DEBOUNCE_MS: u64 = DEBOUNCE_MS;

fn system_time_ms() -> u64 {
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

#[derive(Debug, Default)]
pub struct Debouncer {
    scheduled_at_ms: Option<u64>,
    pending_execution: bool,
}

impl Debouncer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn schedule_execution(&mut self) {
        self.schedule_execution_at(system_time_ms());
    }

    pub fn schedule_execution_at(&mut self, current_time_ms: u64) {
        self.scheduled_at_ms = Some(current_time_ms);
        self.pending_execution = true;
    }

    pub fn should_execute(&self) -> bool {
        self.should_execute_at(system_time_ms())
    }

    pub fn should_execute_at(&self, current_time_ms: u64) -> bool {
        if !self.pending_execution {
            return false;
        }
        match self.scheduled_at_ms {
            Some(scheduled) => current_time_ms >= scheduled + DEBOUNCE_MS,
            None => false,
        }
    }

    pub fn mark_executed(&mut self) {
        self.pending_execution = false;
        self.scheduled_at_ms = None;
    }

    pub fn has_pending(&self) -> bool {
        self.pending_execution
    }
}

#[cfg(test)]
#[path = "debouncer_tests.rs"]
mod debouncer_tests;
