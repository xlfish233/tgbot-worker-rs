//! Dialogue (FSM) system for multi-step conversations.
//!
//! Inspired by teloxide's dialogue system, adapted for serverless environments.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::dialogue::Dialogue;
//! use tgbot_worker_rs::session::KvStorage;
//!
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
//! enum RegState {
//!     #[default]
//!     Start,
//!     AwaitingName,
//!     AwaitingEmail { name: String },
//! }
//!
//! async fn handle(bot: Bot, msg: Message, dialogue: &mut Dialogue<RegState, KvStorage>) -> BotResult<()> {
//!     match dialogue.get() {
//!         RegState::Start => {
//!             bot.send_message(msg.chat_id(), "What's your name?").await?;
//!             dialogue.update(RegState::AwaitingName);
//!         }
//!         RegState::AwaitingName => {
//!             let name = msg.text().unwrap_or("").to_string();
//!             bot.send_message(msg.chat_id(), "What's your email?").await?;
//!             dialogue.update(RegState::AwaitingEmail { name });
//!         }
//!         RegState::AwaitingEmail { name } => {
//!             let email = msg.text().unwrap_or("");
//!             bot.send_message(msg.chat_id(), &format!("Done! {} <{}>", name, email)).await?;
//!             dialogue.exit().await?;
//!         }
//!     }
//!     dialogue.save().await?;
//!     Ok(())
//! }
//! ```

use crate::session::SessionStorage;
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// Dialogue manager for FSM-based conversations.
///
/// Manages state persistence for multi-step conversation flows.
/// Generic over state type S and storage type St.
pub struct Dialogue<S, St>
where
    St: SessionStorage,
{
    storage: St,
    chat_id: i64,
    state: Rc<RefCell<Option<S>>>,
    dirty: Rc<RefCell<bool>>,
}

impl<S, St> Clone for Dialogue<S, St>
where
    St: SessionStorage,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            chat_id: self.chat_id,
            state: self.state.clone(),
            dirty: self.dirty.clone(),
        }
    }
}

impl<S, St> Dialogue<S, St>
where
    S: Default + Clone + Serialize + DeserializeOwned,
    St: SessionStorage,
{
    /// Create a new Dialogue instance.
    pub fn new(storage: St, chat_id: i64) -> Self {
        Self {
            storage,
            chat_id,
            state: Rc::new(RefCell::new(None)),
            dirty: Rc::new(RefCell::new(false)),
        }
    }

    /// Load dialogue state from storage.
    pub async fn load(&self) -> Result<(), St::Error> {
        let key = self.storage_key();
        let state: S = self.storage.get(&key).await?.unwrap_or_default();
        *self.state.borrow_mut() = Some(state);
        Ok(())
    }

    /// Get current state (panics if not loaded).
    pub fn get(&self) -> S {
        self.state
            .borrow()
            .clone()
            .expect("Dialogue not loaded. Call load() first.")
    }

    /// Get current state as reference.
    pub fn get_ref(&self) -> std::cell::Ref<'_, S> {
        std::cell::Ref::map(self.state.borrow(), |s| {
            s.as_ref().expect("Dialogue not loaded. Call load() first.")
        })
    }

    /// Update dialogue state (marks as dirty, call save() to persist).
    pub fn update(&self, state: S) {
        *self.state.borrow_mut() = Some(state);
        *self.dirty.borrow_mut() = true;
    }

    /// Save dialogue state to storage.
    pub async fn save(&self) -> Result<(), St::Error> {
        if !*self.dirty.borrow() {
            return Ok(());
        }

        let key = self.storage_key();
        // Clone state before await to avoid holding RefCell across await point
        let state_clone = self.state.borrow().clone();
        if let Some(ref s) = state_clone {
            self.storage.set(&key, s, None).await?;
        }
        *self.dirty.borrow_mut() = false;
        Ok(())
    }

    /// Exit dialogue (reset to default state and clear storage).
    pub async fn exit(&self) -> Result<(), St::Error> {
        let key = self.storage_key();
        self.storage.delete(&key).await?;
        *self.state.borrow_mut() = Some(S::default());
        *self.dirty.borrow_mut() = false;
        Ok(())
    }

    /// Reset to default state without clearing storage.
    pub fn reset(&self) {
        *self.state.borrow_mut() = Some(S::default());
        *self.dirty.borrow_mut() = true;
    }

    /// Check if state has changed and needs saving.
    pub fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }

    fn storage_key(&self) -> String {
        format!("dialogue:{}", self.chat_id)
    }
}

/// Helper trait for dialogue state matching.
pub trait DialogueState: Default + Clone + Serialize + DeserializeOwned {
    /// Check if this state matches a pattern.
    fn matches(&self, pattern: &Self) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dialogue_creation() {
        // Basic compile test
    }
}
