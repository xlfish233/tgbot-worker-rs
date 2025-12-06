//! Simplified Message and CallbackQuery wrappers.
//!
//! These types wrap frankenstein types and provide convenient accessor methods.

use frankenstein::types::{
    CallbackQuery as FrankensteinCallbackQuery, MaybeInaccessibleMessage,
    Message as FrankensteinMessage, User,
};

/// Simplified Message wrapper.
///
/// Provides convenient accessor methods for common message properties.
#[derive(Clone, Debug)]
pub struct Message {
    inner: FrankensteinMessage,
}

impl Message {
    /// Create a new Message from a frankenstein Message.
    pub fn new(inner: FrankensteinMessage) -> Self {
        Self { inner }
    }

    /// Get the underlying frankenstein Message.
    pub fn inner(&self) -> &FrankensteinMessage {
        &self.inner
    }

    /// Get the chat ID.
    pub fn chat_id(&self) -> i64 {
        self.inner.chat.id
    }

    /// Get the message ID.
    pub fn message_id(&self) -> i32 {
        self.inner.message_id
    }

    /// Get the message text (if any).
    pub fn text(&self) -> Option<&str> {
        self.inner.text.as_deref()
    }

    /// Get the user who sent the message (if any).
    pub fn from(&self) -> Option<&User> {
        self.inner.from.as_ref().map(|u| u.as_ref())
    }

    /// Get the user ID of the sender (if any).
    pub fn from_id(&self) -> Option<u64> {
        self.inner.from.as_ref().map(|u| u.id)
    }

    /// Get the command from the message (e.g., "/start" -> "start").
    pub fn command(&self) -> Option<&str> {
        self.text().and_then(|t| {
            let first = t.split_whitespace().next()?;
            first.strip_prefix('/')
        })
    }

    /// Get the command arguments (text after the command).
    pub fn command_args(&self) -> Option<&str> {
        self.text().and_then(|t| {
            let mut parts = t.splitn(2, char::is_whitespace);
            parts.next()?; // skip command
            parts.next().map(|s| s.trim())
        })
    }

    /// Check if this message is a specific command.
    pub fn is_command(&self, cmd: &str) -> bool {
        self.command().map(|c| c == cmd).unwrap_or(false)
    }

    /// Get the caption (for media messages).
    pub fn caption(&self) -> Option<&str> {
        self.inner.caption.as_deref()
    }
}

impl From<FrankensteinMessage> for Message {
    fn from(inner: FrankensteinMessage) -> Self {
        Self::new(inner)
    }
}

/// Simplified CallbackQuery wrapper.
#[derive(Clone, Debug)]
pub struct CallbackQuery {
    inner: FrankensteinCallbackQuery,
}

impl CallbackQuery {
    /// Create a new CallbackQuery from a frankenstein CallbackQuery.
    pub fn new(inner: FrankensteinCallbackQuery) -> Self {
        Self { inner }
    }

    /// Get the underlying frankenstein CallbackQuery.
    pub fn inner(&self) -> &FrankensteinCallbackQuery {
        &self.inner
    }

    /// Get the callback query ID (for answering).
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    /// Get the callback data.
    pub fn data(&self) -> Option<&str> {
        self.inner.data.as_deref()
    }

    /// Get the user who triggered the callback.
    pub fn from(&self) -> &User {
        &self.inner.from
    }

    /// Get the user ID who triggered the callback.
    pub fn from_id(&self) -> u64 {
        self.inner.from.id
    }

    /// Get the chat ID from the message (if available).
    pub fn chat_id(&self) -> Option<i64> {
        self.inner.message.as_ref().map(|m| match m {
            MaybeInaccessibleMessage::Message(msg) => msg.chat.id,
            MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.chat.id,
        })
    }

    /// Get the message ID (if available).
    pub fn message_id(&self) -> Option<i32> {
        self.inner.message.as_ref().map(|m| match m {
            MaybeInaccessibleMessage::Message(msg) => msg.message_id,
            MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.message_id,
        })
    }

    /// Get the associated message (if available).
    pub fn message(&self) -> Option<Message> {
        match &self.inner.message {
            Some(MaybeInaccessibleMessage::Message(msg)) => Some(Message::new((**msg).clone())),
            _ => None,
        }
    }
}

impl From<FrankensteinCallbackQuery> for CallbackQuery {
    fn from(inner: FrankensteinCallbackQuery) -> Self {
        Self::new(inner)
    }
}
