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
mod json;
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
use input::{FileLoader, reader::InputReader};
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

    // Create App with deferred loading for file input or synchronous loading for stdin
    let app = if let Some(path) = args.input {
        // Validate file exists and contains valid JSON before entering TUI mode
        // This allows us to exit with error code for invalid files (for tests)
        if let Err(e) = validate_json_file(&path) {
            eprintln!("Error: {}", e);
            return Err(e.into());
        }

        // Initialize terminal (handles raw mode, alternate screen, bracketed paste)
        let terminal = init_terminal()?;

        // File input: use deferred loading for instant UI
        let loader = FileLoader::spawn_load(path);
        let app = App::new_with_loader(loader, &config_result.config);
        run(terminal, app, config_result)?
    } else {
        // Stdin input: use synchronous loading
        let json_input = match InputReader::read_json(None) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error reading JSON: {:?}", e);
                return Err(e.into());
            }
        };

        // Initialize terminal (handles raw mode, alternate screen, bracketed paste)
        let terminal = init_terminal()?;

        let app = App::new(json_input.clone(), &config_result.config);
        run(terminal, app, config_result)?
    };

    // Restore terminal (cleanup raw mode, alternate screen, bracketed paste)
    restore_terminal()?;

    // Output results AFTER terminal is restored
    handle_output(&app)?;

    Ok(())
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

/// Validate that a JSON file exists and contains valid JSON
///
/// Performs a quick synchronous check before entering TUI mode.
/// This allows the app to exit with an error code for invalid files (needed for tests).
fn validate_json_file(path: &PathBuf) -> Result<(), JiqError> {
    use std::fs::File;
    use std::io::Read;

    // Check if file exists and can be opened
    let mut file = File::open(path)?;

    // Read file contents
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Validate JSON syntax
    serde_json::from_str::<serde_json::Value>(&contents)
        .map_err(|e| JiqError::InvalidJson(e.to_string()))?;

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
    mut app: App,
    config_result: config::ConfigResult,
) -> Result<App> {
    // Show config warning if there was one
    if let Some(warning) = config_result.warning {
        app.notification.show_warning(&warning);
    }

    // Set up AI worker thread if AI is enabled
    // Requirements 1.1, 1.3, 4.1
    setup_ai_worker(&mut app, &config_result.config);

    // Trigger initial AI request if AI popup is visible on startup
    if app.ai.visible && app.ai.enabled && app.ai.configured {
        app.trigger_ai_request();
    }

    loop {
        // Poll file loader before rendering
        app.poll_file_loader();

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
fn setup_ai_worker(app: &mut App, config: &config::Config) {
    // Warn if AI is explicitly enabled but not properly configured
    if config.ai.enabled && !app.ai.configured {
        app.notification
            .show_warning("AI enabled but not configured. Add provider credentials to config.");
    }

    // Only set up worker if AI is configured (has provider credentials)
    // Worker is needed regardless of `enabled` because user can toggle with Ctrl+A
    if !app.ai.configured {
        return;
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
fn handle_output(app: &App) -> Result<()> {
    match app.output_mode() {
        Some(OutputMode::Results) => {
            // Execute final query and output results
            // Only output if query is available
            if let Some(query_state) = &app.query {
                let json_input = query_state.executor.json_input();
                let executor = JqExecutor::new(json_input.to_string());
                match executor.execute(app.query()) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
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
