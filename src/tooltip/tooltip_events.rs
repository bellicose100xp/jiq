use super::tooltip_state::TooltipState;

pub fn handle_tooltip_toggle(state: &mut TooltipState) -> bool {
    state.toggle();
    true
}

#[cfg(test)]
#[path = "tooltip_events_tests.rs"]
mod tooltip_events_tests;
