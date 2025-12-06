//! Error types for the bot framework.

use thiserror::Error;

/// Main error type for bot operations.
#[derive(Error, Debug)]
pub enum BotError {
    /// Telegram API error
    #[error("Telegram API error: {0}")]
    TelegramApi(String),

    /// Worker/Cloudflare error
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),

    /// Session storage error
    #[error("Session error: {0}")]
    Session(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    /// Custom error message
    #[error("{0}")]
    Custom(String),
}

impl From<frankenstein::Error> for BotError {
    fn from(e: frankenstein::Error) -> Self {
        BotError::TelegramApi(e.to_string())
    }
}

impl From<BotError> for worker::Error {
    fn from(e: BotError) -> Self {
        worker::Error::RustError(e.to_string())
    }
}

/// Result type alias using BotError
pub type BotResult<T> = Result<T, BotError>;
