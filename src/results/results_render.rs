use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

use crate::app::App;
use crate::scroll::ScrollState;
use crate::search::Match;
use crate::search::search_render::SEARCH_BAR_HEIGHT;
use crate::theme;
use crate::widgets::scrollbar;

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Below this column budget, the path-at-cursor span hides entirely rather
/// than rendering a lonely `…` next to the existing stats prefix.
const PATH_AT_CURSOR_MIN_WIDTH: usize = 5;

/// Cells consumed by the ` · ` separator and the trailing space inside the
/// path span (`format!("{} ", path)`). Subtracted from the available top
/// border before head-truncating the path text itself.
const PATH_AT_CURSOR_CHROME_WIDTH: usize = 4;

/// Visible text of the clickable Back badge on the top border. The leading
/// `<` doubles as the keyboard chord, so the visual reads as both shortcut
/// and button.
const BACK_BADGE_TEXT: &str = "[ < Back ]";

/// Width consumed by the back-badge slot in the title: a leading space,
/// the badge body, and the natural separation provided by following
/// spans. Subtracted from the path-at-cursor budget so the truncation
/// computation accounts for the new chrome.
const BACK_BADGE_RENDERED_WIDTH: usize = 11;

/// Build the path-navigation chord segment shared by every bottom border
/// that advertises `>` / `<` / `*` / `^` / `}` / `]` / `[`. The `<` slot
/// reinforces the clickable Back badge by teaching the keyboard chord;
/// it only appears when the undo ring is non-empty so the hint never
/// misleads.
fn path_chord_hints(can_undo: bool) -> Vec<(&'static str, &'static str)> {
    let mut hints: Vec<(&'static str, &'static str)> = vec![(">", "value")];
    if can_undo {
        hints.push(("<", "back"));
    }
    hints.extend([
        ("*", "iterate"),
        ("^", "parent"),
        ("}", "wrap"),
        ("]/[", "siblings"),
    ]);
    hints
}

fn build_results_pane_hints(can_undo: bool) -> Line<'static> {
    let mut hints: Vec<(&'static str, &'static str)> =
        vec![("Ctrl+T", "Edit Query"), ("i", "Edit Query")];
    hints.extend(path_chord_hints(can_undo));
    theme::border_hints::build_hints(&hints, theme::results::hint_key())
}

fn build_search_hints(can_undo: bool) -> Line<'static> {
    let mut hints: Vec<(&'static str, &'static str)> =
        vec![("n/N", "Next/Prev"), ("Enter", "Next")];
    hints.extend(path_chord_hints(can_undo));
    hints.extend([("Ctrl+F", "Edit"), ("Esc", "Close")]);
    theme::border_hints::build_hints(&hints, theme::results::search_active())
}

/// Cells of clear space the centered hint strip leaves on each side so the
/// left-aligned timing badge and right-aligned position indicator stay
/// readable. Without this gap a centered title that runs to the edge will
/// be visually contiguous with the side titles.
const BOTTOM_CHROME_PADDING_PER_SIDE: u16 = 2;

/// Compute the maximum width the centered bottom hint strip may occupy
/// without colliding with the left timing badge or the right position /
/// match-count title.
///
/// ratatui's `Block` centers its title using half-widths around the row's
/// midpoint, so the constraint that the right edge of the center title
/// stays clear of the right title is:
///   `total_width / 2 + right_title_width + padding <= row_width / 2`.
/// Equivalently the center width must fit within
/// `row_width - 2 * (max(left_width, right_width) + padding)`. We use the
/// max of the two side widths (rather than their sum) because the center
/// is symmetric about the midpoint — the wider side is what binds.
fn bottom_center_budget(
    area_width: u16,
    left_title: Option<&Line<'_>>,
    right_title: Option<&Line<'_>>,
) -> u16 {
    use unicode_width::UnicodeWidthStr;
    let title_width = |title: Option<&Line<'_>>| -> u16 {
        title
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| UnicodeWidthStr::width(s.content.as_ref()) as u16)
                    .sum()
            })
            .unwrap_or(0)
    };
    let left_w = title_width(left_title);
    let right_w = title_width(right_title);
    let bind = left_w.max(right_w) + BOTTOM_CHROME_PADDING_PER_SIDE;
    // 2 cells for the rounded corners; the centered title is symmetric
    // around the row midpoint, so reserve 2 * bind off the row.
    area_width.saturating_sub(2).saturating_sub(2 * bind)
}

