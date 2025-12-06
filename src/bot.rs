//! Simplified Bot wrapper for Telegram API interactions.
//!
//! This module provides a `Bot` struct that wraps the frankenstein API client
//! and offers simplified methods for common Telegram operations.

use frankenstein::client_reqwest::Bot as FrankensteinBot;
use frankenstein::methods::{
    AnswerCallbackQueryParams, BanChatMemberParams, DeleteMessageParams, DeleteMyCommandsParams,
    EditMessageTextParams, GetMyCommandsParams, PinChatMessageParams, PromoteChatMemberParams,
    RestrictChatMemberParams, SendMessageParams, SendPhotoParams, SetChatDescriptionParams,
    SetChatTitleParams, SetMyCommandsParams, UnbanChatMemberParams, UnpinAllChatMessagesParams,
    UnpinChatMessageParams,
};
use frankenstein::types::BotCommand;
use frankenstein::types::{
    ChatPermissions, InlineKeyboardMarkup, ReplyKeyboardMarkup, ReplyMarkup, ReplyParameters,
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

    // =========================================================================
    // Admin methods
    // =========================================================================

    /// Ban a user from a chat (group, supergroup, or channel).
    ///
    /// The bot must be an administrator with the appropriate permissions.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `user_id` - User to ban
    /// * `until_date` - Optional Unix timestamp when the ban will be lifted (0 or None = permanent)
    /// * `revoke_messages` - Delete all messages from the user in the chat
    pub async fn ban_chat_member(
        &self,
        chat_id: i64,
        user_id: u64,
        until_date: Option<u64>,
        revoke_messages: bool,
    ) -> BotResult<()> {
        let mut params = BanChatMemberParams::builder()
            .chat_id(chat_id)
            .user_id(user_id)
            .revoke_messages(revoke_messages)
            .build();
        params.until_date = until_date;
        self.api.ban_chat_member(&params).await?;
        Ok(())
    }

    /// Unban a previously banned user in a chat.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `user_id` - User to unban
    /// * `only_if_banned` - Only unban if the user is currently banned
    pub async fn unban_chat_member(
        &self,
        chat_id: i64,
        user_id: u64,
        only_if_banned: bool,
    ) -> BotResult<()> {
        let params = UnbanChatMemberParams::builder()
            .chat_id(chat_id)
            .user_id(user_id)
            .only_if_banned(only_if_banned)
            .build();
        self.api.unban_chat_member(&params).await?;
        Ok(())
    }

    /// Restrict a user in a supergroup.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `user_id` - User to restrict
    /// * `permissions` - New user permissions
    /// * `until_date` - Optional Unix timestamp when restrictions will be lifted
    pub async fn restrict_chat_member(
        &self,
        chat_id: i64,
        user_id: u64,
        permissions: ChatPermissions,
        until_date: Option<u64>,
    ) -> BotResult<()> {
        let mut params = RestrictChatMemberParams::builder()
            .chat_id(chat_id)
            .user_id(user_id)
            .permissions(permissions)
            .build();
        params.until_date = until_date;
        self.api.restrict_chat_member(&params).await?;
        Ok(())
    }

    /// Promote or demote a user in a supergroup or channel.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `user_id` - User to promote/demote
    /// * `rights` - Admin rights to grant (use `AdminRights` builder)
    pub async fn promote_chat_member(
        &self,
        chat_id: i64,
        user_id: u64,
        rights: AdminRights,
    ) -> BotResult<()> {
        let mut params = PromoteChatMemberParams::builder()
            .chat_id(chat_id)
            .user_id(user_id)
            .build();
        // Apply admin rights
        params.can_manage_chat = Some(rights.can_manage_chat);
        params.can_delete_messages = Some(rights.can_delete_messages);
        params.can_manage_video_chats = Some(rights.can_manage_video_chats);
        params.can_restrict_members = Some(rights.can_restrict_members);
        params.can_promote_members = Some(rights.can_promote_members);
        params.can_change_info = Some(rights.can_change_info);
        params.can_invite_users = Some(rights.can_invite_users);
        params.can_post_messages = Some(rights.can_post_messages);
        params.can_edit_messages = Some(rights.can_edit_messages);
        params.can_pin_messages = Some(rights.can_pin_messages);
        params.can_post_stories = Some(rights.can_post_stories);
        params.can_edit_stories = Some(rights.can_edit_stories);
        params.can_delete_stories = Some(rights.can_delete_stories);
        params.can_manage_topics = Some(rights.can_manage_topics);
        self.api.promote_chat_member(&params).await?;
        Ok(())
    }

    /// Pin a message in a chat.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `message_id` - Message to pin
    /// * `disable_notification` - Don't send notification to all members
    pub async fn pin_message(
        &self,
        chat_id: i64,
        message_id: i32,
        disable_notification: bool,
    ) -> BotResult<()> {
        let params = PinChatMessageParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .disable_notification(disable_notification)
            .build();
        self.api.pin_chat_message(&params).await?;
        Ok(())
    }

    /// Unpin a specific message in a chat.
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `message_id` - Message to unpin (None = unpin the most recent pinned message)
    pub async fn unpin_message(&self, chat_id: i64, message_id: Option<i32>) -> BotResult<()> {
        let mut params = UnpinChatMessageParams::builder().chat_id(chat_id).build();
        params.message_id = message_id;
        self.api.unpin_chat_message(&params).await?;
        Ok(())
    }

    /// Unpin all messages in a chat.
    pub async fn unpin_all_messages(&self, chat_id: i64) -> BotResult<()> {
        let params = UnpinAllChatMessagesParams::builder()
            .chat_id(chat_id)
            .build();
        self.api.unpin_all_chat_messages(&params).await?;
        Ok(())
    }

    /// Set the title of a chat.
    pub async fn set_chat_title(&self, chat_id: i64, title: &str) -> BotResult<()> {
        let params = SetChatTitleParams::builder()
            .chat_id(chat_id)
            .title(title)
            .build();
        self.api.set_chat_title(&params).await?;
        Ok(())
    }

    /// Set the description of a chat.
    pub async fn set_chat_description(&self, chat_id: i64, description: &str) -> BotResult<()> {
        let params = SetChatDescriptionParams::builder()
            .chat_id(chat_id)
            .description(description)
            .build();
        self.api.set_chat_description(&params).await?;
        Ok(())
    }

    // =========================================================================
    // Bot Commands Menu
    // =========================================================================

    /// Set the bot's command menu.
    ///
    /// # Example
    /// ```ignore
    /// bot.set_my_commands(&[
    ///     ("start", "Start the bot"),
    ///     ("help", "Show help message"),
    ///     ("settings", "Open settings"),
    /// ]).await?;
    /// ```
    pub async fn set_my_commands(&self, commands: &[(&str, &str)]) -> BotResult<bool> {
        let bot_commands: Vec<BotCommand> = commands
            .iter()
            .map(|(cmd, desc)| {
                BotCommand::builder()
                    .command(cmd.to_string())
                    .description(desc.to_string())
                    .build()
            })
            .collect();

        let params = SetMyCommandsParams::builder()
            .commands(bot_commands)
            .build();
        let result = self.api.set_my_commands(&params).await?;
        Ok(result.result)
    }

    /// Delete the bot's command menu.
    pub async fn delete_my_commands(&self) -> BotResult<bool> {
        let params = DeleteMyCommandsParams::builder().build();
        let result = self.api.delete_my_commands(&params).await?;
        Ok(result.result)
    }

    /// Get the current bot command menu.
    pub async fn get_my_commands(&self) -> BotResult<Vec<(String, String)>> {
        let params = GetMyCommandsParams::builder().build();
        let result = self.api.get_my_commands(&params).await?;
        Ok(result
            .result
            .into_iter()
            .map(|c| (c.command, c.description))
            .collect())
    }

    /// Kick a user from a chat (ban and immediately unban).
    ///
    /// This removes the user but allows them to rejoin.
    pub async fn kick_chat_member(&self, chat_id: i64, user_id: u64) -> BotResult<()> {
        self.ban_chat_member(chat_id, user_id, None, false).await?;
        self.unban_chat_member(chat_id, user_id, true).await?;
        Ok(())
    }

    /// Mute a user (restrict all messaging permissions).
    ///
    /// # Arguments
    /// * `chat_id` - Target chat ID
    /// * `user_id` - User to mute
    /// * `until_date` - Optional Unix timestamp when the mute will be lifted
    pub async fn mute_chat_member(
        &self,
        chat_id: i64,
        user_id: u64,
        until_date: Option<u64>,
    ) -> BotResult<()> {
        let permissions = ChatPermissions::builder()
            .can_send_messages(false)
            .can_send_audios(false)
            .can_send_documents(false)
            .can_send_photos(false)
            .can_send_videos(false)
            .can_send_video_notes(false)
            .can_send_voice_notes(false)
            .can_send_polls(false)
            .can_send_other_messages(false)
            .can_add_web_page_previews(false)
            .build();
        self.restrict_chat_member(chat_id, user_id, permissions, until_date)
            .await
    }

    /// Unmute a user (restore default permissions).
    pub async fn unmute_chat_member(&self, chat_id: i64, user_id: u64) -> BotResult<()> {
        let permissions = ChatPermissions::builder()
            .can_send_messages(true)
            .can_send_audios(true)
            .can_send_documents(true)
            .can_send_photos(true)
            .can_send_videos(true)
            .can_send_video_notes(true)
            .can_send_voice_notes(true)
            .can_send_polls(true)
            .can_send_other_messages(true)
            .can_add_web_page_previews(true)
            .build();
        self.restrict_chat_member(chat_id, user_id, permissions, None)
            .await
    }
}

