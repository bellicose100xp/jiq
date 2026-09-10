//! Conversation types shared by the AI state, the prompt builder, and the
//! provider clients.
//!
//! A request to the model is an [`AiPrompt`]: one system prompt plus an
//! ordered list of user/assistant turns ending with the user turn to answer.
//! Completed question/answer pairs typed into the AI popup are kept as
//! [`ChatExchange`]s and replayed on later requests so follow-up questions
//! keep their context.

use super::suggestion::Suggestion;

/// Who authored a conversation turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// One turn of the conversation sent to the provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// A fully assembled model request.
///
/// `messages` always ends with a user turn and alternates roles, which is
/// what every supported provider requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiPrompt {
    pub system: String,
    pub messages: Vec<ChatMessage>,
}

impl AiPrompt {
    /// A prompt with no prior conversation: just the system text and one user turn.
    #[cfg(test)]
    pub fn single(system: impl Into<String>, user: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            messages: vec![ChatMessage::user(user)],
        }
    }

    /// Combined character length of the system prompt and every turn (for logging).
    pub fn total_len(&self) -> usize {
        self.system.len() + self.messages.iter().map(|m| m.content.len()).sum::<usize>()
    }
}

/// One completed question/answer pair from the AI popup's chat input.
///
/// `raw_response` is replayed verbatim as the assistant turn on later
/// requests so the model sees its own earlier output in the format it
/// produced it. `answer` and `suggestions` are the parsed form used for
/// display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatExchange {
    /// The question exactly as the user typed it.
    pub question: String,
    /// The jq query in the input box at the time the question was asked.
    pub query: String,
    /// The provider's complete response text.
    pub raw_response: String,
    /// Prose part of the parsed response, if the model gave one.
    pub answer: Option<String>,
    /// Applyable queries from the parsed response.
    pub suggestions: Vec<Suggestion>,
}

impl ChatExchange {
    /// The user turn replayed to the model for this exchange.
    ///
    /// The query is included so the model knows what the question referred
    /// to, while the bulky JSON context from the original request is left
    /// out to keep the history small.
    pub fn user_turn(&self) -> String {
        if self.query.trim().is_empty() {
            self.question.clone()
        } else {
            format!("Query at the time: `{}`\n\n{}", self.query, self.question)
        }
    }
}

#[cfg(test)]
#[path = "chat_tests.rs"]
mod chat_tests;
