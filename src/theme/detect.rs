use super::ResolvedTheme;
use terminal_colorsaurus::{QueryOptions, ThemeMode as TcThemeMode, theme_mode};

/// Query the terminal (OSC 11) for its background polarity.
///
/// MUST run once at startup BEFORE crossterm raw mode / alt screen.
/// terminal_colorsaurus opens the controlling tty itself (probes
/// stderr/stdout/stdin//dev/tty), so it works even though jiq's stdin
/// is a piped JSON stream. Any failure falls back to Dark.
pub fn detect_background() -> ResolvedTheme {
    // Use the library default timeout (1s). A shorter timeout risks the
    // terminal's OSC reply arriving *after* we give up, leaking the
    // raw "10;rgb:..." bytes into the input box on slower links (SSH,
    // Cloud Desktop, tmux passthrough). On success colorsaurus drains
    // its full reply, and its DA1 feature-detection returns early for
    // terminals that don't support the query, so startup stays snappy.
    let opts = QueryOptions::default();
    match theme_mode(opts) {
        Ok(TcThemeMode::Light) => ResolvedTheme::Light,
        Ok(TcThemeMode::Dark) => ResolvedTheme::Dark,
        Err(e) => {
            log::debug!("terminal background detection failed: {e}; defaulting to dark");
            ResolvedTheme::Dark
        }
    }
}

#[cfg(test)]
#[path = "detect_tests.rs"]
mod detect_tests;