// =========================================================================
// Admin rights builder
// =========================================================================

/// Builder for admin rights when promoting a chat member.
///
/// # Example
/// ```ignore
/// let rights = AdminRights::new()
///     .can_delete_messages(true)
///     .can_restrict_members(true)
///     .can_pin_messages(true);
/// bot.promote_chat_member(chat_id, user_id, rights).await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdminRights {
    pub can_manage_chat: bool,
    pub can_delete_messages: bool,
    pub can_manage_video_chats: bool,
    pub can_restrict_members: bool,
    pub can_promote_members: bool,
    pub can_change_info: bool,
    pub can_invite_users: bool,
    pub can_post_messages: bool,
    pub can_edit_messages: bool,
    pub can_pin_messages: bool,
    pub can_post_stories: bool,
    pub can_edit_stories: bool,
    pub can_delete_stories: bool,
    pub can_manage_topics: bool,
}

impl AdminRights {
    /// Create a new AdminRights with all permissions set to false.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create AdminRights with full admin permissions.
    pub fn full() -> Self {
        Self {
            can_manage_chat: true,
            can_delete_messages: true,
            can_manage_video_chats: true,
            can_restrict_members: true,
            can_promote_members: true,
            can_change_info: true,
            can_invite_users: true,
            can_post_messages: true,
            can_edit_messages: true,
            can_pin_messages: true,
            can_post_stories: true,
            can_edit_stories: true,
            can_delete_stories: true,
            can_manage_topics: true,
        }
    }

