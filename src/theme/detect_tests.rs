use super::*;

#[test]
fn detect_background_returns_a_valid_variant() {
    let resolved = detect_background();
    assert!(matches!(
        resolved,
        ResolvedTheme::Light | ResolvedTheme::Dark
    ));
}