/// Trim trailing hints from a centered hint `Line` until its rendered
/// width fits inside `max_width`. The builder produces a leading `" "`
/// span followed by triples of `(separator, key, description)`, so we
/// pop spans three at a time so the remaining strip always ends on a
/// fully-rendered hint rather than a stray separator or key. Ratatui's
/// title renderer overlays alignment slots on the same row, and a
/// centered title that runs to the edge clobbers the right title; this
/// guarantees the centered line never reaches that edge.
fn truncate_hints_to_width(line: Line<'static>, max_width: u16) -> Line<'static> {
    use unicode_width::UnicodeWidthStr;
    let mut spans = line.spans;
    let line_width = |spans: &[Span<'_>]| -> u16 {
        spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()) as u16)
            .sum()
    };
    while line_width(&spans) > max_width && spans.len() >= 4 {
        // Pop one full hint triple (separator + key + description) per
        // iteration. Stop at <= 3 spans, which leaves the leading raw
        // space + first hint pair (key + desc) in place for readability.
        for _ in 0..3 {
            spans.pop();
        }
    }
    // If it still doesn't fit (terminal pathologically narrow), fall back
    // to per-span truncation so we never return a wider line than asked.
    while line_width(&spans) > max_width && !spans.is_empty() {
        spans.pop();
    }
    Line::from(spans)
}

/// Build the styled `[ < Back ]` badge spans, plus the screen rect they
/// occupy on the results-pane top border. The caller is responsible for
/// inserting the spans into the title at the right slot (after the
/// optional spinner) and storing the rect in [`LayoutRegions::back_button`]
/// so mouse hit-testing can route clicks to `drill_back`.
///
/// `start_col_offset` is the number of cells consumed by spans rendered
/// before the badge in the title (the rounded corner is always 1 cell, and
/// pending state adds a 2-cell `<spinner> ` prefix). The returned rect
/// covers the full badge text including the surrounding whitespace, giving
/// the user a generous click target.
fn build_back_badge(
    area: Rect,
    start_col_offset: u16,
    hovered: bool,
) -> (Vec<Span<'static>>, Rect) {
    let style = if hovered {
        theme::results::badge_back_hover()
    } else {
        theme::results::badge_back()
    };
    let spans = vec![Span::raw(" "), Span::styled(BACK_BADGE_TEXT, style)];
    // Block draws the rounded corner at `area.x`, the title row at `area.y`.
    // The leading raw space sits at `area.x + 1`, so the badge body starts
    // at `area.x + 2 + start_col_offset`.
    let rect = Rect {
        x: area.x.saturating_add(2).saturating_add(start_col_offset),
        y: area.y,
        width: BACK_BADGE_TEXT.len() as u16,
        height: 1,
    };
    (spans, rect)
}

fn get_spinner(frame_count: u64) -> (char, Color) {
    let index = (frame_count / 8) as usize;
    let colors = theme::results::spinner_colors();
    (
        SPINNER_CHARS[index % SPINNER_CHARS.len()],
        colors[index % colors.len()],
    )
}

fn format_position_indicator(scroll: &ScrollState, line_count: u32) -> String {
    if line_count == 0 {
        return String::new();
    }
    let start = scroll.offset as u32 + 1;
    let end = (scroll.offset as u32 + scroll.viewport_height as u32).min(line_count);
    let percentage = (scroll.offset as u32 * 100) / line_count;
    format!("L{}-{}/{} ({}%)", start, end, line_count, percentage)
}