    pub fn can_manage_chat(mut self, v: bool) -> Self {
        self.can_manage_chat = v;
        self
    }
    pub fn can_delete_messages(mut self, v: bool) -> Self {
        self.can_delete_messages = v;
        self
    }
    pub fn can_manage_video_chats(mut self, v: bool) -> Self {
        self.can_manage_video_chats = v;
        self
    }
    pub fn can_restrict_members(mut self, v: bool) -> Self {
        self.can_restrict_members = v;
        self
    }
    pub fn can_promote_members(mut self, v: bool) -> Self {
        self.can_promote_members = v;
        self
    }
    pub fn can_change_info(mut self, v: bool) -> Self {
        self.can_change_info = v;
        self
    }
    pub fn can_invite_users(mut self, v: bool) -> Self {
        self.can_invite_users = v;
        self
    }
    pub fn can_post_messages(mut self, v: bool) -> Self {
        self.can_post_messages = v;
        self
    }
    pub fn can_edit_messages(mut self, v: bool) -> Self {
        self.can_edit_messages = v;
        self
    }
    pub fn can_pin_messages(mut self, v: bool) -> Self {
        self.can_pin_messages = v;
        self
    }
    pub fn can_post_stories(mut self, v: bool) -> Self {
        self.can_post_stories = v;
        self
    }
    pub fn can_edit_stories(mut self, v: bool) -> Self {
        self.can_edit_stories = v;
        self
    }
    pub fn can_delete_stories(mut self, v: bool) -> Self {
        self.can_delete_stories = v;
        self
    }
    pub fn can_manage_topics(mut self, v: bool) -> Self {
        self.can_manage_topics = v;
        self
    }
}
