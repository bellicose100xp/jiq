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
mod snippets;
mod stats;
mod syntax_highlight;
#[cfg(test)]
mod test_utils;
mod tooltip;
mod widgets;

use app::{App, OutputMode};
use error::JiqError;
use input::FileLoader;
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

    color_eyre::install()?;

    // Load config early to avoid defaults during app initialization
    let config_result = config::load_config();

    let args = Args::parse();

    validate_jq_exists()?;

    let terminal = init_terminal()?;

    // Deferred loading prevents blocking on large files/stdin
    let loader = if let Some(path) = args.input {
        FileLoader::spawn_load(path)
    } else {
        FileLoader::spawn_load_stdin()
    };

    let app = App::new_with_loader(loader, &config_result.config);
    let result = run(terminal, app, config_result);

    restore_terminal()?;
    let app = result?;

    // Output after terminal restore to prevent corruption
    handle_output(&app)?;

    #[cfg(debug_assertions)]
    log::debug!("=== JIQ DEBUG SESSION ENDED ===");

    Ok(())
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

/// Initialize terminal with raw mode, alternate screen, and bracketed paste
fn init_terminal() -> Result<DefaultTerminal> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableBracketedPaste, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        hook(info);
    }));

    enable_raw_mode()?;

    // If any subsequent operations fail, ensure raw mode is disabled
    match execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste) {
        Ok(_) => {}
        Err(e) => {
            let _ = disable_raw_mode();
            return Err(e.into());
        }
    }

    match ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout())) {
        Ok(terminal) => Ok(terminal),
        Err(e) => {
            let _ = execute!(stdout(), DisableBracketedPaste, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            Err(e.into())
        }
    }
}

/// Restore terminal to normal state
fn restore_terminal() -> Result<()> {
    let _ = execute!(stdout(), DisableBracketedPaste, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    mut app: App,
    config_result: config::ConfigResult,
) -> Result<App> {
    if let Some(warning) = config_result.warning {
        app.notification.show_warning(&warning);
    }

    // Requirements 1.1, 1.3, 4.1
    setup_ai_worker(&mut app, &config_result.config);

    // Trigger initial request when AI popup visible on startup
    if app.ai.visible && app.ai.enabled && app.ai.configured {
        app.trigger_ai_request();
    }

    loop {
        // Poll before render to load data from background thread
        app.poll_file_loader();

        if app.should_render() {
            terminal.draw(|frame| app.render(frame))?;
            app.clear_dirty();
        }

        app.handle_events()?;

        if app.should_quit() {
            break;
        }
    }

    Ok(app)
}

/// Set up the AI worker thread and channels
fn setup_ai_worker(app: &mut App, config: &config::Config) {
    if config.ai.enabled && !app.ai.configured {
        app.notification
            .show_warning("AI enabled but not configured. Add provider credentials to config.");
    }

    // Worker needed even when disabled to support Ctrl+A toggle
    if !app.ai.configured {
        return;
    }

    let (request_tx, request_rx) = std::sync::mpsc::channel();
    let (response_tx, response_rx) = std::sync::mpsc::channel();
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
                let cancel_token = tokio_util::sync::CancellationToken::new();
                match executor.execute_with_cancel(app.query(), &cancel_token) {
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