/// Compute the column budget for the path-at-cursor span on the success
/// branch's top border, given the rendered widths of the surrounding chrome.
/// Caller still gates on [`PATH_AT_CURSOR_MIN_WIDTH`] so a degenerate budget
/// hides the span entirely instead of showing a lonely `…`.
///
/// `top_right_width` is the rendered width of the right-aligned position /
/// match-count indicator that now shares the top border. ratatui overlays
/// the right title over the same row as the left content, so the breadcrumb
/// must yield first: we subtract the indicator's width (plus a per-side gap
/// mirroring [`BOTTOM_CHROME_PADDING_PER_SIDE`]) so the path truncates before
/// it can collide with the indicator on narrow widths.
fn path_at_cursor_budget(
    area_width: u16,
    stats_info: &str,
    is_pending: bool,
    has_back_badge: bool,
    top_right_width: usize,
) -> usize {
    use unicode_width::UnicodeWidthStr;

    // Block borders consume the leftmost and rightmost cells of the title row.
    const BORDER_CORNERS: usize = 2;
    // The stats prefix is rendered as ` {stats} ` (one cell of padding on
    // each side). Pending state prepends `<spinner> ` on the very left.
    const STATS_PADDING: usize = 2;
    const SPINNER_WIDTH: usize = 2;

    let stats_width = UnicodeWidthStr::width(stats_info) + STATS_PADDING;
    let spinner_width = if is_pending { SPINNER_WIDTH } else { 0 };
    let back_width = if has_back_badge {
        BACK_BADGE_RENDERED_WIDTH
    } else {
        0
    };
    // Reserve the right-aligned indicator plus a one-cell gap on each side so
    // the breadcrumb never runs flush into it. Zero when no indicator shows.
    let right_reservation = if top_right_width > 0 {
        top_right_width + 2 * BOTTOM_CHROME_PADDING_PER_SIDE as usize
    } else {
        0
    };

    (area_width as usize)
        .saturating_sub(BORDER_CORNERS)
        .saturating_sub(stats_width)
        .saturating_sub(spinner_width)
        .saturating_sub(back_width)
        .saturating_sub(right_reservation)
        .saturating_sub(PATH_AT_CURSOR_CHROME_WIDTH)
}

