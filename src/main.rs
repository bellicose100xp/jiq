use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::path::PathBuf;

mod app;
mod autocomplete;
mod editor;
mod error;
mod input;
mod query;
mod syntax;

use app::{App, OutputMode};
use error::JiqError;
use input::reader::InputReader;
use query::executor::JqExecutor;

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
    let app = run(terminal, json_input.clone())?;

    // Restore terminal (automatic cleanup)
    ratatui::restore();

    // Output results AFTER terminal is restored
    handle_output(&app, &json_input)?;

    Ok(())
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

fn run(mut terminal: DefaultTerminal, json_input: String) -> Result<App> {
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

    Ok(app)
}

/// Handle output after terminal is restored
fn handle_output(app: &App, json_input: &str) -> Result<()> {
    match app.output_mode() {
        Some(OutputMode::Results) => {
            // Execute final query and output results
            let executor = JqExecutor::new(json_input.to_string());
            match executor.execute(app.query()) {
                Ok(result) => println!("{}", result),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Some(OutputMode::Query) => {
            // Output just the query string
            println!("{}", app.query());
        }
        None => {
            // No output mode (exited with Ctrl+C or q)
        }
    }

    Ok(())
}
