//! Retry utilities for Telegram API error handling.
//!
//! Provides functions to detect retryable errors and extract retry timing.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::retry::{is_retryable, get_retry_after};
//!
//! match bot.send_message(chat_id, "Hello").await {
//!     Ok(_) => { /* success */ }
//!     Err(e) => {
//!         let err = e.to_string();
//!         if is_retryable(&err) {
//!             let delay = get_retry_after(&err).unwrap_or(30);
//!             // Handle retry (e.g., via Queue delayed delivery)
//!         }
//!     }
//! }
//! ```

/// Check if an error is retryable.
///
/// Returns true for:
/// - Rate limit errors (429 Too Many Requests)
/// - Temporary server errors (5xx)
/// - Network/timeout errors
///
/// Returns false for:
/// - Client errors (4xx except 429)
/// - Permanent failures (blocked, invalid token, etc.)
pub fn is_retryable(error: &str) -> bool {
    let lower = error.to_lowercase();

    // Rate limit - always retryable
    if lower.contains("too many requests") || lower.contains("429") {
        return true;
    }

    // Server errors - usually transient
    if lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("internal server error")
        || lower.contains("bad gateway")
        || lower.contains("service unavailable")
    {
        return true;
    }

    // Network errors - transient
    if lower.contains("timeout")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("timed out")
    {
        return true;
    }

    // Telegram-specific retryable
    if lower.contains("retry") || lower.contains("temporarily") || lower.contains("flood") {
        return true;
    }

    false
}

/// Extract retry_after seconds from a rate limit error.
///
/// Telegram returns errors like: "Too Many Requests: retry after 30"
///
/// Returns `Some(seconds)` if found, `None` otherwise.
pub fn get_retry_after(error: &str) -> Option<u64> {
    let lower = error.to_lowercase();

    // Must be a rate limit error
    if !lower.contains("too many requests")
        && !lower.contains("429")
        && !lower.contains("retry after")
    {
        return None;
    }

    // Extract the number after "retry after"
    lower
        .split("retry after")
        .nth(1)
        .and_then(|s| {
            s.trim()
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse().ok())
        })
        .or(Some(30)) // Default to 30 seconds if parsing fails
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        // Retryable errors
        assert!(is_retryable("Too Many Requests: retry after 30"));
        assert!(is_retryable("Error 429"));
        assert!(is_retryable("Internal Server Error 500"));
        assert!(is_retryable("502 Bad Gateway"));
        assert!(is_retryable("Connection timeout"));
        assert!(is_retryable("Network error"));
        assert!(is_retryable("Service temporarily unavailable"));

        // Non-retryable errors
        assert!(!is_retryable("Bad Request: invalid chat_id"));
        assert!(!is_retryable("Forbidden: bot was blocked"));
        assert!(!is_retryable("Not Found: chat not found"));
        assert!(!is_retryable("Unauthorized: invalid token"));
    }

    #[test]
    fn test_get_retry_after() {
        assert_eq!(
            get_retry_after("Too Many Requests: retry after 30"),
            Some(30)
        );
        assert_eq!(
            get_retry_after("Error 429: retry after 60 seconds"),
            Some(60)
        );
        assert_eq!(get_retry_after("retry after 15"), Some(15));

        // Should return default 30 for rate limit without explicit retry_after
        assert_eq!(get_retry_after("Too Many Requests"), Some(30));
        assert_eq!(get_retry_after("429"), Some(30));

        // Non-rate-limit errors
        assert_eq!(get_retry_after("Bad Request"), None);
        assert_eq!(get_retry_after("Internal Server Error"), None);
    }
}