fn format_execution_time(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

fn get_timing_color(ms: u64, border_color: Color) -> Color {
    if ms < 200 {
        border_color
    } else if ms < 1000 {
        theme::results::timing_slow()
    } else {
        theme::results::timing_very_slow()
    }
}

fn render_scrollbar(frame: &mut Frame, area: Rect, scroll: &ScrollState, line_count: u32) {
    let scrollbar_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    scrollbar::render_vertical_scrollbar(
        frame,
        scrollbar_area,
        line_count as usize,
        scroll.viewport_height as usize,
        scroll.offset as usize,
    );
}

/// Render the results pane
///
/// Returns the (results_area, search_bar_area) tuple for region tracking.
pub fn render_pane(app: &mut App, frame: &mut Frame, area: Rect) -> (Rect, Option<Rect>) {
    let (results_area, search_area) = if app.search.is_visible() {
        let layout = Layout::vertical([Constraint::Min(3), Constraint::Length(SEARCH_BAR_HEIGHT)])
            .split(area);
        (layout[0], Some(layout[1]))
    } else {
        (area, None)
    };

    // Check if query is available
    let query_state = match &app.query {
        Some(q) => q,
        None => {
            // Show loading indicator or error if file loader is present
            if let Some(loader) = &app.file_loader {
                if loader.is_loading() {
                    render_loading_indicator(frame, results_area);
                } else if let crate::input::loader::LoadingState::Error(e) = loader.state() {
                    render_error_message(
                        frame,
                        results_area,
                        &format!("Failed to load file: {}", e),
                    );
                }
            }
            return (results_area, search_area);
        }
    };

    let is_pending = query_state.is_pending();
    let stats_info = app.stats.display().unwrap_or_else(|| "Results".to_string());

    // Path-at-cursor: only resolved on success branch. Always falls back to
    // the cursor row, but prefers the current search match's row when
    // search is visible and has matches — that way the path tracks what the
    // user is actually looking at while typing or navigating a search.
    // Computed once here so the borrow on query_state below stays clean.
    let path_at_cursor_jq: Option<String> = if app.focus == crate::app::Focus::ResultsPane
        && !query_state.is_synthetic_merge
        && query_state.result.is_ok()
        && !query_state.is_empty_result
    {
        let path_row = if app.search.is_visible() {
            app.search.current_match().map(|m| m.line)
        } else {
            None
        };
        match path_row {
            Some(row) => app.path_at_row(row).map(|p| p.to_jq()),
            None if !app.search.is_visible() => app.current_cursor_path().map(|p| p.to_jq()),
            None => None,
        }
    } else {
        None
    };

    // Re-borrow query_state after the &mut self call above released it.
    let query_state = match &app.query {
        Some(q) => q,
        None => return (results_area, search_area),
    };

    // Calculate viewport dimensions and position indicator early for title
    let viewport_height = results_area.height.saturating_sub(2);
    let viewport_width = results_area.width.saturating_sub(2);
    let line_count = app.results_line_count_u32();
    app.results_scroll
        .update_bounds(line_count, viewport_height);
    if let Some(q) = &app.query {
        app.results_scroll
            .update_h_bounds(q.max_line_width(), viewport_width);
    }

    app.results_cursor.update_total_lines(line_count);

    if let Some(q) = &app.query
        && let Some(widths) = &q.cached_line_widths
    {
        app.results_cursor
            .update_line_widths(std::sync::Arc::clone(widths));
    }

    let position_indicator = format_position_indicator(&app.results_scroll, line_count);

    let search_visible = app.search.is_visible();

    // Rendered width of the right-aligned top-border indicator (match-count
    // badge when search is confirmed, otherwise the position indicator). The
    // breadcrumb budget below subtracts this so the two never collide on the
    // shared top row. Match-count is wrapped as ` {badge} ` with two extra
    // padding cells inside the badge (`  {n}  `), totalling 6 chrome cells;
    // the position indicator is wrapped as ` {text} ` (2 chrome cells).
    let top_right_indicator_width: usize = {
        use unicode_width::UnicodeWidthStr;
        if search_visible && app.search.is_confirmed() {
            UnicodeWidthStr::width(app.search.match_count_display().as_str()) + 6
        } else if !position_indicator.is_empty() {
            UnicodeWidthStr::width(position_indicator.as_str()) + 2
        } else {
            0
        }
    };

    // When search is confirmed (navigating results), results pane is active (purple)
    // When search is not confirmed (editing search), results pane is inactive (gray)
    let search_text_color = if search_visible && app.search.is_confirmed() {
        theme::results::search_active()
    } else if search_visible {
        theme::results::search_inactive()
    } else {
        theme::palette::text()
    };

    // The clickable Back badge appears on the top border whenever there is
    // something to undo. Build its spans and rect up-front so the rect can
    // be stored in layout_regions for mouse hit-testing regardless of which
    // title branch we render below. The badge sits AFTER the optional
    // spinner so it stays in a stable location relative to the corner.
    let has_back_badge = !app.query_undo.is_empty();
    let back_start_col_offset: u16 = if is_pending { 2 } else { 0 };
    let (back_spans, back_rect): (Vec<Span<'static>>, Option<Rect>) = if has_back_badge {
        let (spans, rect) =
            build_back_badge(results_area, back_start_col_offset, app.back_button_hovered);
        (spans, Some(rect))
    } else {
        (Vec::new(), None)
    };
    app.layout_regions.back_button = back_rect;

    let (title, unfocused_border_color) = if query_state.result.is_err() {
        // ERROR: Yellow text, yellow border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            theme::results::result_warning()
        };
        let mut spans = Vec::new();
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            spans.push(Span::styled(
                format!("{} ", spinner_char),
                Style::default().fg(spinner_color),
            ));
        }
        // The back-badge spans already start with a leading space. Always
        // push a trailing neutral space so the cyan badge never renders flush
        // against the colored status badge to its right; without the badge the
        // same space serves as the separator from the rounded corner.
        if has_back_badge {
            spans.extend(back_spans.clone());
        }
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "  ⚠ Syntax Error  ",
            theme::results::badge_syntax_error(),
        ));
        if !stats_info.is_empty() {
            spans.push(Span::styled(
                format!(" {} | Showing last successful result ", stats_info),
                Style::default().fg(text_color),
            ));
        }
        (Line::from(spans), theme::results::border_warning())
    } else if query_state.is_empty_result {
        // EMPTY: Gray text, gray border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            theme::results::result_pending()
        };
        let mut spans = Vec::new();
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            spans.push(Span::styled(
                format!("{} ", spinner_char),
                Style::default().fg(spinner_color),
            ));
        }
        if has_back_badge {
            spans.extend(back_spans.clone());
        }
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "  ∅ No Results  ",
            theme::results::badge_empty_result(),
        ));
        spans.push(Span::styled(
            format!(" {} | Showing last non-empty result ", stats_info),
            Style::default().fg(text_color),
        ));
        (Line::from(spans), theme::results::border_unfocused())
    } else {
        // SUCCESS: Green text, green border (unfocused) - or search color when search visible
        let text_color = if search_visible {
            search_text_color
        } else {
            theme::results::result_ok()
        };
        let path_budget = path_at_cursor_budget(
            results_area.width,
            &stats_info,
            is_pending,
            has_back_badge,
            top_right_indicator_width,
        );
        let path_spans: Vec<Span<'static>> = match path_at_cursor_jq.as_deref() {
            Some(path) if !path.is_empty() && path_budget >= PATH_AT_CURSOR_MIN_WIDTH => {
                let truncated = crate::str_utils::head_truncate_to_width(path, path_budget);
                vec![
                    Span::styled(
                        " · ",
                        Style::default().fg(theme::results::path_at_cursor_separator()),
                    ),
                    Span::styled(
                        format!("{} ", truncated),
                        Style::default().fg(theme::results::path_at_cursor()),
                    ),
                ]
            }
            _ => Vec::new(),
        };
        if is_pending {
            let (spinner_char, spinner_color) = get_spinner(app.frame_count);
            let mut spans = vec![Span::styled(
                format!("{} ", spinner_char),
                Style::default().fg(spinner_color),
            )];
            // Without the back badge, the spinner already supplies the
            // trailing space before the stats. With the badge inserted
            // between them, the stats span needs its own leading space so
            // the `]` of `[ < Back ]` doesn't collide with the stats text.
            let stats_format = if has_back_badge {
                format!(" {} ", stats_info)
            } else {
                format!("{} ", stats_info)
            };
            spans.extend(back_spans.clone());
            spans.push(Span::styled(stats_format, Style::default().fg(text_color)));
            spans.extend(path_spans);
            (Line::from(spans), theme::results::result_ok())
        } else {
            let mut spans = back_spans.clone();
            spans.push(Span::styled(
                format!(" {} ", stats_info),
                Style::default().fg(text_color),
            ));
            spans.extend(path_spans);
            (Line::from(spans), theme::results::result_ok())
        }
    };

    let position_title_color = if search_visible {
        search_text_color
    } else {
        unfocused_border_color
    };
    // Position indicator lives on the top-right border so it stays visible
    // above the AI / help boxes. When search is confirmed the match-count
    // badge takes that slot, so callers must suppress this title in that case.
    let position_title: Option<Line<'static>> = if !position_indicator.is_empty() {
        Some(Line::from(Span::styled(
            format!(" {} ", position_indicator),
            Style::default().fg(position_title_color),
        )))
    } else {
        None
    };

    // When search is confirmed (navigating), results pane is active (purple)
    // When search is not confirmed (editing), results pane is inactive (gray)
    let border_color = if search_visible {
        if app.search.is_confirmed() {
            theme::results::search_active()
        } else {
            theme::results::search_inactive()
        }
    } else if app.focus == crate::app::Focus::ResultsPane {
        theme::results::border_focused()
    } else {
        unfocused_border_color
    };

    let search_no_match = search_visible
        && !app.search.is_confirmed()
        && !app.search.query().is_empty()
        && app.search.matches().is_empty();
    let is_stale = query_state.result.is_err() || query_state.is_empty_result || search_no_match;

    let title = if search_no_match {
        let mut spans = vec![
            Span::raw(" "),
            Span::styled("  ⚠ No Matches  ", theme::search::badge_no_matches()),
            Span::raw(" "),
        ];
        spans.extend(title.spans);
        Line::from(spans)
    } else {
        title
    };

    // Always render from cached pre-rendered text
    if let Some(rendered) = &query_state.last_successful_result_rendered {
        // Pre-compute the bottom-row pieces so the centered hint strip can be
        // trimmed to the room left over after the timing (left) claims its
        // slot. The right-anchored indicator now lives on the TOP border, so
        // the bottom-right slot is empty and the centered strip reclaims that
        // space (right_title = None). ratatui's Block draws right then center
        // then left over the same row, so a centered title wider than its
        // slot would silently overwrite a right title — explicit trimming
        // here keeps the strip clear of the timing badge.
        let timing_title = query_state.cached_execution_time_ms.map(|ms| {
            let timing_text = format!(" {} ", format_execution_time(ms));
            let timing_color = get_timing_color(ms, border_color);
            Line::from(vec![Span::styled(
                timing_text,
                Style::default().fg(timing_color),
            )])
        });
        let match_count_badge = if search_visible && app.search.is_confirmed() {
            let match_count = app.search.match_count_display();
            Some(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    format!("  {}  ", match_count),
                    theme::search::badge_match_count(),
                ),
                Span::raw(" "),
            ]))
        } else {
            None
        };
        // Right-anchored TOP-border indicator: match-count badge during
        // confirmed search, position indicator otherwise.
        let top_right_title: Option<Line<'static>> =
            match_count_badge.or_else(|| position_title.clone());

        let center_budget = bottom_center_budget(results_area.width, timing_title.as_ref(), None);

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::right(1))
            .title(title)
            .border_style(Style::default().fg(border_color));
        // The right-aligned top title coexists with the left-anchored `.title`
        // above (ratatui supports multiple top titles with distinct
        // alignments); the breadcrumb budget already reserved space for it.
        if let Some(rt) = top_right_title {
            block = block.title_top(rt.alignment(Alignment::Right));
        }
        if search_visible && app.search.is_confirmed() {
            block = block.title_bottom(
                truncate_hints_to_width(
                    build_search_hints(!app.query_undo.is_empty()),
                    center_budget,
                )
                .alignment(Alignment::Center),
            );
        }
        // While editing the search query the drill chords are inert
        // (typed as text instead), so the results-pane bottom strip is
        // intentionally bare in that mode — the search bar carries the
        // applicable Enter / Tab / Esc hints.

        // Add navigation hints when results pane is focused and search is not visible
        if !search_visible && app.focus == crate::app::Focus::ResultsPane {
            block = block.title_bottom(
                truncate_hints_to_width(
                    build_results_pane_hints(!app.query_undo.is_empty()),
                    center_budget,
                )
                .alignment(Alignment::Center),
            );
        }

        // Add execution time display in bottom-left corner
        if let Some(tt) = timing_title.clone() {
            block = block.title_bottom(tt.alignment(Alignment::Left));
        }

        // Use cached pre-rendered text
        // Optimization: Only clone visible viewport to avoid massive allocations
        let scroll_offset = app.results_scroll.offset as usize;
        let viewport_lines = viewport_height as usize;

        // Slice to viewport range (with bounds checking)
        let total_lines = rendered.lines.len();
        let end_line = (scroll_offset + viewport_lines).min(total_lines);
        let visible_lines = if scroll_offset < total_lines {
            &rendered.lines[scroll_offset..end_line]
        } else {
            &[]
        };

        // Clone only visible lines (50 lines instead of 100K+ for large files!)
        let viewport_text = Text::from(visible_lines.to_vec());

        // Apply the theme's stale modifier (DIM on dark; none on light,
        // where DIM bleaches toward the near-white background).
        let viewport_text = if is_stale {
            apply_stale_modifier(viewport_text)
        } else {
            viewport_text
        };

        // Apply search highlights only to visible viewport
        let final_text = if app.search.is_visible() && !app.search.matches().is_empty() {
            apply_search_highlights(
                viewport_text,
                &app.search,
                app.results_scroll.offset,
                viewport_height,
            )
        } else {
            viewport_text
        };

        let show_cursor = app.focus == crate::app::Focus::ResultsPane;
        let final_text = if show_cursor {
            apply_cursor_highlights(final_text, &app.results_cursor, app.results_scroll.offset)
        } else {
            final_text
        };

        // Vertical scroll handled by viewport slicing, but horizontal scroll still needed
        let content = Paragraph::new(final_text)
            .block(block)
            .scroll((0, app.results_scroll.h_offset));

        frame.render_widget(content, results_area);
        render_scrollbar(frame, results_area, &app.results_scroll, line_count);

        if show_cursor {
            render_cursor_indicator(
                frame,
                results_area,
                &app.results_cursor,
                app.results_scroll.offset,
                app.results_scroll.h_offset,
            );
        }
    } else {
        // No successful result yet - show empty
        let timing_title = query_state.cached_execution_time_ms.map(|ms| {
            let timing_text = format!(" {} ", format_execution_time(ms));
            let timing_color = get_timing_color(ms, border_color);
            Line::from(vec![Span::styled(
                timing_text,
                Style::default().fg(timing_color),
            )])
        });
        let match_count_badge = if search_visible && app.search.is_confirmed() {
            let match_count = app.search.match_count_display();
            Some(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    format!("  {}  ", match_count),
                    theme::search::badge_match_count(),
                ),
                Span::raw(" "),
            ]))
        } else {
            None
        };
        // Right-anchored TOP-border indicator (see the cached branch above):
        // match-count badge during confirmed search, position otherwise. With
        // it off the bottom border, the centered hint strip reclaims the
        // bottom-right space (right_title = None).
        let top_right_title: Option<Line<'static>> =
            match_count_badge.or_else(|| position_title.clone());

        let center_budget = bottom_center_budget(results_area.width, timing_title.as_ref(), None);

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::right(1))
            .title(title)
            .border_style(Style::default().fg(border_color));
        if let Some(rt) = top_right_title {
            block = block.title_top(rt.alignment(Alignment::Right));
        }
        if search_visible && app.search.is_confirmed() {
            block = block.title_bottom(
                truncate_hints_to_width(
                    build_search_hints(!app.query_undo.is_empty()),
                    center_budget,
                )
                .alignment(Alignment::Center),
            );
        } else if app.focus == crate::app::Focus::ResultsPane && !search_visible {
            block = block.title_bottom(
                truncate_hints_to_width(
                    build_results_pane_hints(!app.query_undo.is_empty()),
                    center_budget,
                )
                .alignment(Alignment::Center),
            );
        }

        if let Some(tt) = timing_title {
            block = block.title_bottom(tt.alignment(Alignment::Left));
        }

        let empty_text = Text::from("");
        let content = Paragraph::new(empty_text).block(block);

        frame.render_widget(content, results_area);
    }
    if let Some(search_rect) = search_area {
        crate::search::search_render::render_bar(app, frame, search_rect);
    }

    (results_area, search_area)
}

