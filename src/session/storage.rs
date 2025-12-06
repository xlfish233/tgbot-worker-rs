use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::cell::RefCell;
use std::rc::Rc;

/// Session storage backend trait
#[async_trait(?Send)]
pub trait SessionStorage: Clone {
    type Error: std::error::Error + 'static;

    /// Get session data by key
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;

    /// Set session data with optional TTL (in seconds)
    async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<u64>,
    ) -> Result<(), Self::Error>;

    /// Delete session data
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.get::<serde_json::Value>(key).await?.is_some())
    }
}

/// Session wrapper for a specific chat/user
pub struct Session<T, S: SessionStorage> {
    key: String,
    data: Rc<RefCell<T>>,
    storage: S,
    modified: Rc<RefCell<bool>>,
}

impl<T, S: SessionStorage> Clone for Session<T, S> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            data: self.data.clone(),
            storage: self.storage.clone(),
            modified: self.modified.clone(),
        }
    }
}

impl<T: Default + Serialize + DeserializeOwned + Clone, S: SessionStorage> Session<T, S> {
    /// Load session from storage
    pub async fn load(storage: S, chat_id: i64, user_id: Option<u64>) -> Result<Self, S::Error> {
        let key = match user_id {
            Some(uid) => format!("{}:{}", chat_id, uid),
            None => chat_id.to_string(),
        };
        let data = storage.get::<T>(&key).await?.unwrap_or_default();
        Ok(Self {
            key,
            data: Rc::new(RefCell::new(data)),
            storage,
            modified: Rc::new(RefCell::new(false)),
        })
    }

    /// Get session data reference
    pub fn get(&self) -> T {
        self.data.borrow().clone()
    }

    /// Update session data
    pub fn set(&self, data: T) {
        *self.data.borrow_mut() = data;
        *self.modified.borrow_mut() = true;
    }

    /// Modify session data with a closure
    pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut self.data.borrow_mut());
        *self.modified.borrow_mut() = true;
    }

    /// Save session to storage if modified
    pub async fn save(&self, ttl: Option<u64>) -> Result<(), S::Error> {
        if *self.modified.borrow() {
            let data = self.data.borrow().clone();
            self.storage.set(&self.key, &data, ttl).await?;
            *self.modified.borrow_mut() = false;
        }
        Ok(())
    }

    /// Force save session regardless of modification state
    pub async fn force_save(&self, ttl: Option<u64>) -> Result<(), S::Error> {
        let data = self.data.borrow().clone();
        self.storage.set(&self.key, &data, ttl).await?;
        *self.modified.borrow_mut() = false;
        Ok(())
    }

    /// Clear session data
    pub async fn clear(&self) -> Result<(), S::Error> {
        self.storage.delete(&self.key).await?;
        *self.data.borrow_mut() = T::default();
        *self.modified.borrow_mut() = false;
        Ok(())
    }

    /// Check if session was modified
    pub fn is_modified(&self) -> bool {
        *self.modified.borrow()
    }
}
