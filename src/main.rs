use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::path::PathBuf;

mod app;
mod error;
mod input;

use app::App;
use error::JiqError;
use input::reader::InputReader;

/// Interactive JSON query tool
#[derive(Parser, Debug)]
#[command(version, about = "Interactive JSON query tool with real-time filtering using jq")]
struct Args {
    /// Input JSON file (if not provided, reads from stdin)
    input: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Install color-eyre panic hook for better error messages
    color_eyre::install()?;

    // Parse CLI arguments
    let args = Args::parse();

    // Validate jq binary exists
    validate_jq_exists()?;

    // Read JSON input
    let json_input = match InputReader::read_json(args.input.as_deref()) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error reading JSON: {:?}", e);
            return Err(e.into());
        }
    };

    // Initialize terminal (handles raw mode, alternate screen, etc.)
    let terminal = ratatui::init();

    // Run the application with JSON input
    let result = run(terminal, json_input);

    // Restore terminal (automatic cleanup)
    ratatui::restore();

    result
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

fn run(mut terminal: DefaultTerminal, json_input: String) -> Result<()> {
    let mut app = App::new(json_input);

    loop {
        // Render the UI
        terminal.draw(|frame| app.render(frame))?;

        // Handle events (all logic in app.rs now)
        app.handle_events()?;

        // Check if we should exit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