fn render_loading_indicator(frame: &mut Frame, area: Rect) {
    let text = "Loading file...";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Loading ")
        .border_style(Style::default().fg(theme::results::border_warning()));

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(theme::results::border_warning()));

    frame.render_widget(paragraph, area);
}

fn render_error_message(frame: &mut Frame, area: Rect, message: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Error ")
        .border_style(Style::default().fg(theme::results::border_error()));

    let paragraph = Paragraph::new(message)
        .block(block)
        .style(Style::default().fg(theme::results::border_error()));

    frame.render_widget(paragraph, area);
}

fn apply_stale_modifier(text: Text<'_>) -> Text<'static> {
    let stale_modifier = theme::results::stale_modifier();
    Text::from(
        text.lines
            .into_iter()
            .map(|line| {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| {
                            Span::styled(
                                span.content.into_owned(),
                                span.style.add_modifier(stale_modifier),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn apply_search_highlights(
    text: Text<'_>,
    search_state: &crate::search::SearchState,
    scroll_offset: u16,
    viewport_height: u16,
) -> Text<'static> {
    let matches = search_state.matches();
    let current_match_index = search_state.current_index();

    if matches.is_empty() {
        return Text::from(
            text.lines
                .into_iter()
                .map(|line| {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(span.content.into_owned(), span.style))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    let _ = viewport_height;
    let highlighted_lines: Vec<Line<'static>> = text
        .lines
        .into_iter()
        .enumerate()
        .map(|(line_idx, line)| {
            // Adjust line_idx by scroll_offset to get absolute line number
            let absolute_line = line_idx + scroll_offset as usize;
            let line_matches: Vec<(usize, &Match)> =
                search_state.matches_on_line(absolute_line as u32).collect();

            if line_matches.is_empty() {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| Span::styled(span.content.into_owned(), span.style))
                        .collect::<Vec<_>>(),
                )
            } else {
                apply_highlights_to_line(line, &line_matches, current_match_index)
            }
        })
        .collect();

    Text::from(highlighted_lines)
}
fn apply_highlights_to_line(
    line: Line<'_>,
    matches: &[(usize, &Match)],
    current_match_index: usize,
) -> Line<'static> {
    let mut char_styles: Vec<(char, Style)> = Vec::new();

    for span in &line.spans {
        for ch in span.content.chars() {
            char_styles.push((ch, span.style));
        }
    }

    for (match_idx, m) in matches {
        let col_start = m.col as usize;
        let col_end = col_start + m.len as usize;

        let highlight_style = if *match_idx == current_match_index {
            Style::default()
                .fg(theme::results::current_match_fg())
                .bg(theme::results::current_match_bg())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme::results::match_highlight_fg())
                .bg(theme::results::match_highlight_bg())
        };

        for i in col_start..col_end.min(char_styles.len()) {
            char_styles[i].1 = highlight_style;
        }
    }

    let visible_chars: Vec<(char, Style)> = char_styles;
    let mut result_spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style: Option<Style> = None;

    for (ch, style) in visible_chars {
        match current_style {
            Some(s) if s == style => {
                current_text.push(ch);
            }
            _ => {
                if !current_text.is_empty()
                    && let Some(s) = current_style
                {
                    result_spans.push(Span::styled(current_text.clone(), s));
                }
                current_text = ch.to_string();
                current_style = Some(style);
            }
        }
    }
    if !current_text.is_empty()
        && let Some(s) = current_style
    {
        result_spans.push(Span::styled(current_text, s));
    }

    Line::from(result_spans)
}

fn apply_cursor_highlights(
    text: Text<'_>,
    cursor_state: &crate::results::cursor_state::CursorState,
    scroll_offset: u16,
) -> Text<'static> {
    let cursor_line = cursor_state.cursor_line();
    let hovered_line = cursor_state.hovered_line();
    let is_visual = cursor_state.is_visual_mode();
    let (sel_start, sel_end) = cursor_state.selection_range();

    Text::from(
        text.lines
            .into_iter()
            .enumerate()
            .map(|(line_idx, line)| {
                let absolute_line = line_idx as u32 + scroll_offset as u32;

                let bg_color =
                    if is_visual && absolute_line >= sel_start && absolute_line <= sel_end {
                        Some(theme::results::visual_selection_bg())
                    } else if absolute_line == cursor_line {
                        Some(theme::results::cursor_line_bg())
                    } else if Some(absolute_line) == hovered_line {
                        Some(theme::results::hovered_line_bg())
                    } else {
                        None
                    };

                if let Some(bg) = bg_color {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| {
                                let existing_bg = span.style.bg;
                                let is_search_highlight = existing_bg
                                    == Some(theme::results::current_match_bg())
                                    || existing_bg == Some(theme::results::match_highlight_bg());

                                if is_search_highlight {
                                    Span::styled(span.content.into_owned(), span.style)
                                } else {
                                    Span::styled(span.content.into_owned(), span.style.bg(bg))
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(span.content.into_owned(), span.style))
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .collect::<Vec<_>>(),
    )
}

fn render_cursor_indicator(
    frame: &mut Frame,
    results_area: Rect,
    cursor_state: &crate::results::cursor_state::CursorState,
    scroll_offset: u16,
    _h_offset: u16,
) {
    let cursor_line = cursor_state.cursor_line();

    if cursor_line < scroll_offset as u32 {
        return;
    }

    let relative_line = cursor_line.saturating_sub(scroll_offset as u32) as u16;
    let viewport_height = results_area.height.saturating_sub(2);

    if relative_line >= viewport_height {
        return;
    }

    let indicator_x = results_area.x;
    let indicator_y = results_area
        .y
        .saturating_add(1)
        .saturating_add(relative_line);

    let indicator = Span::styled(
        "▌",
        Style::default().fg(theme::results::cursor_indicator_fg()),
    );
    frame.render_widget(
        Paragraph::new(Line::from(indicator)),
        Rect {
            x: indicator_x,
            y: indicator_y,
            width: 1,
            height: 1,
        },
    );
}

#[cfg(test)]
#[path = "results_render_tests.rs"]
mod results_render_tests;
