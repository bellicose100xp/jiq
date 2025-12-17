use crate::app::App;
use crate::stats::parser::StatsParser;
use crate::stats::types::ResultStats;

pub fn update_stats_from_app(app: &mut App) {
    if let Some(result) = &app.query.last_successful_result_unformatted {
        app.stats.compute(result);
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsState {
    stats: Option<ResultStats>,
}

impl StatsState {
    pub fn compute(&mut self, result: &str) {
        let trimmed = result.trim();
        if trimmed.is_empty() {
            return;
        }
        self.stats = Some(StatsParser::parse(result));
    }

    pub fn display(&self) -> Option<String> {
        self.stats.as_ref().map(|s| s.to_string())
    }

    #[cfg(test)]
    pub fn stats(&self) -> Option<&ResultStats> {
        self.stats.as_ref()
    }
}

#[cfg(test)]
#[path = "stats_state_tests.rs"]
mod stats_state_tests;
