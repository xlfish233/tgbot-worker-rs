use frankenstein::{
    AsyncApi, AsyncTelegramApi, MaybeInaccessibleMessage, SendMessageParams, UpdateContent,
};
use worker::{Env, Response};

use super::storage::{Session, SessionStorage};
use crate::AppResult;
use crate::frankenstein::Update;

/// Request context with session support
pub struct Context<T, S: SessionStorage> {
    pub update: Update,
    pub env: Env,
    pub session: Session<T, S>,
}

impl<T, S: SessionStorage> Clone for Context<T, S> {
    fn clone(&self) -> Self {
        Self {
            update: self.update.clone(),
            env: self.env.clone(),
            session: self.session.clone(),
        }
    }
}

impl<T: Default + serde::Serialize + serde::de::DeserializeOwned + Clone, S: SessionStorage>
    Context<T, S>
{
    /// Create a new context
    pub fn new(update: Update, env: Env, session: Session<T, S>) -> Self {
        Self {
            update,
            env,
            session,
        }
    }

    /// Get chat ID from update
    pub fn chat_id(&self) -> Option<i64> {
        match &self.update.content {
            UpdateContent::Message(m) => Some(m.chat.id),
            UpdateContent::EditedMessage(m) => Some(m.chat.id),
            UpdateContent::ChannelPost(m) => Some(m.chat.id),
            UpdateContent::EditedChannelPost(m) => Some(m.chat.id),
            UpdateContent::CallbackQuery(c) => c.message.as_ref().map(|m| match m {
                MaybeInaccessibleMessage::Message(msg) => msg.chat.id,
                MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.chat.id,
            }),
            _ => None,
        }
    }

    /// Get user ID from update (returns u64 as per Telegram API)
    pub fn user_id(&self) -> Option<u64> {
        match &self.update.content {
            UpdateContent::Message(m) => m.from.as_ref().map(|u| u.id),
            UpdateContent::EditedMessage(m) => m.from.as_ref().map(|u| u.id),
            UpdateContent::CallbackQuery(c) => Some(c.from.id),
            UpdateContent::InlineQuery(q) => Some(q.from.id),
            _ => None,
        }
    }

    /// Get message text
    pub fn text(&self) -> Option<&str> {
        match &self.update.content {
            UpdateContent::Message(m) => m.text.as_deref(),
            UpdateContent::EditedMessage(m) => m.text.as_deref(),
            UpdateContent::ChannelPost(m) => m.text.as_deref(),
            UpdateContent::EditedChannelPost(m) => m.text.as_deref(),
            UpdateContent::CallbackQuery(c) => c.data.as_deref(),
            _ => None,
        }
    }

    /// Get command from message (e.g., "/start" -> "start")
    pub fn command(&self) -> Option<&str> {
        self.text().and_then(|t| {
            let first = t.split_whitespace().next()?;
            first.strip_prefix('/')
        })
    }

    /// Get command arguments (text after the command)
    pub fn command_args(&self) -> Option<&str> {
        self.text().and_then(|t| {
            let mut parts = t.splitn(2, char::is_whitespace);
            parts.next()?; // skip command
            parts.next().map(|s| s.trim())
        })
    }

    /// Get Telegram API client
    pub fn telegram_api(&self) -> AppResult<AsyncApi> {
        let key = self
            .env
            .secret("API_KEY")
            .map_err(|_| worker::Error::RustError("API_KEY not found".to_string()))?
            .to_string();
        Ok(AsyncApi::new(&key))
    }

    /// Reply to the current chat
    pub async fn reply(&self, text: &str) -> AppResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self
            .chat_id()
            .ok_or_else(|| worker::Error::RustError("no chat_id".to_string()))?;

        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .build();

        api.send_message(&params)
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        Ok(())
    }

    /// Reply with HTML formatted text
    pub async fn reply_html(&self, text: &str) -> AppResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self
            .chat_id()
            .ok_or_else(|| worker::Error::RustError("no chat_id".to_string()))?;

        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .parse_mode(frankenstein::ParseMode::Html)
            .build();

        api.send_message(&params)
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        Ok(())
    }

    /// Create an empty OK response
    pub fn ok_response() -> AppResult<Response> {
        Response::ok("")
    }
}

/// Helper to extract chat_id from Update
pub fn extract_chat_id(update: &Update) -> Option<i64> {
    match &update.content {
        UpdateContent::Message(m) => Some(m.chat.id),
        UpdateContent::EditedMessage(m) => Some(m.chat.id),
        UpdateContent::ChannelPost(m) => Some(m.chat.id),
        UpdateContent::EditedChannelPost(m) => Some(m.chat.id),
        UpdateContent::CallbackQuery(c) => c.message.as_ref().map(|m| match m {
            MaybeInaccessibleMessage::Message(msg) => msg.chat.id,
            MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.chat.id,
        }),
        _ => None,
    }
}

/// Helper to extract user_id from Update (returns u64 as per Telegram API)
pub fn extract_user_id(update: &Update) -> Option<u64> {
    match &update.content {
        UpdateContent::Message(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::EditedMessage(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::CallbackQuery(c) => Some(c.from.id),
        UpdateContent::InlineQuery(q) => Some(q.from.id),
        _ => None,
    }
}
