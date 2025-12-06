use core::ops::ControlFlow;

use frankenstein::client_reqwest::Bot;
use frankenstein::input_file::FileUpload;
use frankenstein::methods::{
    AnswerCallbackQueryParams, DeleteMessageParams, EditMessageTextParams, SendMessageParams,
    SendPhotoParams,
};
use frankenstein::ParseMode;
use frankenstein::types::{MaybeInaccessibleMessage, ReplyParameters};
use frankenstein::updates::{Update, UpdateContent};
use frankenstein::AsyncTelegramApi;
use worker::{Env, Response};

use super::storage::{Session, SessionStorage};
use crate::error::{BotError, BotResult};
use crate::{AppResult, Flow};

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
    pub fn telegram_api(&self) -> AppResult<Bot> {
        let key = self
            .env
            .secret("API_KEY")
            .map_err(|_| worker::Error::RustError("API_KEY not found".to_string()))?
            .to_string();
        Ok(Bot::new(&key))
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
            .parse_mode(ParseMode::Html)
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

    // =========================================================================
    // Convenience methods for simplified handler returns
    // =========================================================================

    /// End handler processing with a success response.
    ///
    /// Use this to signal that the handler has completed successfully
    /// and no further handlers should process this update.
    ///
    /// # Example
    /// ```ignore
    /// app.on_command_ctx("hello", storage, |ctx| async move {
    ///     ctx.reply("Hello!").await?;
    ///     Context::done()
    /// });
    /// ```
    pub fn done() -> BotResult<Flow> {
        Ok(ControlFlow::Break(Response::ok("")?))
    }

    /// Skip this handler and continue to the next one.
    ///
    /// Use this when the handler decides it should not process
    /// this update and the next handler should be tried.
    ///
    /// # Example
    /// ```ignore
    /// app.on_update_ctx(storage, |ctx| async move {
    ///     if ctx.text().is_none() {
    ///         return Context::skip();
    ///     }
    ///     // process text message...
    ///     Context::done()
    /// });
    /// ```
    pub fn skip() -> BotResult<Flow> {
        Ok(ControlFlow::Continue(()))
    }

    /// Reply with text and end handler processing.
    ///
    /// This is a convenience method that combines `reply()` and `done()`.
    ///
    /// # Example
    /// ```ignore
    /// app.on_command_ctx("hello", storage, |ctx| async move {
    ///     ctx.reply_and_done("Hello!").await
    /// });
    /// ```
    pub async fn reply_and_done(&self, text: &str) -> BotResult<Flow> {
        self.reply(text).await?;
        Self::done()
    }

    /// Reply with HTML and end handler processing.
    pub async fn reply_html_and_done(&self, text: &str) -> BotResult<Flow> {
        self.reply_html(text).await?;
        Self::done()
    }

    // =========================================================================
    // Extended Telegram API methods
    // =========================================================================

    /// Get the message ID from the current update
    pub fn message_id(&self) -> Option<i32> {
        match &self.update.content {
            UpdateContent::Message(m) => Some(m.message_id),
            UpdateContent::EditedMessage(m) => Some(m.message_id),
            UpdateContent::ChannelPost(m) => Some(m.message_id),
            UpdateContent::EditedChannelPost(m) => Some(m.message_id),
            UpdateContent::CallbackQuery(c) => c.message.as_ref().map(|m| match m {
                MaybeInaccessibleMessage::Message(msg) => msg.message_id,
                MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.message_id,
            }),
            _ => None,
        }
    }

    /// Reply to a specific message (quote reply)
    pub async fn reply_to(&self, text: &str) -> BotResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self.chat_id().ok_or(BotError::MissingField("chat_id"))?;
        let msg_id = self
            .message_id()
            .ok_or(BotError::MissingField("message_id"))?;

        let reply_params = ReplyParameters::builder().message_id(msg_id).build();

        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .reply_parameters(reply_params)
            .build();

        api.send_message(&params).await?;
        Ok(())
    }

    /// Reply to message and end handler processing
    pub async fn reply_to_and_done(&self, text: &str) -> BotResult<Flow> {
        self.reply_to(text).await?;
        Self::done()
    }

    /// Answer a callback query (dismiss the loading state on inline buttons)
    pub async fn answer_callback(&self, text: Option<&str>, show_alert: bool) -> BotResult<()> {
        let callback_id = match &self.update.content {
            UpdateContent::CallbackQuery(c) => c.id.clone(),
            _ => return Err(BotError::MissingField("callback_query_id")),
        };

        let api = self.telegram_api()?;
        let mut params = AnswerCallbackQueryParams::builder()
            .callback_query_id(callback_id)
            .show_alert(show_alert)
            .build();

        if let Some(t) = text {
            params.text = Some(t.to_string());
        }

        api.answer_callback_query(&params).await?;
        Ok(())
    }

    /// Answer callback and end handler processing
    pub async fn answer_callback_and_done(
        &self,
        text: Option<&str>,
        show_alert: bool,
    ) -> BotResult<Flow> {
        self.answer_callback(text, show_alert).await?;
        Self::done()
    }

    /// Edit the text of a message
    pub async fn edit_text(&self, text: &str) -> BotResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self.chat_id().ok_or(BotError::MissingField("chat_id"))?;
        let msg_id = self
            .message_id()
            .ok_or(BotError::MissingField("message_id"))?;

        let params = EditMessageTextParams::builder()
            .chat_id(chat_id)
            .message_id(msg_id)
            .text(text)
            .build();

        api.edit_message_text(&params).await?;
        Ok(())
    }

    /// Edit text and end handler processing
    pub async fn edit_text_and_done(&self, text: &str) -> BotResult<Flow> {
        self.edit_text(text).await?;
        Self::done()
    }

    /// Delete the current message
    pub async fn delete_message(&self) -> BotResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self.chat_id().ok_or(BotError::MissingField("chat_id"))?;
        let msg_id = self
            .message_id()
            .ok_or(BotError::MissingField("message_id"))?;

        let params = DeleteMessageParams::builder()
            .chat_id(chat_id)
            .message_id(msg_id)
            .build();

        api.delete_message(&params).await?;
        Ok(())
    }

    /// Send a photo to the current chat
    pub async fn send_photo(&self, photo: impl Into<FileUpload>) -> BotResult<()> {
        let api = self.telegram_api()?;
        let chat_id = self.chat_id().ok_or(BotError::MissingField("chat_id"))?;

        let params = SendPhotoParams::builder()
            .chat_id(chat_id)
            .photo(photo)
            .build();

        api.send_photo(&params).await?;
        Ok(())
    }

    /// Get callback data from callback query
    pub fn callback_data(&self) -> Option<&str> {
        match &self.update.content {
            UpdateContent::CallbackQuery(c) => c.data.as_deref(),
            _ => None,
        }
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
