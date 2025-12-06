//! Command argument parsing utilities.
//!
//! Inspired by [teloxide](https://github.com/teloxide/teloxide)'s BotCommands derive macro.
//!
//! Since proc-macros have limitations in Cloudflare Workers, this module provides
//! runtime parsing utilities instead.
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::command::{CommandParser, ParseError};
//!
//! // Parse "/remind 30 minutes Buy milk"
//! let parser = CommandParser::new("/remind 30 minutes Buy milk");
//! let duration: u32 = parser.arg(0)?;
//! let unit: String = parser.arg(1)?;
//! let text: String = parser.rest(2)?;
//! ```

use std::str::FromStr;

/// Error type for command parsing.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Command not found in message.
    NotACommand,
    /// Missing required argument at index.
    MissingArg(usize),
    /// Failed to parse argument.
    InvalidArg {
        index: usize,
        expected: &'static str,
        got: String,
    },
    /// Custom parse error.
    Custom(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotACommand => write!(f, "Not a command"),
            ParseError::MissingArg(i) => write!(f, "Missing argument at position {}", i),
            ParseError::InvalidArg {
                index,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Invalid argument at {}: expected {}, got '{}'",
                    index, expected, got
                )
            }
            ParseError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Command parser for extracting arguments from command messages.
///
/// # Example
/// ```ignore
/// let parser = CommandParser::new("/ban 123 1h spam");
/// assert_eq!(parser.command(), Some("ban"));
/// let user_id: u64 = parser.arg(0)?;    // 123
/// let duration: String = parser.arg(1)?; // "1h"
/// let reason: String = parser.rest(2)?;  // "spam"
/// ```
#[derive(Debug, Clone)]
pub struct CommandParser {
    command: Option<String>,
    args: Vec<String>,
    raw: String,
}

impl CommandParser {
    /// Create a new parser from message text.
    pub fn new(text: &str) -> Self {
        let text = text.trim();
        let mut parts = text.split_whitespace();

        let first = parts.next().unwrap_or("");
        let (command, args) = if first.starts_with('/') {
            // Extract command name (without '/' and bot username)
            let cmd = first
                .strip_prefix('/')
                .unwrap_or("")
                .split('@')
                .next()
                .unwrap_or("")
                .to_string();
            (Some(cmd), parts.map(String::from).collect())
        } else {
            (None, Vec::new())
        };

        Self {
            command,
            args,
            raw: text.to_string(),
        }
    }

    /// Get the command name (without '/').
    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }

    /// Check if this is a specific command.
    pub fn is_command(&self, cmd: &str) -> bool {
        self.command.as_deref() == Some(cmd)
    }

    /// Get the number of arguments.
    pub fn arg_count(&self) -> usize {
        self.args.len()
    }

    /// Get raw argument string at index.
    pub fn arg_str(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    /// Parse argument at index.
    pub fn arg<T: FromStr>(&self, index: usize) -> Result<T, ParseError> {
        let s = self.args.get(index).ok_or(ParseError::MissingArg(index))?;
        s.parse().map_err(|_| ParseError::InvalidArg {
            index,
            expected: std::any::type_name::<T>(),
            got: s.clone(),
        })
    }

    /// Parse optional argument at index.
    pub fn arg_opt<T: FromStr>(&self, index: usize) -> Result<Option<T>, ParseError> {
        match self.args.get(index) {
            Some(s) => s.parse().map(Some).map_err(|_| ParseError::InvalidArg {
                index,
                expected: std::any::type_name::<T>(),
                got: s.clone(),
            }),
            None => Ok(None),
        }
    }

    /// Get the rest of arguments from index as a single string.
    pub fn rest(&self, from_index: usize) -> Option<String> {
        if from_index >= self.args.len() {
            return None;
        }
        Some(self.args[from_index..].join(" "))
    }

    /// Get all arguments as strings.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Get raw message text.
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

/// Parse duration string like "30m", "1h", "2d" into seconds.
///
/// Supported units:
/// - `s` - seconds
/// - `m` - minutes
/// - `h` - hours
/// - `d` - days
/// - `w` - weeks
///
/// # Example
/// ```ignore
/// assert_eq!(parse_duration("30m"), Ok(1800));
/// assert_eq!(parse_duration("1h"), Ok(3600));
/// assert_eq!(parse_duration("1d"), Ok(86400));
/// ```
pub fn parse_duration(s: &str) -> Result<u64, ParseError> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err(ParseError::Custom("Empty duration".into()));
    }

    let (num_str, unit) = if s.ends_with(char::is_alphabetic) {
        let split_pos = s.len() - 1;
        (&s[..split_pos], &s[split_pos..])
    } else {
        (s.as_str(), "s") // default to seconds
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| ParseError::Custom(format!("Invalid number in duration: {}", num_str)))?;

    let multiplier = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        "w" => 604800,
        _ => {
            return Err(ParseError::Custom(format!(
                "Unknown duration unit: {}",
                unit
            )));
        }
    };

    Ok(num * multiplier)
}

/// Split command arguments by a custom separator.
///
/// # Example
/// ```ignore
/// let args = split_args("arg1|arg2|arg3", "|");
/// assert_eq!(args, vec!["arg1", "arg2", "arg3"]);
/// ```
pub fn split_args<'a>(text: &'a str, separator: &str) -> Vec<&'a str> {
    text.split(separator)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parser() {
        let parser = CommandParser::new("/ban 123 1h spam reason");
        assert_eq!(parser.command(), Some("ban"));
        assert_eq!(parser.arg::<u64>(0).unwrap(), 123);
        assert_eq!(parser.arg::<String>(1).unwrap(), "1h");
        assert_eq!(parser.rest(2), Some("spam reason".to_string()));
    }

    #[test]
    fn test_command_with_bot_username() {
        let parser = CommandParser::new("/start@my_bot hello");
        assert_eq!(parser.command(), Some("start"));
        assert_eq!(parser.arg::<String>(0).unwrap(), "hello");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), 30);
        assert_eq!(parse_duration("30m").unwrap(), 1800);
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("1d").unwrap(), 86400);
        assert_eq!(parse_duration("1w").unwrap(), 604800);
        assert_eq!(parse_duration("30").unwrap(), 30); // default seconds
    }

    #[test]
    fn test_not_a_command() {
        let parser = CommandParser::new("hello world");
        assert_eq!(parser.command(), None);
    }
}
