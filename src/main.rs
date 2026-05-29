use clap::Parser;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::stdout;
use std::path::PathBuf;

mod ai;
mod app;
mod autocomplete;
mod bench_script;
mod clipboard;
mod config;
mod editor;
mod error;
mod help;
mod history;
mod input;
mod json;
mod json_path;
mod layout;
mod notification;
mod path_at_cursor;
mod path_at_cursor_apply;
pub mod perf;
mod query;
mod query_undo;
mod results;
mod save;
mod scroll;
mod search;
mod snippets;
mod stats;
mod str_utils;
mod syntax_highlight;
#[cfg(test)]
mod test_utils;
pub mod theme;
mod tooltip;
mod widgets;

use app::{App, OutputMode};
use error::JiqError;
use input::loader::peek_clipboard;
use input::{FileLoader, PasteRecoveryState, SourcePickerState};
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

    /// Force reading JSON from the system clipboard. Mutually exclusive
    /// with --paste; cannot be combined with piped stdin (the
    /// invocation is contradictory and is rejected with exit code 2).
    #[arg(long, conflicts_with = "paste")]
    clipboard: bool,

    /// Drop straight into manual-paste mode without reading the
    /// clipboard. Mutually exclusive with --clipboard; cannot be
    /// combined with piped stdin (rejected with exit code 2).
    #[arg(long, conflicts_with = "clipboard")]
    paste: bool,

    /// Enable debug logging to /tmp/jiq-debug.log
    #[arg(long)]
    debug: bool,

    /// Replay a benchmark script of keystrokes (for perf measurement).
    /// See tests/perf_scripts/README.md for the format.
    #[arg(long, value_name = "PATH")]
    bench_script: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    init_logger(args.debug);

    color_eyre::install()?;

    // Wrap the panic hook so any perf timings collected before the panic
    // still get flushed to the debug log. dump_summary() is idempotent.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        perf::dump_summary();
        prev_hook(info);
    }));

    // Load config early to avoid defaults during app initialization
    let config_result = config::load_config();

    validate_jq_exists()?;
    log::debug!("jq binary found in PATH");

    // H1 hard-error: an explicit source flag combined with ANY other
    // source is contradictory. The user typed `--clipboard` / `--paste`
    // to override the default; a file argument or piped stdin
    // contradicts the override. Refuse to launch rather than silently
    // dropping one.
    //
    // (clap already rejects --clipboard + --paste at parse time via
    // conflicts_with, so we don't need to check that combination here.)
    //
    // We deliberately do NOT trip on `file + piped stdin` without a
    // flag. That preserves today's behavior (file arg wins) for
    // scripts/CI invocations where stdin happens to be redirected
    // (e.g. `jiq foo.json < /dev/null` from cron).
    //
    // Runs *before* init_terminal so the message lands on the user's
    // normal terminal, not inside the alt screen.
    let has_file = args.input.is_some();
    let has_pipe = !std::io::IsTerminal::is_terminal(&std::io::stdin());
    let flag: Option<&str> = if args.clipboard {
        Some("--clipboard")
    } else if args.paste {
        Some("--paste")
    } else {
        None
    };
    if let Some(f) = flag
        && (has_file || has_pipe)
    {
        let mut got = Vec::with_capacity(3);
        if has_file {
            got.push("a file argument".to_string());
        }
        if has_pipe {
            got.push("piped stdin".to_string());
        }
        got.push(f.to_string());
        print_ambiguous_source_error(&got);
        std::process::exit(2);
    }

    // Resolve the input source before `init_terminal()`. Clipboard
    // reads MUST happen pre-init: the OSC 52 fallback toggles raw mode
    // around its read, and crossterm's `disable_raw_mode()` does not
    // preserve `EnableBracketedPaste`. Doing the read after init would
    // wipe bracketed-paste support for the rest of the session, which
    // makes large pastes inside the recovery textarea arrive byte by
    // byte instead of as a single `Event::Paste`.
    //
    // File and stdin loads stay deferred so a large input never blocks
    // the splash. `--paste` skips the clipboard entirely.
    let pre_input = resolve_pre_input(&args);

    // Load the bench script before init_terminal so a parse error lands on
    // the user's normal terminal, not inside the alt screen.
    let bench_script = match args.bench_script.as_deref() {
        Some(path) => match bench_script::BenchScript::load(path) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(2);
            }
        },
        None => None,
    };

    let terminal = init_terminal()?;
    let app = match pre_input {
        PreInput::Loader(loader) => App::new_with_loader(loader, &config_result.config),
        PreInput::PasteRecovery(state) => {
            App::new_with_paste_recovery(state, &config_result.config)
        }
        PreInput::Picker(state) => App::new_with_source_picker(state, &config_result.config),
    };
    let result = run(terminal, app, config_result, bench_script);

    restore_terminal()?;
    let app = result?;

    // Output after terminal restore to prevent corruption
    handle_output(&app)?;

    log::debug!("=== JIQ DEBUG SESSION ENDED ===");

    Ok(())
}

