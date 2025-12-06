#[cfg(feature = "session")]
mod do_storage;
mod kv_storage;
mod storage;

#[cfg(feature = "session")]
pub use do_storage::DurableObjectStorage;
pub use kv_storage::KvStorage;
pub use storage::{Session, SessionStorage};

// Helper functions for extracting data from Update
use frankenstein::types::MaybeInaccessibleMessage;
use frankenstein::updates::{Update, UpdateContent};

/// Extract chat_id from Update
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

/// Extract user_id from Update
pub fn extract_user_id(update: &Update) -> Option<u64> {
    match &update.content {
        UpdateContent::Message(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::EditedMessage(m) => m.from.as_ref().map(|u| u.id),
        UpdateContent::CallbackQuery(c) => Some(c.from.id),
        UpdateContent::InlineQuery(q) => Some(q.from.id),
        _ => None,
    }
}
