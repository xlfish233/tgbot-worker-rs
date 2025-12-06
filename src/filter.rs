//! Filter combinators for update handling.
//!
//! Filters can be used to conditionally handle updates.

use frankenstein::types::ChatType;
use frankenstein::updates::{Update, UpdateContent};

/// Check if update is a message
pub fn is_message(u: &Update) -> bool {
    matches!(u.content, UpdateContent::Message(_))
}

/// Check if update is an edited message
pub fn is_edited_message(u: &Update) -> bool {
    matches!(u.content, UpdateContent::EditedMessage(_))
}

/// Check if update is a channel post
pub fn is_channel_post(u: &Update) -> bool {
    matches!(u.content, UpdateContent::ChannelPost(_))
}

/// Check if update is a callback query
pub fn is_callback_query(u: &Update) -> bool {
    matches!(u.content, UpdateContent::CallbackQuery(_))
}

/// Check if update is an inline query
pub fn is_inline_query(u: &Update) -> bool {
    matches!(u.content, UpdateContent::InlineQuery(_))
}

/// Check if update has text content
pub fn has_text(u: &Update) -> bool {
    get_text(u).is_some()
}

/// Check if text contains a substring
pub fn text_contains(substring: &'static str) -> impl Fn(&Update) -> bool {
    move |u| get_text(u).is_some_and(|t| t.contains(substring))
}

/// Check if text starts with a prefix
pub fn text_starts_with(prefix: &'static str) -> impl Fn(&Update) -> bool {
    move |u| get_text(u).is_some_and(|t| t.starts_with(prefix))
}

/// Check if text matches exactly
pub fn text_equals(text: &'static str) -> impl Fn(&Update) -> bool {
    move |u| get_text(u).is_some_and(|t| t == text)
}

/// Check if message is a command (starts with /)
pub fn is_command(u: &Update) -> bool {
    get_text(u).is_some_and(|t| t.starts_with('/'))
}

/// Check if callback data equals a value
pub fn callback_data_equals(data: &'static str) -> impl Fn(&Update) -> bool {
    move |u| match &u.content {
        UpdateContent::CallbackQuery(c) => c.data.as_deref() == Some(data),
        _ => false,
    }
}

/// Check if callback data starts with a prefix
pub fn callback_data_starts_with(prefix: &'static str) -> impl Fn(&Update) -> bool {
    move |u| match &u.content {
        UpdateContent::CallbackQuery(c) => c.data.as_ref().is_some_and(|d| d.starts_with(prefix)),
        _ => false,
    }
}

/// Check if message is from a specific chat
pub fn from_chat(chat_id: i64) -> impl Fn(&Update) -> bool {
    move |u| get_chat_id(u) == Some(chat_id)
}

/// Check if message is from a specific user
pub fn from_user(user_id: u64) -> impl Fn(&Update) -> bool {
    move |u| get_user_id(u) == Some(user_id)
}

/// Combine two filters with AND logic
pub fn and<F1, F2>(f1: F1, f2: F2) -> impl Fn(&Update) -> bool
where
    F1: Fn(&Update) -> bool,
    F2: Fn(&Update) -> bool,
{
    move |u| f1(u) && f2(u)
}

/// Combine two filters with OR logic
pub fn or<F1, F2>(f1: F1, f2: F2) -> impl Fn(&Update) -> bool
where
    F1: Fn(&Update) -> bool,
    F2: Fn(&Update) -> bool,
{
    move |u| f1(u) || f2(u)
}

/// Negate a filter
pub fn not<F>(f: F) -> impl Fn(&Update) -> bool
where
    F: Fn(&Update) -> bool,
{
    move |u| !f(u)
}

// =========================================================================
// Chat type filters (Guards)
// =========================================================================

/// Check if update is from a private chat
pub fn is_private(u: &Update) -> bool {
    get_chat_type(u) == Some(ChatType::Private)
}

/// Check if update is from a group (group or supergroup)
pub fn is_group(u: &Update) -> bool {
    matches!(
        get_chat_type(u),
        Some(ChatType::Group) | Some(ChatType::Supergroup)
    )
}

