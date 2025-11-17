use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

mod app;

use app::App;

fn main() -> Result<()> {
    // Install color-eyre panic hook for better error messages
    color_eyre::install()?;

    // Initialize terminal (handles raw mode, alternate screen, etc.)
    let terminal = ratatui::init();

    // Run the application
    let result = run(terminal);

    // Restore terminal (automatic cleanup)
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let app = App::new();

    loop {
        // Render the UI
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if let Event::Key(key) = event::read()? {
            // Only process key press events (avoid duplicates)
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
