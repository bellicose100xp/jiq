use super::*;

/// Extracts the `(r, g, b)` channels from a `Color::Rgb`, asserting the field
/// is a concrete RGB triple. A non-Rgb variant here would itself break the
/// documented "tuned against pure-white" promise, so the panic is meaningful.
fn rgb(color: Color) -> (f64, f64, f64) {
    match color {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        other => panic!("expected Color::Rgb for a WCAG-checked field, got {other:?}"),
    }
}

/// WCAG relative-luminance channel linearization.
fn linearize(channel: f64) -> f64 {
    let c = channel / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an sRGB color.
fn luminance(color: Color) -> f64 {
    let (r, g, b) = rgb(color);
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// WCAG contrast ratio between two colors (order-independent).
fn contrast_ratio(fg: Color, bg: Color) -> f64 {
    let lf = luminance(fg);
    let lb = luminance(bg);
    let (hi, lo) = if lf >= lb { (lf, lb) } else { (lb, lf) };
    (hi + 0.05) / (lo + 0.05)
}

/// The module/function docs promise galaxy_light is WCAG-AA tuned against its
/// pure-white background. This pins that documented invariant: every primary
/// text-on-white pair clears AA normal-text contrast (>= 4.5:1), and the
/// transient UI-status accent clears AA large/UI contrast (>= 3.0:1). A future
/// palette retune that silently drops a foreground below threshold fails here.
#[test]
fn galaxy_light_meets_wcag_aa_text_contrast() {
    let t = galaxy_light();

    let white = Color::Rgb(255, 255, 255);
    assert_eq!(
        t.palette.bg_dark, white,
        "galaxy_light background must be pure white per the documented invariant"
    );
    assert_eq!(
        t.results.background, white,
        "galaxy_light results background must be pure white per the documented invariant"
    );

    // Primary text and full-strength accents rendered on the white background
    // must meet AA normal-text contrast (>= 4.5:1).
    let normal_text_pairs = [
        ("palette.text", t.palette.text),
        ("palette.text_dim", t.palette.text_dim),
        ("palette.text_muted", t.palette.text_muted),
        ("palette.error", t.palette.error),
        ("palette.success", t.palette.success),
        ("palette.warning", t.palette.warning),
        ("palette.info", t.palette.info),
        ("results.error_summary", t.results.error_summary),
        ("results.error_hint_label", t.results.error_hint_label),
        ("results.error_hint_text", t.results.error_hint_text),
        ("results.error_location", t.results.error_location),
        ("results.result_ok", t.results.result_ok),
        ("results.result_warning", t.results.result_warning),
        ("results.result_error", t.results.result_error),
        ("results.timing_normal", t.results.timing_normal),
        ("results.timing_slow", t.results.timing_slow),
        ("results.timing_very_slow", t.results.timing_very_slow),
    ];
    for (name, fg) in normal_text_pairs {
        let ratio = contrast_ratio(fg, white);
        assert!(
            ratio >= 4.5,
            "{name} contrast against white is {ratio:.3}, below WCAG-AA normal-text 4.5:1"
        );
    }

    // result_pending is a de-emphasized transient status accent; it is allowed
    // to land in the AA large/UI band but must still clear 3.0:1.
    let pending_ratio = contrast_ratio(t.results.result_pending, white);
    assert!(
        pending_ratio >= 3.0,
        "results.result_pending contrast against white is {pending_ratio:.3}, below WCAG-AA large/UI 3.0:1"
    );
}
