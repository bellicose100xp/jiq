// Deterministic input replay for performance benchmarking.
//
// A bench script is a small text file describing a sequence of key events
// and waits. When jiq is launched with `--bench-script <path>`, events from
// the script are injected into the same handler that crossterm's Event::Key
// flows into, so timing and code paths exactly mirror real keyboard input.
//
// Script format (one directive per line, '#' starts a comment):
//
//   text <chars>     Type each char as a Char key event.
//   key <name>       Press a special key. Names: enter, esc, tab, backspace,
//                    space, up, down, left, right, home, end, pageup,
//                    pagedown, delete, f1..f12. Modifier prefix accepted:
//                    "ctrl-x", "alt-x", "shift-x".
//   wait <ms>        Sleep <ms> milliseconds before the next directive.
//
// The script ends with implicit exit. When the last directive completes,
// jiq triggers a clean shutdown so the perf summary dumps.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Step {
    Key(KeyEvent),
    Wait(Duration),
}

pub struct BenchScript {
    steps: Vec<Step>,
    cursor: usize,
    /// The earliest moment the next step is allowed to fire. Updated when a
    /// `wait` step is consumed.
    next_ready_at: Instant,
}

impl BenchScript {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("cannot read bench script {}: {e}", path.display()))?;
        let steps = parse(&content)?;
        Ok(Self {
            steps,
            cursor: 0,
            next_ready_at: Instant::now(),
        })
    }

    /// Returns the next ready step if any, advancing the cursor. Honors
    /// `wait` steps by deferring readiness — callers that get None when
    /// `is_finished()` is also false should poll again shortly.
    pub fn next_step(&mut self) -> Option<Step> {
        if self.cursor >= self.steps.len() {
            return None;
        }
        if Instant::now() < self.next_ready_at {
            return None;
        }
        let step = self.steps.get(self.cursor)?;
        self.cursor += 1;
        match step {
            Step::Wait(d) => {
                self.next_ready_at = Instant::now() + *d;
                // Recurse to return the directive that follows the wait,
                // unless the wait isn't yet satisfied.
                self.next_step()
            }
            Step::Key(k) => Some(Step::Key(*k)),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.cursor >= self.steps.len() && Instant::now() >= self.next_ready_at
    }
}

fn parse(input: &str) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for (lineno, raw) in input.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let (cmd, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match cmd {
            "text" => {
                if rest.is_empty() {
                    return Err(format!("line {}: 'text' requires an argument", lineno + 1));
                }
                for ch in rest.chars() {
                    steps.push(Step::Key(plain_key(KeyCode::Char(ch))));
                }
            }
            "key" => {
                let key = parse_key(rest)
                    .ok_or_else(|| format!("line {}: unknown key '{}'", lineno + 1, rest))?;
                steps.push(Step::Key(key));
            }
            "wait" => {
                let ms: u64 = rest.parse().map_err(|_| {
                    format!("line {}: 'wait' expects ms (got '{}')", lineno + 1, rest)
                })?;
                steps.push(Step::Wait(Duration::from_millis(ms)));
            }
            other => {
                return Err(format!(
                    "line {}: unknown directive '{}'",
                    lineno + 1,
                    other
                ));
            }
        }
    }
    Ok(steps)
}

fn parse_key(spec: &str) -> Option<KeyEvent> {
    let mut modifiers = KeyModifiers::NONE;
    let mut name = spec;
    loop {
        if let Some(rest) = name.strip_prefix("ctrl-") {
            modifiers |= KeyModifiers::CONTROL;
            name = rest;
        } else if let Some(rest) = name.strip_prefix("alt-") {
            modifiers |= KeyModifiers::ALT;
            name = rest;
        } else if let Some(rest) = name.strip_prefix("shift-") {
            modifiers |= KeyModifiers::SHIFT;
            name = rest;
        } else {
            break;
        }
    }
    let code = match name.to_lowercase().as_str() {
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "space" => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "delete" | "del" => KeyCode::Delete,
        s if s.len() == 1 => KeyCode::Char(s.chars().next()?),
        s if s.starts_with('f') => {
            let n: u8 = s[1..].parse().ok()?;
            if (1..=12).contains(&n) {
                KeyCode::F(n)
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn plain_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}
