//! Paginated menu system for interactive inline button menus.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::menu::Menu;
//!
//! let menu = Menu::new("settings")
//!     .item("Profile", "profile")
//!     .item("Notifications", "notifications")
//!     .item("Privacy", "privacy")
//!     .item("Language", "language")
//!     .item("Help", "help")
//!     .page_size(3);
//!
//! // Build page 0 (first page)
//! let keyboard = menu.build_page(0);
//! bot.send_inline_keyboard(chat_id, "Settings:", keyboard).await?;
//!
//! // Handle pagination callback
//! if let Some(page) = Menu::parse_page(callback_data, "settings") {
//!     let keyboard = menu.build_page(page);
//!     bot.edit_with_keyboard(chat_id, msg_id, "Settings:", keyboard.build()).await?;
//! }
//! ```

use crate::keyboard::{InlineButton, InlineKeyboard};

/// A menu item with display text and callback data.
#[derive(Clone, Debug)]
pub struct MenuItem {
    /// Display text on the button
    pub text: String,
    /// Callback data sent when clicked
    pub data: String,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(text: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            data: data.into(),
        }
    }
}

/// A paginated menu builder.
///
/// Creates inline keyboards with pagination controls.
#[derive(Clone, Debug)]
pub struct Menu {
    items: Vec<MenuItem>,
    page_size: usize,
    prefix: String,
    columns: usize,
}

impl Menu {
    /// Create a new menu with a callback prefix.
    ///
    /// The prefix is used to namespace pagination callbacks.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            page_size: 5,
            prefix: prefix.into(),
            columns: 1,
        }
    }

    /// Add a menu item.
    pub fn item(mut self, text: impl Into<String>, data: impl Into<String>) -> Self {
        self.items.push(MenuItem::new(text, data));
        self
    }

    /// Add multiple items at once.
    pub fn items(mut self, items: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (text, data) in items {
            self.items.push(MenuItem::new(text, data));
        }
        self
    }

    /// Set the number of items per page (default: 5).
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(1);
        self
    }

    /// Set the number of columns for items (default: 1).
    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }

    /// Get the total number of pages.
    pub fn total_pages(&self) -> usize {
        if self.items.is_empty() {
            1
        } else {
            self.items.len().div_ceil(self.page_size)
        }
    }

    /// Build the keyboard for a specific page.
    ///
    /// Page numbers are 0-indexed.
    pub fn build_page(&self, page: usize) -> InlineKeyboard {
        let total_pages = self.total_pages();
        let page = page.min(total_pages.saturating_sub(1));

        let start = page * self.page_size;
        let end = (start + self.page_size).min(self.items.len());

        let mut keyboard = InlineKeyboard::new();

        // Add menu items
        let page_items: Vec<_> = self.items[start..end].iter().collect();

        if self.columns == 1 {
            // Single column: each item is its own row
            for item in page_items {
                keyboard = keyboard.button(InlineButton::callback(&item.text, &item.data));
            }
        } else {
            // Multiple columns: group items into rows
            for chunk in page_items.chunks(self.columns) {
                let buttons: Vec<_> = chunk
                    .iter()
                    .map(|item| InlineButton::callback(&item.text, &item.data))
                    .collect();
                keyboard = keyboard.row(buttons);
            }
        }

        // Add pagination controls if needed
        if total_pages > 1 {
            let mut nav_buttons = Vec::new();

            // Previous button
            if page > 0 {
                nav_buttons.push(InlineButton::callback(
                    "◀️ Prev",
                    format!("{}:page:{}", self.prefix, page - 1),
                ));
            }

            // Page indicator
            nav_buttons.push(InlineButton::callback(
                format!("{}/{}", page + 1, total_pages),
                format!("{}:noop", self.prefix),
            ));

            // Next button
            if page < total_pages - 1 {
                nav_buttons.push(InlineButton::callback(
                    "Next ▶️",
                    format!("{}:page:{}", self.prefix, page + 1),
                ));
            }

            keyboard = keyboard.row(nav_buttons);
        }

        keyboard
    }

    /// Parse a page number from callback data.
    ///
    /// Returns `Some(page)` if the callback matches `{prefix}:page:{n}`.
    pub fn parse_page(data: &str, prefix: &str) -> Option<usize> {
        let page_prefix = format!("{}:page:", prefix);
        if data.starts_with(&page_prefix) {
            data[page_prefix.len()..].parse().ok()
        } else {
            None
        }
    }

    /// Check if callback data is a no-op (page indicator click).
    pub fn is_noop(data: &str, prefix: &str) -> bool {
        data == format!("{}:noop", prefix)
    }

    /// Get the prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_basic() {
        let menu = Menu::new("test")
            .item("Item 1", "item1")
            .item("Item 2", "item2")
            .item("Item 3", "item3");

        assert_eq!(menu.total_pages(), 1);
    }

    #[test]
    fn test_menu_pagination() {
        let menu = Menu::new("test")
            .item("Item 1", "item1")
            .item("Item 2", "item2")
            .item("Item 3", "item3")
            .item("Item 4", "item4")
            .item("Item 5", "item5")
            .item("Item 6", "item6")
            .page_size(2);

        assert_eq!(menu.total_pages(), 3);
    }

    #[test]
    fn test_parse_page() {
        assert_eq!(Menu::parse_page("settings:page:0", "settings"), Some(0));
        assert_eq!(Menu::parse_page("settings:page:5", "settings"), Some(5));
        assert_eq!(Menu::parse_page("settings:page:abc", "settings"), None);
        assert_eq!(Menu::parse_page("other:page:0", "settings"), None);
    }

    #[test]
    fn test_is_noop() {
        assert!(Menu::is_noop("settings:noop", "settings"));
        assert!(!Menu::is_noop("settings:page:0", "settings"));
        assert!(!Menu::is_noop("other:noop", "settings"));
    }

    #[test]
    fn test_empty_menu() {
        let menu = Menu::new("empty");
        assert_eq!(menu.total_pages(), 1);
    }
}
