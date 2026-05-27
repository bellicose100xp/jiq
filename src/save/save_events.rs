use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::Input;

use super::save_io::{current_timestamp, write_atomic};
use super::save_state::{SaveMode, WriteOutcome};
use crate::app::App;

pub fn open_save_popup(app: &mut App) {
    let result = current_result_text(app);
    if result.is_none() {
        app.notification.show("Nothing to save");
        return;
    }
    app.save.open(current_timestamp());
    app.autocomplete.hide();
    app.history.close();
}

pub fn handle_save_popup_key(app: &mut App, key: KeyEvent) {
    match app.save.mode().clone() {
        SaveMode::Closed => {}
        SaveMode::EnterFilename => handle_enter_filename(app, key),
    }
}

fn handle_enter_filename(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.save.close();
        return;
    }
    if key.code == KeyCode::Enter && !key.modifiers.contains(KeyModifiers::SHIFT) {
        attempt_write_from_filename(app);
        return;
    }
    let input: Input = key.into();
    if app.save.filename_mut().input(input) {
        app.save.mark_filename_edited();
    }
}

fn attempt_write_from_filename(app: &mut App) {
    match app.save.prepare_write() {
        WriteOutcome::ReadyToWrite(path) => write_to_path(app, &path),
        WriteOutcome::Error(err) => {
            app.notification
                .show_error(&format!("Save failed: {}", err));
        }
    }
}

fn write_to_path(app: &mut App, path: &std::path::Path) {
    let result = match current_result_text(app) {
        Some(text) => text,
        None => {
            app.notification.show("Nothing to save");
            app.save.close();
            return;
        }
    };

    match write_atomic(path, &result) {
        Ok(canonical) => {
            app.notification
                .show(&format!("Saved to {}", canonical.display()));
            app.save.close();
        }
        Err(err) => {
            app.notification
                .show_error(&format!("Save failed: {}", err));
        }
    }
}

fn current_result_text(app: &App) -> Option<String> {
    let query_state = app.query.as_ref()?;
    let text_arc = query_state.last_successful_result_unformatted.as_ref()?;
    let text = text_arc.as_ref();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
#[path = "save_events_tests.rs"]
mod save_events_tests;