/// Check if update is from a supergroup specifically
pub fn is_supergroup(u: &Update) -> bool {
    get_chat_type(u) == Some(ChatType::Supergroup)
}

/// Check if update is from a channel
pub fn is_channel(u: &Update) -> bool {
    get_chat_type(u) == Some(ChatType::Channel)
}

/// Check if user is in a list of allowed user IDs (admin whitelist)
///
/// # Example
/// ```ignore
/// const ADMINS: &[u64] = &[123456789, 987654321];
/// if is_user_in(ADMINS)(&update) {
///     // User is an admin
/// }
/// ```
pub fn is_user_in(user_ids: &'static [u64]) -> impl Fn(&Update) -> bool {
    move |u| {
        get_user_id(u)
            .map(|id| user_ids.contains(&id))
            .unwrap_or(false)
    }
}

// =========================================================================
// Time-based filters
// =========================================================================

/// Filter out updates older than specified seconds.
///
/// Useful for ignoring stale updates that accumulated while bot was offline.
///
/// # Example
/// ```ignore
/// // Only process updates from the last 60 seconds
/// if is_recent(60)(&update) {
///     // Process update
/// }
/// ```
pub fn is_recent(max_age_seconds: u64) -> impl Fn(&Update) -> bool {
    move |u| {
        let msg_date = get_message_date(u);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // If we can't get the date, assume it's recent
        msg_date
            .map(|d| now.saturating_sub(d) <= max_age_seconds)
            .unwrap_or(true)
    }
}

// =========================================================================
// Helper functions
// =========================================================================

fn get_text(u: &Update) -> Option<&str> {
    match &u.content {
        UpdateContent::Message(m) => m.text.as_deref(),
        UpdateContent::EditedMessage(m) => m.text.as_deref(),
        UpdateContent::ChannelPost(m) => m.text.as_deref(),
        UpdateContent::EditedChannelPost(m) => m.text.as_deref(),
        UpdateContent::CallbackQuery(c) => c.data.as_deref(),
        _ => None,
    }
}

fn get_chat_id(u: &Update) -> Option<i64> {
    match &u.content {
        UpdateContent::Message(m) => Some(m.chat.id),
        UpdateContent::EditedMessage(m) => Some(m.chat.id),
        UpdateContent::ChannelPost(m) => Some(m.chat.id),
        UpdateContent::EditedChannelPost(m) => Some(m.chat.id),
        _ => None,
    }
}

fn get_user_id(u: &Update) -> Option<u64> {
    match &u.content {
        UpdateContent::Message(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::EditedMessage(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::CallbackQuery(c) => Some(c.from.id),
        UpdateContent::InlineQuery(q) => Some(q.from.id),
        _ => None,
    }
}

fn get_chat_type(u: &Update) -> Option<ChatType> {
    match &u.content {
        UpdateContent::Message(m) => Some(m.chat.type_field),
        UpdateContent::EditedMessage(m) => Some(m.chat.type_field),
        UpdateContent::ChannelPost(m) => Some(m.chat.type_field),
        UpdateContent::EditedChannelPost(m) => Some(m.chat.type_field),
        UpdateContent::CallbackQuery(c) => c.message.as_ref().map(|m| match m {
            frankenstein::types::MaybeInaccessibleMessage::Message(msg) => msg.chat.type_field,
            frankenstein::types::MaybeInaccessibleMessage::InaccessibleMessage(msg) => {
                msg.chat.type_field
            }
        }),
        _ => None,
    }
}

fn get_message_date(u: &Update) -> Option<u64> {
    match &u.content {
        UpdateContent::Message(m) => Some(m.date),
        UpdateContent::EditedMessage(m) => Some(m.date),
        UpdateContent::ChannelPost(m) => Some(m.date),
        UpdateContent::EditedChannelPost(m) => Some(m.date),
        UpdateContent::CallbackQuery(c) => c.message.as_ref().map(|m| match m {
            frankenstein::types::MaybeInaccessibleMessage::Message(msg) => msg.date,
            frankenstein::types::MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.date,
        }),
        _ => None,
    }
}
