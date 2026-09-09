//! Conversation bookkeeping for the AI popup.
//!
//! Tracks the question behind the current request, archives completed
//! exchanges into `history`, and owns the chat input buffer.

use ratatui::style::Style;
use tui_textarea::TextArea;

use crate::ai::ai_state::AiState;
use crate::ai::chat::{AiPrompt, ChatExchange};
use crate::ai::prompt::MAX_HISTORY_EXCHANGES;
use crate::theme;

/// Placeholder shown in the empty chat input.
pub const CHAT_PLACEHOLDER: &str = "Ask about this query or data…";

/// A single-line text buffer for the chat input.
pub fn new_chat_input() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
    textarea.set_placeholder_text(CHAT_PLACEHOLDER);
    textarea.set_placeholder_style(Style::default().fg(theme::ai::chat_placeholder()));
    textarea
}

impl AiState {
    /// Send a chat question. The prompt has already been assembled by the
    /// caller; this records the question so the response is displayed and
    /// later archived as an exchange.
    pub fn send_chat_request(&mut self, prompt: AiPrompt, question: String, query: String) -> bool {
        if !self.send_request(prompt) {
            return false;
        }
        self.current_question = Some(question);
        self.current_query = query;
        true
    }

    /// Move a completed chat exchange from the active slot into `history`.
    ///
    /// Only exchanges with a parsed answer or suggestions are kept; failed or
    /// cancelled requests are dropped so the replayed conversation never
    /// contains an assistant turn the model did not actually produce.
    pub fn archive_current_exchange(&mut self) {
        let Some(question) = self.current_question.take() else {
            return;
        };
        let query = std::mem::take(&mut self.current_query);
        let completed = !self.loading && !self.response.is_empty();
        let has_content = self.answer.is_some() || !self.suggestions.is_empty();
        if !completed || !has_content {
            return;
        }
        self.history.push(ChatExchange {
            question,
            query,
            raw_response: self.response.clone(),
            answer: self.answer.clone(),
            suggestions: self.suggestions.clone(),
        });
        if self.history.len() > MAX_HISTORY_EXCHANGES {
            let excess = self.history.len() - MAX_HISTORY_EXCHANGES;
            self.history.drain(..excess);
        }
    }

    /// Forget the whole conversation, including a chat answer on screen.
    /// Suggestions produced by a query change are left in place.
    pub fn clear_conversation(&mut self) {
        self.history.clear();
        if self.current_question.take().is_some() {
            self.current_query.clear();
            self.answer = None;
            if !self.loading {
                self.response.clear();
                self.suggestions.clear();
                self.parse_failed = false;
                self.no_suggestions = false;
                self.selection.clear_selection();
                self.selection.clear_layout();
            }
        }
    }

    /// The text currently typed in the chat input.
    pub fn chat_input_text(&self) -> &str {
        self.chat_input
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Take the typed question, leaving the input empty. Returns `None` when
    /// the input is blank.
    pub fn take_chat_input(&mut self) -> Option<String> {
        let text = self.chat_input_text().trim().to_string();
        self.chat_input.select_all();
        self.chat_input.cut();
        if text.is_empty() { None } else { Some(text) }
    }

    /// Show or hide the chat input's cursor depending on focus.
    pub fn set_chat_focused(&mut self, focused: bool) {
        let style = if focused {
            theme::palette::cursor()
        } else {
            Style::default()
        };
        self.chat_input.set_cursor_style(style);
    }
}

#[cfg(test)]
#[path = "conversation_tests.rs"]
mod conversation_tests;
