use super::*;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_snippet_popup_to_string(results_area: Rect, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal.draw(|f| render_popup(f, results_area)).unwrap();
    terminal.backend().to_string()
}

#[test]
fn snapshot_empty_snippet_popup() {
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 20,
    };
    let output = render_snippet_popup_to_string(results_area, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_narrow_terminal() {
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 15,
    };
    let output = render_snippet_popup_to_string(results_area, 40, 20);
    assert_snapshot!(output);
}

#[test]
fn snapshot_snippet_popup_small_height() {
    let results_area = Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 6,
    };
    let output = render_snippet_popup_to_string(results_area, 80, 10);
    assert_snapshot!(output);
}
