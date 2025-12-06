//! Structured logging macros for Cloudflare Workers.
//!
//! Since Cloudflare Workers don't have traditional stdout/stderr,
//! these macros use `worker::console_log!` internally.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::logging::*;
//!
//! log_info!("Bot started");
//! log_error!("Failed to process update: {}", error);
//! log_debug!("Received update: {:?}", update);
//! ```

/// Log an info-level message.
///
/// # Example
/// ```ignore
/// log_info!("Processing command: {}", cmd);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        worker::console_log!("[INFO] {}", format!($($arg)*));
    };
}

/// Log an error-level message.
///
/// # Example
/// ```ignore
/// log_error!("Failed to send message: {}", err);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        worker::console_log!("[ERROR] {}", format!($($arg)*));
    };
}

/// Log a warning-level message.
///
/// # Example
/// ```ignore
/// log_warn!("Rate limit approaching");
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        worker::console_log!("[WARN] {}", format!($($arg)*));
    };
}

/// Log a debug-level message (only in debug builds).
///
/// # Example
/// ```ignore
/// log_debug!("Update payload: {:?}", update);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        worker::console_log!("[DEBUG] {}", format!($($arg)*));
    };
}

/// Simple metrics counter for tracking bot activity.
///
/// # Example
/// ```ignore
/// let mut metrics = Metrics::default();
/// metrics.record_update();
/// metrics.record_command("start");
/// metrics.record_error();
///
/// // Log metrics summary
/// log_info!("{}", metrics.summary());
/// ```
#[derive(Default, Clone)]
pub struct Metrics {
    pub updates_processed: u64,
    pub commands_handled: u64,
    pub callbacks_handled: u64,
    pub errors: u64,
}

impl Metrics {
    /// Create a new metrics counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an update being processed.
    pub fn record_update(&mut self) {
        self.updates_processed += 1;
    }

    /// Record a command being handled.
    pub fn record_command(&mut self, _cmd: &str) {
        self.commands_handled += 1;
    }

    /// Record a callback query being handled.
    pub fn record_callback(&mut self) {
        self.callbacks_handled += 1;
    }

    /// Record an error.
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Get a summary string.
    pub fn summary(&self) -> String {
        format!(
            "updates={} commands={} callbacks={} errors={}",
            self.updates_processed, self.commands_handled, self.callbacks_handled, self.errors
        )
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        self.updates_processed = 0;
        self.commands_handled = 0;
        self.callbacks_handled = 0;
        self.errors = 0;
    }
}
