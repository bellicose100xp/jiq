use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::path::PathBuf;

mod app;
mod autocomplete;
mod clipboard;
mod config;
mod editor;
mod error;
mod help;
mod history;
mod input;
mod notification;
mod query;
mod results;
mod scroll;
mod syntax_highlight;
mod widgets;

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
    // Initialize logger (only in debug builds)
    // Writes to /tmp/jiq-debug.log at DEBUG level
    #[cfg(debug_assertions)]
    {
        use std::io::Write;

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/jiq-debug.log")
            .expect("Failed to open /tmp/jiq-debug.log");

        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .format(|buf, record| {
                use std::time::SystemTime;
                let datetime: chrono::DateTime<chrono::Local> = SystemTime::now().into();
                writeln!(
                    buf,
                    "[{}] [{}] {}",
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f"),
                    record.level(),
                    record.args()
                )
            })
            .init();

        log::debug!("=== JIQ DEBUG SESSION STARTED ===");
    }

    // Install color-eyre panic hook for better error messages
    color_eyre::install()?;

    // Load configuration early in startup
    let config_result = config::load_config();

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

    // Run the application with JSON input and config
    let app = run(terminal, json_input.clone(), config_result)?;

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

fn run(
    mut terminal: DefaultTerminal,
    json_input: String,
    config_result: config::ConfigResult,
) -> Result<App> {
    let mut app = App::new(json_input, config_result.config.clipboard.backend);

    // Show config warning if there was one
    if let Some(warning) = config_result.warning {
        app.notification.show_warning(&warning);
    }

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
