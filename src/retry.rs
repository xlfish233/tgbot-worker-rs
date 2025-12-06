//! Retry utilities with exponential backoff for Telegram API requests.
//!
//! Inspired by [teloxide](https://github.com/teloxide/teloxide)'s auto-retry mechanism.
//!
//! In serverless environments like Cloudflare Workers, we can't use traditional
//! sleep-based retry. Instead, we provide utilities to:
//! 1. Detect rate limit errors (429 Too Many Requests)
//! 2. Calculate retry delays with exponential backoff
//! 3. Help structure retry logic in handlers
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::retry::{RetryPolicy, should_retry};
//!
//! let policy = RetryPolicy::default();
//! let mut attempt = 0;
//!
//! loop {
//!     match bot.send_message(chat_id, "Hello").await {
//!         Ok(_) => break,
//!         Err(e) if should_retry(&e) && attempt < policy.max_retries => {
//!             attempt += 1;
//!             // In serverless, we might need to reschedule via queue
//!             let delay = policy.delay_for(attempt);
//!             // ... schedule retry after delay
//!         }
//!         Err(e) => return Err(e),
//!     }
//! }
//! ```

use std::time::Duration;

/// Configuration for retry behavior.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Base delay for exponential backoff (in milliseconds).
    pub base_delay_ms: u64,
    /// Maximum delay cap (in milliseconds).
    pub max_delay_ms: u64,
    /// Jitter factor (0.0 to 1.0) to randomize delays.
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // 1 second
            max_delay_ms: 30000, // 30 seconds
            jitter: 0.1,         // 10% jitter
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy.
    pub fn new(max_retries: u32, base_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            ..Default::default()
        }
    }

    /// Set maximum delay.
    pub fn with_max_delay(mut self, max_delay_ms: u64) -> Self {
        self.max_delay_ms = max_delay_ms;
        self
    }

    /// Set jitter factor.
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given attempt (1-indexed).
    pub fn delay_for(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        // Exponential backoff: base * 2^(attempt-1)
        let exp_delay = self
            .base_delay_ms
            .saturating_mul(1u64 << (attempt - 1).min(10));
        let capped_delay = exp_delay.min(self.max_delay_ms);

        // Simple deterministic "jitter" based on attempt number
        // (Real jitter would use random, but we want to be deterministic)
        let jitter_factor = 1.0 + (self.jitter * (attempt as f64 % 3.0 - 1.0) / 2.0);
        let final_delay = (capped_delay as f64 * jitter_factor) as u64;

        Duration::from_millis(final_delay)
    }

    /// Calculate delay in seconds for a given attempt.
    pub fn delay_seconds(&self, attempt: u32) -> u64 {
        self.delay_for(attempt).as_secs()
    }
}

/// Information about a rate limit error.
#[derive(Clone, Debug)]
pub struct RateLimitInfo {
    /// Seconds to wait before retrying (from Telegram's retry_after).
    pub retry_after: u64,
    /// The error message.
    pub message: String,
}

/// Check if an error is a rate limit (429) error and extract retry info.
pub fn parse_rate_limit(error: &str) -> Option<RateLimitInfo> {
    // Telegram returns errors like:
    // "Too Many Requests: retry after 30"
    let lower = error.to_lowercase();
    if lower.contains("too many requests") || lower.contains("429") || lower.contains("retry after")
    {
        // Try to extract retry_after value
        let retry_after = lower
            .split("retry after")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(30); // Default to 30 seconds

        return Some(RateLimitInfo {
            retry_after,
            message: error.to_string(),
        });
    }
    None
}

/// Check if an error should trigger a retry.
///
/// Returns true for:
/// - Rate limit errors (429)
/// - Temporary server errors (5xx)
/// - Network/timeout errors
pub fn should_retry(error: &str) -> bool {
    let lower = error.to_lowercase();

    // Rate limit
    if lower.contains("too many requests") || lower.contains("429") {
        return true;
    }

    // Server errors
    if lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
    {
        return true;
    }

    // Network errors
    if lower.contains("timeout") || lower.contains("connection") || lower.contains("network") {
        return true;
    }

    // Telegram-specific retryable errors
    if lower.contains("retry") || lower.contains("temporarily") {
        return true;
    }

    false
}

/// Retry context for tracking retry state.
#[derive(Clone, Debug)]
pub struct RetryContext {
    policy: RetryPolicy,
    attempt: u32,
    last_error: Option<String>,
}

impl RetryContext {
    /// Create a new retry context.
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempt: 0,
            last_error: None,
        }
    }

    /// Create with default policy.
    pub fn default_policy() -> Self {
        Self::new(RetryPolicy::default())
    }

    /// Current attempt number (0 = first try).
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Check if more retries are available.
    pub fn can_retry(&self) -> bool {
        self.attempt < self.policy.max_retries
    }

    /// Record a failed attempt.
    pub fn record_failure(&mut self, error: impl Into<String>) {
        self.attempt += 1;
        self.last_error = Some(error.into());
    }

    /// Get delay for next retry.
    pub fn next_delay(&self) -> Duration {
        self.policy.delay_for(self.attempt)
    }

    /// Get delay in seconds for next retry.
    pub fn next_delay_seconds(&self) -> u64 {
        self.next_delay().as_secs()
    }

    /// Get the last error.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Reset the context for a new operation.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::new(5, 1000).with_jitter(0.0);

        assert_eq!(policy.delay_for(1).as_millis(), 1000);
        assert_eq!(policy.delay_for(2).as_millis(), 2000);
        assert_eq!(policy.delay_for(3).as_millis(), 4000);
        assert_eq!(policy.delay_for(4).as_millis(), 8000);
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy::new(10, 1000)
            .with_max_delay(5000)
            .with_jitter(0.0);

        assert_eq!(policy.delay_for(5).as_millis(), 5000); // Would be 16000, capped to 5000
        assert_eq!(policy.delay_for(10).as_millis(), 5000);
    }

    #[test]
    fn test_parse_rate_limit() {
        let info = parse_rate_limit("Too Many Requests: retry after 30").unwrap();
        assert_eq!(info.retry_after, 30);

        let info = parse_rate_limit("Error 429: retry after 60 seconds").unwrap();
        assert_eq!(info.retry_after, 60);

        assert!(parse_rate_limit("Not found").is_none());
    }

    #[test]
    fn test_should_retry() {
        assert!(should_retry("Too Many Requests: retry after 30"));
        assert!(should_retry("Error 429"));
        assert!(should_retry("Internal Server Error 500"));
        assert!(should_retry("Connection timeout"));
        assert!(!should_retry("Bad Request: invalid chat_id"));
        assert!(!should_retry("Forbidden: bot was blocked"));
    }

    #[test]
    fn test_retry_context() {
        let mut ctx = RetryContext::default_policy();

        assert!(ctx.can_retry());
        assert_eq!(ctx.attempt(), 0);

        ctx.record_failure("Error 1");
        assert!(ctx.can_retry());
        assert_eq!(ctx.attempt(), 1);

        ctx.record_failure("Error 2");
        ctx.record_failure("Error 3");
        assert!(!ctx.can_retry()); // max_retries = 3
        assert_eq!(ctx.last_error(), Some("Error 3"));
    }
}
