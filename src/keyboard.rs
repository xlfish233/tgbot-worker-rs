//! Keyboard builders for Telegram inline and reply keyboards.
//!
//! Inspired by [teloxide](https://github.com/teloxide/teloxide)'s keyboard API.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::keyboard::{InlineKeyboard, InlineButton};
//!
//! let keyboard = InlineKeyboard::new()
//!     .row([
//!         InlineButton::callback("Yes", "yes"),
//!         InlineButton::callback("No", "no"),
//!     ])
//!     .row([InlineButton::url("Visit", "https://example.com")]);
//! ```

use frankenstein::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, ReplyKeyboardMarkup,
    ReplyKeyboardRemove,
};
use serde::{Deserialize, Serialize};

/// Builder for inline keyboards (buttons below messages).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InlineKeyboard {
    rows: Vec<Vec<InlineKeyboardButton>>,
}

impl InlineKeyboard {
    /// Create a new empty inline keyboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an inline keyboard from rows of buttons.
    pub fn from_rows<I, R>(rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = InlineKeyboardButton>,
    {
        Self {
            rows: rows.into_iter().map(|r| r.into_iter().collect()).collect(),
        }
    }

    /// Add a row of buttons.
    pub fn row<I>(mut self, buttons: I) -> Self
    where
        I: IntoIterator<Item = InlineKeyboardButton>,
    {
        self.rows.push(buttons.into_iter().collect());
        self
    }

    /// Add a single button as a row.
    pub fn button(self, button: InlineKeyboardButton) -> Self {
        self.row([button])
    }

    /// Append a button to the last row (or create a new row if empty).
    pub fn append(mut self, button: InlineKeyboardButton) -> Self {
        if let Some(last_row) = self.rows.last_mut() {
            last_row.push(button);
        } else {
            self.rows.push(vec![button]);
        }
        self
    }

    /// Build the keyboard markup.
    pub fn build(self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup {
            inline_keyboard: self.rows,
        }
    }
}

impl From<InlineKeyboard> for InlineKeyboardMarkup {
    fn from(kb: InlineKeyboard) -> Self {
        kb.build()
    }
}

/// Helper for creating inline keyboard buttons.
pub struct InlineButton;

impl InlineButton {
    /// Create a callback button.
    ///
    /// When pressed, sends `callback_data` to the bot.
    pub fn callback(
        text: impl Into<String>,
        callback_data: impl Into<String>,
    ) -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(text)
            .callback_data(callback_data)
            .build()
    }

    /// Create a URL button.
    ///
    /// When pressed, opens the URL.
    pub fn url(text: impl Into<String>, url: impl Into<String>) -> InlineKeyboardButton {
        InlineKeyboardButton::builder().text(text).url(url).build()
    }

    /// Create a switch inline query button.
    ///
    /// When pressed, prompts the user to select a chat and inserts the bot's
    /// username and the specified query in the input field.
    pub fn switch_inline(
        text: impl Into<String>,
        query: impl Into<String>,
    ) -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(text)
            .switch_inline_query(query)
            .build()
    }

    /// Create a switch inline query button for the current chat.
    pub fn switch_inline_current(
        text: impl Into<String>,
        query: impl Into<String>,
    ) -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(text)
            .switch_inline_query_current_chat(query)
            .build()
    }
}

/// Builder for reply keyboards (custom keyboard replacing the default).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReplyKeyboard {
    rows: Vec<Vec<KeyboardButton>>,
    resize: bool,
    one_time: bool,
    input_placeholder: Option<String>,
    selective: bool,
    persistent: bool,
}

impl ReplyKeyboard {
    /// Create a new empty reply keyboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a row of buttons.
    pub fn row<I>(mut self, buttons: I) -> Self
    where
        I: IntoIterator<Item = KeyboardButton>,
    {
        self.rows.push(buttons.into_iter().collect());
        self
    }

    /// Add a single text button as a row.
    pub fn text(self, text: impl Into<String>) -> Self {
        self.row([ReplyButton::text(text)])
    }

    /// Resize keyboard to fit buttons (smaller if possible).
    pub fn resize(mut self) -> Self {
        self.resize = true;
        self
    }

    /// Hide keyboard after a button is pressed.
    pub fn one_time(mut self) -> Self {
        self.one_time = true;
        self
    }

    /// Set placeholder text in the input field.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.input_placeholder = Some(text.into());
        self
    }

    /// Show keyboard only to specific users.
    pub fn selective(mut self) -> Self {
        self.selective = true;
        self
    }

    /// Keep keyboard persistent.
    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    /// Build the keyboard markup.
    pub fn build(self) -> ReplyKeyboardMarkup {
        let mut markup = ReplyKeyboardMarkup::builder().keyboard(self.rows).build();
        if self.resize {
            markup.resize_keyboard = Some(true);
        }
        if self.one_time {
            markup.one_time_keyboard = Some(true);
        }
        markup.input_field_placeholder = self.input_placeholder;
        if self.selective {
            markup.selective = Some(true);
        }
        if self.persistent {
            markup.is_persistent = Some(true);
        }
        markup
    }
}

impl From<ReplyKeyboard> for ReplyKeyboardMarkup {
    fn from(kb: ReplyKeyboard) -> Self {
        kb.build()
    }
}

/// Helper for creating reply keyboard buttons.
pub struct ReplyButton;

impl ReplyButton {
    /// Create a simple text button.
    pub fn text(text: impl Into<String>) -> KeyboardButton {
        KeyboardButton::builder().text(text).build()
    }

    /// Create a button that requests the user's phone number.
    pub fn request_contact(text: impl Into<String>) -> KeyboardButton {
        KeyboardButton::builder()
            .text(text)
            .request_contact(true)
            .build()
    }

    /// Create a button that requests the user's location.
    pub fn request_location(text: impl Into<String>) -> KeyboardButton {
        KeyboardButton::builder()
            .text(text)
            .request_location(true)
            .build()
    }
}

/// Remove reply keyboard.
pub fn remove_keyboard() -> ReplyKeyboardRemove {
    ReplyKeyboardRemove::builder().remove_keyboard(true).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_keyboard() {
        let kb = InlineKeyboard::new()
            .row([
                InlineButton::callback("A", "a"),
                InlineButton::callback("B", "b"),
            ])
            .button(InlineButton::url("Link", "https://example.com"));

        let markup = kb.build();
        assert_eq!(markup.inline_keyboard.len(), 2);
        assert_eq!(markup.inline_keyboard[0].len(), 2);
        assert_eq!(markup.inline_keyboard[1].len(), 1);
    }

    #[test]
    fn test_reply_keyboard() {
        let kb = ReplyKeyboard::new()
            .text("Option 1")
            .text("Option 2")
            .resize()
            .one_time();

        let markup = kb.build();
        assert_eq!(markup.keyboard.len(), 2);
        assert_eq!(markup.resize_keyboard, Some(true));
        assert_eq!(markup.one_time_keyboard, Some(true));
    }
}