/// Validate that jq binary exists in PATH
fn validate_jq_exists() -> Result<(), JiqError> {
    which::which("jq").map_err(|_| JiqError::JqNotFound)?;
    Ok(())
}

/// What jiq has lined up before the TUI starts: a deferred file/stdin
/// loader, an immediate paste-recovery editor, or the source picker.
/// Resolved pre-init so any blocking clipboard read happens before
/// crossterm's raw-mode + bracketed-paste handshake.
enum PreInput {
    Loader(FileLoader),
    PasteRecovery(PasteRecoveryState),
    Picker(SourcePickerState),
}

/// Decide which pre-input the user asked for and produce it. Runs
/// before `init_terminal()` because the clipboard read (whether for
/// the picker peek or `--clipboard` direct load) may briefly own raw
/// stdin via the OSC 52 fallback; doing this after init would wipe
/// bracketed-paste support for the rest of the session.
///
/// On bare TTY launches we peek the clipboard once, then choose
/// between the picker and a direct-to-paste shortcut: showing the
/// picker only makes sense when the Clipboard option is actually
/// usable. If the clipboard is unreadable / empty / invalid /
/// primitive, the Clipboard option can't be confirmed anyway, so we
/// skip the chooser entirely and drop straight into the explicit-paste
/// editor with a context line explaining what jiq saw on the
/// clipboard.
fn resolve_pre_input(args: &Args) -> PreInput {
    if args.paste {
        log::debug!("Entering explicit paste mode (--paste)");
        return PreInput::PasteRecovery(PasteRecoveryState::new_explicit());
    }
    if let Some(ref path) = args.input {
        log::debug!("File loader spawned for: {:?}", path);
        return PreInput::Loader(FileLoader::spawn_load(path.clone()));
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        log::debug!("File loader spawned for stdin");
        return PreInput::Loader(FileLoader::spawn_load_stdin());
    }
    if args.clipboard {
        log::debug!("Loading clipboard synchronously (forced via --clipboard)");
        return PreInput::Loader(FileLoader::load_clipboard_blocking());
    }
    log::debug!("Bare TTY launch — peeking clipboard for source picker");
    let peek = peek_clipboard();
    if peek.is_usable() {
        PreInput::Picker(SourcePickerState::from_peek(peek))
    } else {
        log::debug!("Clipboard not usable; jumping straight to explicit paste");
        PreInput::PasteRecovery(PasteRecoveryState::new_explicit_with_context(
            peek.failure_context(),
        ))
    }
}

