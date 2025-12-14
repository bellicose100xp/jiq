use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::stdout;
use std::path::PathBuf;

mod ai;
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
mod search;
mod stats;
mod syntax_highlight;
#[cfg(test)]
mod test_utils;
mod tooltip;
mod widgets;

use app::{App, OutputMode};
use error::JiqError;
use input::reader::InputReader;
use query::executor::JqExecutor;

/// Interactive JSON query tool
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Interactive JSON query tool with real-time filtering using jq"
)]
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

    // Initialize terminal (handles raw mode, alternate screen, bracketed paste)
    let terminal = init_terminal()?;

    // Run the application with JSON input and config
    let app = run(terminal, json_input.clone(), config_result)?;

    // Restore terminal (cleanup raw mode, alternate screen, bracketed paste)
    restore_terminal()?;

    // Output results AFTER terminal is restored
    handle_output(&app, &json_input)?;

    Ok(())
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

/// Initialize terminal with raw mode, alternate screen, and bracketed paste
fn init_terminal() -> Result<DefaultTerminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste)?;
    let terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

/// Restore terminal to normal state
fn restore_terminal() -> Result<()> {
    execute!(stdout(), DisableBracketedPaste, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    json_input: String,
    config_result: config::ConfigResult,
) -> Result<App> {
    let mut app = App::new(json_input, &config_result.config);

    // Show config warning if there was one
    if let Some(warning) = config_result.warning {
        app.notification.show_warning(&warning);
    }

    // Set up AI worker thread if AI is enabled
    // Requirements 1.1, 1.3, 4.1
    setup_ai_worker(&mut app, &config_result.config);

    // Trigger initial AI request if AI popup is visible on startup
    // Requirements 8.4: WHEN the AI_Popup becomes visible THEN the AI_Assistant SHALL
    // immediately analyze the current query context and provide suggestions
    if app.ai.visible && app.ai.enabled && app.ai.configured {
        let query = app.input.query().to_string();
        let cursor_pos = app.input.textarea.cursor().1;
        let json_input = app.query.executor.json_input().to_string();
        ai::ai_events::handle_execution_result(
            &mut app.ai,
            &app.query.result,
            false, // Don't auto-show (already visible)
            &query,
            cursor_pos,
            &json_input,
        );
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

/// Set up the AI worker thread and channels
///
/// Creates request/response channels and spawns the worker thread.
/// Also validates the config and shows a warning if AI is enabled but not configured.
///
/// # Requirements
/// - 1.1: WHEN a user adds an `[ai]` section with `enabled = true` and valid credentials
///        THEN the AI_Assistant SHALL initialize successfully
/// - 1.3: WHEN the `[ai.anthropic]` section has a missing or empty `api_key` field
///        THEN the AI_Assistant SHALL display a configuration error message
/// - 4.1: WHEN the AI provider sends a streaming response THEN the AI_Popup
///        SHALL display text incrementally as chunks arrive
fn setup_ai_worker(app: &mut App, config: &config::Config) {
    // Only set up worker if AI is enabled
    if !config.ai.enabled {
        return;
    }

    // Validate config and warn if API key is missing
    if config
        .ai
        .anthropic
        .api_key
        .as_ref()
        .map_or(true, |k| k.trim().is_empty())
    {
        app.notification.show_warning(
            "AI enabled but API key missing. Add api_key to [ai.anthropic] in config.",
        );
    }

    // Create channels for communication with worker thread
    let (request_tx, request_rx) = std::sync::mpsc::channel();
    let (response_tx, response_rx) = std::sync::mpsc::channel();

    // Set channel handles in AiState
    app.ai.set_channels(request_tx, response_rx);

    // Spawn the worker thread
    ai::worker::spawn_worker(&config.ai, request_rx, response_tx);
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
