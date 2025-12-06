//! Simplified Bot wrapper for Telegram API interactions.
//!
//! This module provides a `Bot` struct that wraps the frankenstein API client
//! and offers simplified methods for common Telegram operations.

use frankenstein::client_reqwest::Bot as FrankensteinBot;
use frankenstein::methods::{
    AnswerCallbackQueryParams, DeleteMessageParams, EditMessageTextParams, SendMessageParams,
    SendPhotoParams,
};
use frankenstein::types::{
    InlineKeyboardMarkup, ReplyKeyboardMarkup, ReplyMarkup, ReplyParameters,
};
use frankenstein::{AsyncTelegramApi, ParseMode};
use worker::Env;

use crate::error::{BotError, BotResult};
use crate::keyboard::{InlineKeyboard, ReplyKeyboard};
use crate::message::Message;

/// Simplified Bot wrapper that hides frankenstein complexity.
///
/// # Example
/// ```ignore
/// let bot = Bot::from_env(&env)?;
/// bot.send_message(chat_id, "Hello!").await?;
/// ```
#[derive(Clone)]
pub struct Bot {
    api: FrankensteinBot,
}

impl Bot {
    /// Create a new Bot from API key.
    pub fn new(api_key: &str) -> Self {
        Self {
            api: FrankensteinBot::new(api_key),
        }
    }

    /// Create a Bot from environment variable `API_KEY`.
    pub fn from_env(env: &Env) -> BotResult<Self> {
        let key = env
            .secret("API_KEY")
            .map_err(|_| BotError::MissingField("API_KEY"))?
            .to_string();
        Ok(Self::new(&key))
    }

    /// Get the underlying frankenstein Bot for advanced usage.
    pub fn inner(&self) -> &FrankensteinBot {
        &self.api
    }

    /// Send a text message to a chat.
    pub async fn send_message(&self, chat_id: i64, text: &str) -> BotResult<()> {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Send an HTML-formatted message to a chat.
    pub async fn send_html(&self, chat_id: i64, text: &str) -> BotResult<()> {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .parse_mode(ParseMode::Html)
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Reply to a message (quote reply).
    pub async fn reply(&self, msg: &Message, text: &str) -> BotResult<()> {
        let reply_params = ReplyParameters::builder()
            .message_id(msg.message_id())
            .build();
        let params = SendMessageParams::builder()
            .chat_id(msg.chat_id())
            .text(text)
            .reply_parameters(reply_params)
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Reply to a message with HTML formatting.
    pub async fn reply_html(&self, msg: &Message, text: &str) -> BotResult<()> {
        let reply_params = ReplyParameters::builder()
            .message_id(msg.message_id())
            .build();
        let params = SendMessageParams::builder()
            .chat_id(msg.chat_id())
            .text(text)
            .parse_mode(ParseMode::Html)
            .reply_parameters(reply_params)
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Answer a callback query.
    pub async fn answer_callback(
        &self,
        callback_id: &str,
        text: Option<&str>,
        show_alert: bool,
    ) -> BotResult<()> {
        let mut params = AnswerCallbackQueryParams::builder()
            .callback_query_id(callback_id)
            .show_alert(show_alert)
            .build();
        if let Some(t) = text {
            params.text = Some(t.to_string());
        }
        self.api.answer_callback_query(&params).await?;
        Ok(())
    }

    /// Edit a message's text.
    pub async fn edit_message(&self, chat_id: i64, message_id: i32, text: &str) -> BotResult<()> {
        let params = EditMessageTextParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .text(text)
            .build();
        self.api.edit_message_text(&params).await?;
        Ok(())
    }

    /// Delete a message.
    pub async fn delete_message(&self, chat_id: i64, message_id: i32) -> BotResult<()> {
        let params = DeleteMessageParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .build();
        self.api.delete_message(&params).await?;
        Ok(())
    }

    /// Send a photo to a chat.
    pub async fn send_photo(
        &self,
        chat_id: i64,
        photo: impl Into<frankenstein::input_file::FileUpload>,
    ) -> BotResult<()> {
        let params = SendPhotoParams::builder()
            .chat_id(chat_id)
            .photo(photo)
            .build();
        self.api.send_photo(&params).await?;
        Ok(())
    }

    // =========================================================================
    // Keyboard methods
    // =========================================================================

    /// Send a message with an inline keyboard.
    ///
    /// # Example
    /// ```ignore
    /// let keyboard = InlineKeyboard::new()
    ///     .row([InlineButton::callback("Yes", "yes"), InlineButton::callback("No", "no")]);
    /// bot.send_with_keyboard(chat_id, "Choose:", keyboard).await?;
    /// ```
    pub async fn send_with_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: impl Into<InlineKeyboardMarkup>,
    ) -> BotResult<()> {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard.into()))
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Send a message with a reply keyboard.
    pub async fn send_with_reply_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: impl Into<ReplyKeyboardMarkup>,
    ) -> BotResult<()> {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .reply_markup(ReplyMarkup::ReplyKeyboardMarkup(keyboard.into()))
            .build();
        self.api.send_message(&params).await?;
        Ok(())
    }

    /// Edit a message with a new inline keyboard.
    pub async fn edit_with_keyboard(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
        keyboard: impl Into<InlineKeyboardMarkup>,
    ) -> BotResult<()> {
        let params = EditMessageTextParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .text(text)
            .reply_markup(keyboard.into())
            .build();
        self.api.edit_message_text(&params).await?;
        Ok(())
    }

    /// Convenience: send with InlineKeyboard builder.
    pub async fn send_inline_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: InlineKeyboard,
    ) -> BotResult<()> {
        self.send_with_keyboard(chat_id, text, keyboard.build())
            .await
    }

    /// Convenience: send with ReplyKeyboard builder.
    pub async fn send_reply_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: ReplyKeyboard,
    ) -> BotResult<()> {
        self.send_with_reply_keyboard(chat_id, text, keyboard.build())
            .await
    }
}