/// Render the H1 "ambiguous input source" error to stderr with ANSI
/// colors when stderr is a TTY, plain text otherwise (so redirecting
/// `2>file` produces clean output without escape codes). Colors mirror
/// the rest of jiq's palette: red for the error label, yellow for the
/// rejected sources, dim for shell-comments, bold for the heading.
///
/// `got` is the list of sources jiq detected on this invocation
/// (file argument / piped stdin / `--clipboard` / `--paste`); jiq
/// accepts at most one, so any list of length ≥ 2 trips this error.
fn print_ambiguous_source_error(got: &[String]) {
    let tty = std::io::IsTerminal::is_terminal(&std::io::stderr());

    // ANSI escapes — only emitted when stderr is a TTY.
    let (red, yellow, cyan, dim, bold, reset) = if tty {
        (
            "\x1b[31m", "\x1b[33m", "\x1b[36m", "\x1b[2m", "\x1b[1m", "\x1b[0m",
        )
    } else {
        ("", "", "", "", "", "")
    };

    let got_list = match got.len() {
        0 | 1 => unreachable!("ambiguous error requires at least 2 sources"),
        2 => format!("{yellow}{}{reset} AND {yellow}{}{reset}", got[0], got[1]),
        _ => {
            let last = got.len() - 1;
            let head = got[..last]
                .iter()
                .map(|s| format!("{yellow}{s}{reset}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{head}, AND {yellow}{}{reset}", got[last])
        }
    };

    eprintln!(
        "{bold}{red}error:{reset}{bold} ambiguous input source{reset} — got {got_list} at the same time.\n\n\
         jiq accepts exactly one input source per invocation. Pick one:\n\
         \x20 {cyan}jiq <file>{reset}            {dim}# load from a file{reset}\n\
         \x20 {cyan}cat <file> | jiq{reset}      {dim}# load from piped stdin{reset}\n\
         \x20 {cyan}jiq{reset}                   {dim}# load from system clipboard (default){reset}\n\
         \x20 {cyan}jiq --clipboard{reset}       {dim}# load from system clipboard (explicit){reset}\n\
         \x20 {cyan}jiq --paste{reset}           {dim}# open the manual-paste editor{reset}"
    );
}

/// Initialize terminal with raw mode, alternate screen, and bracketed paste
fn init_terminal() -> Result<DefaultTerminal> {
    log::debug!("Initializing terminal");
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(
            stdout(),
            DisableMouseCapture,
            DisableBracketedPaste,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        hook(info);
    }));

    enable_raw_mode()?;
    log::debug!("Raw mode enabled");

    // If any subsequent operations fail, ensure raw mode is disabled
    match execute!(
        stdout(),
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableMouseCapture
    ) {
        Ok(_) => {
            log::debug!("Alternate screen entered");
        }
        Err(e) => {
            log::error!("Failed to enter alternate screen: {}", e);
            let _ = disable_raw_mode();
            return Err(e.into());
        }
    }

    match ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout())) {
        Ok(terminal) => {
            log::debug!("Terminal backend created");
            Ok(terminal)
        }
        Err(e) => {
            log::error!("Failed to create terminal backend: {}", e);
            let _ = execute!(
                stdout(),
                DisableMouseCapture,
                DisableBracketedPaste,
                LeaveAlternateScreen
            );
            let _ = disable_raw_mode();
            Err(e.into())
        }
    }
}

