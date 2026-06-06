use super::*;
use theme::ResolvedTheme;

#[test]
fn light_mode_ignores_detection() {
    assert_eq!(
        resolve_theme(config::ThemeMode::Light, || ResolvedTheme::Dark),
        ResolvedTheme::Light
    );
}

#[test]
fn dark_mode_ignores_detection() {
    assert_eq!(
        resolve_theme(config::ThemeMode::Dark, || ResolvedTheme::Light),
        ResolvedTheme::Dark
    );
}

#[test]
fn auto_mode_uses_detected_dark() {
    assert_eq!(
        resolve_theme(config::ThemeMode::Auto, || ResolvedTheme::Dark),
        ResolvedTheme::Dark
    );
}

#[test]
fn auto_mode_uses_detected_light() {
    assert_eq!(
        resolve_theme(config::ThemeMode::Auto, || ResolvedTheme::Light),
        ResolvedTheme::Light
    );
}

#[test]
fn setup_ai_worker_warns_when_enabled_but_unconfigured() {
    use crate::config::Config;
    use crate::test_utils::test_helpers::{TEST_JSON, test_app};

    let mut app = test_app(TEST_JSON);
    // Default config => provider=None => AiState is not configured.
    assert!(
        !app.ai.configured,
        "precondition: app.ai.configured must be false with default config"
    );

    let mut config = Config::default();
    config.ai.enabled = true;

    setup_ai_worker(&mut app, &config);

    // Enabled-but-unconfigured must surface the exact incomplete-config warning,
    // and the function must early-return without spawning a worker (no channels set).
    assert_eq!(
        app.notification.current_message(),
        Some("AI enabled but not configured. Add provider credentials to config.")
    );
    assert!(
        !app.ai.configured,
        "guard must not flip configured; worker spawn is skipped"
    );
}
