//! dptree-style handler composition system for serverless environments.
//!
//! Inspired by [teloxide's dptree](https://github.com/teloxide/dptree).
//!
//! # Example
//! ```ignore
//! use tgbot_worker_rs::handler::dptree;
//!
//! let handler = dptree::entry()
//!     .branch(
//!         dptree::filter(|upd: &Update| is_message(upd))
//!             .chain(dptree::endpoint(handle_message))
//!     )
//!     .branch(dptree::endpoint(handle_fallback));
//! ```

mod core;
mod endpoint;
mod filter;

pub use core::{Handler, HandlerResult};
pub use endpoint::{endpoint, endpoint_callback, endpoint_update};
pub use filter::{filter, filter_async, filter_map};

use frankenstein::updates::Update;
use futures_util::FutureExt;

/// Create an entry point handler that always continues.
pub fn entry<'a>() -> Handler<'a> {
    Handler::new(|_update, _env, _bot| async move { HandlerResult::Continue }.boxed_local())
}

/// Filter by matching a command string.
///
/// # Example
/// ```ignore
/// dptree::filter_command("start").chain(dptree::endpoint(handle_start))
/// ```
pub fn filter_command(cmd: &'static str) -> Handler<'static> {
    let cmd_with_slash = if cmd.starts_with('/') {
        cmd.to_string()
    } else {
        format!("/{}", cmd)
    };

    filter(move |update: &Update| {
        use frankenstein::updates::UpdateContent;
        match &update.content {
            UpdateContent::Message(msg) => msg
                .text
                .as_ref()
                .map(|t| t.split_whitespace().next().unwrap_or("") == cmd_with_slash)
                .unwrap_or(false),
            _ => false,
        }
    })
}

/// Filter by matching callback data.
pub fn filter_callback_data(data: &'static str) -> Handler<'static> {
    filter(move |update: &Update| {
        use frankenstein::updates::UpdateContent;
        match &update.content {
            UpdateContent::CallbackQuery(q) => q.data.as_ref().map(|d| d == data).unwrap_or(false),
            _ => false,
        }
    })
}

/// Filter by callback data prefix.
pub fn filter_callback_prefix(prefix: &'static str) -> Handler<'static> {
    filter(move |update: &Update| {
        use frankenstein::updates::UpdateContent;
        match &update.content {
            UpdateContent::CallbackQuery(q) => {
                q.data.as_ref().map(|d| d.starts_with(prefix)).unwrap_or(false)
            }
            _ => false,
        }
    })
}

/// Prelude for convenient imports
pub mod prelude {
    pub use super::{
        endpoint, endpoint_callback, endpoint_update, entry, filter, filter_async,
        filter_callback_data, filter_callback_prefix, filter_command, filter_map, Handler,
        HandlerResult,
    };
}