/// Restore terminal to normal state
fn restore_terminal() -> Result<()> {
    log::debug!("Restoring terminal");
    let _ = execute!(
        stdout(),
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen
    );
    disable_raw_mode()?;
    perf::dump_summary();
    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    mut app: App,
    config_result: config::ConfigResult,
    mut bench_script: Option<bench_script::BenchScript>,
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

    // Set when the bench script finishes its directives. While Some, the
    // run loop is in "drain mode": it keeps pumping handle_events (so the
    // worker thread's responses arrive and its perf timers drop into the
    // tally) until the worker is quiescent, then requests quit.
    let mut bench_drain_started_at: Option<std::time::Instant> = None;

    loop {
        // Poll before render to load data from background thread
        app.poll_file_loader();

        if app.should_render() {
            terminal.draw(|frame| app.render(frame))?;
            app.clear_dirty();
        }

        // Inject any ready bench-script events first so they run through
        // the same dispatcher as real keystrokes.
        if let Some(script) = bench_script.as_mut() {
            while let Some(step) = script.next_step() {
                if let bench_script::Step::Key(key) = step {
                    let _t = perf::Stopwatch::new("inject_key_event");
                    app.inject_key_event(key);
                }
            }
            if script.is_finished() {
                // Enter drain mode: keep the loop running so handle_events
                // pumps worker responses, allowing in-flight preprocessing
                // (and its perf timers) to complete before we quit. Without
                // this the worker's RAII Stopwatches drop AFTER the perf
                // summary is dumped and their samples are lost.
                log::debug!("bench script finished, entering drain mode");
                bench_script = None;
                bench_drain_started_at = Some(std::time::Instant::now());
            }
        } else if let Some(drain_start) = bench_drain_started_at {
            // In drain mode: wait for worker quiescence, then quit. Cap at
            // 30s so a wedged worker can't hang the bench harness.
            let drained = !app.has_pending_query();
            let timed_out = drain_start.elapsed() > std::time::Duration::from_secs(30);
            if drained || timed_out {
                if timed_out {
                    log::warn!("bench drain timed out after 30s with pending work");
                }
                log::debug!("bench drain complete, requesting quit");
                bench_drain_started_at = None;
                app.request_quit();
            }
        }

        {
            let _t = perf::Stopwatch::new("handle_events");
            app.handle_events()?;
        }

        // While a bench script is still running or draining, suppress
        // `should_quit` and clear the flag. Enter sets should_quit=true as
        // a side effect of committing the query (see app_events/global.rs);
        // we don't want that to short-circuit the script's later directives
        // or the drain phase. The drain code above is the only thing that
        // can request quit while in bench mode.
        if (bench_script.is_some() || bench_drain_started_at.is_some()) && app.should_quit() {
            app.cancel_quit();
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(app)
}

/// Set up the AI worker thread and channels
fn setup_ai_worker(app: &mut App, config: &config::Config) {
    log::debug!(
        "AI setup: enabled={}, configured={}",
        config.ai.enabled,
        app.ai.configured
    );
    if config.ai.enabled && !app.ai.configured {
        app.notification
            .show_warning("AI enabled but not configured. Add provider credentials to config.");
    }

    // Worker needed even when disabled to support Ctrl+A toggle
    if !app.ai.configured {
        log::debug!("AI: skipping worker spawn (not configured)");
        return;
    }

    let (request_tx, request_rx) = std::sync::mpsc::channel();
    let (response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.set_channels(request_tx, response_rx);

    // Spawn the worker thread
    ai::worker::spawn_worker(&config.ai, request_rx, response_tx);
    log::debug!("AI worker spawned");
}

/// Initialize debug logger when activated via --debug flag, JIQ_DEBUG=1, or debug build.
/// All output goes to /tmp/jiq-debug.log (never stdout/stderr).
fn init_logger(cli_debug: bool) {
    let env_debug = std::env::var("JIQ_DEBUG").is_ok_and(|v| v == "1");
    let debug_build = cfg!(debug_assertions);
    let enabled = cli_debug || env_debug || debug_build;

    if !enabled {
        return;
    }

    use std::io::Write;

    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/jiq-debug.log")
    {
        Ok(f) => f,
        Err(_) => return,
    };

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

    let method = match (cli_debug, env_debug, debug_build) {
        (true, true, _) => "--debug flag and JIQ_DEBUG env var",
        (true, false, _) => "--debug flag",
        (false, true, _) => "JIQ_DEBUG env var",
        (false, false, true) => "debug build",
        _ => "unknown",
    };

    log::debug!(
        "=== JIQ DEBUG SESSION STARTED (v{}) ===",
        env!("CARGO_PKG_VERSION")
    );
    log::debug!("Activated via: {}", method);
    log::debug!(
        "Platform: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    // Enable perf instrumentation alongside the debug log. Stopwatches
    // accumulate to an in-memory tally; the percentile summary is dumped
    // to the same log on exit.
    perf::enable();
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
