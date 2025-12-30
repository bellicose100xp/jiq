use crate::app::app_state::App;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

pub fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

pub fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal.draw(|f| app.render(f)).unwrap();
    terminal.backend().to_string()
}

#[cfg(test)]
mod basic_ui_tests;

#[cfg(test)]
mod popup_tests;

#[cfg(test)]
mod search_tests;

#[cfg(test)]
mod processing_tests;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod error_handling_tests;

#[cfg(test)]
mod result_state_tests;
