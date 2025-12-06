//! Endpoint handlers that terminate the chain.

use super::core::{Handler, HandlerResult};
use crate::{Bot, BotResult, Message};
use frankenstein::updates::{Update, UpdateContent};
use futures_util::FutureExt;
use std::future::Future;
use std::rc::Rc;

/// Create an endpoint handler for messages.
///
/// This is the terminal handler in a chain. It always returns `Break`.
///
/// # Example
/// ```ignore
/// use tgbot_worker_rs::handler::dptree;
///
/// async fn handle_message(bot: Bot, msg: Message) -> BotResult<()> {
///     bot.reply(&msg, "Hello!").await?;
///     Ok(())
/// }
///
/// let handler = dptree::filter_command("start")
///     .chain(dptree::endpoint(handle_message));
/// ```
pub fn endpoint<F, Fut>(handler: F) -> Handler<'static>
where
    F: Fn(Bot, Message) -> Fut + 'static,
    Fut: Future<Output = BotResult<()>> + 'static,
{
    let handler = Rc::new(handler);

    Handler::new(move |update: Update, _env, bot: Bot| {
        let handler = handler.clone();
        async move {
            let msg = match &update.content {
                UpdateContent::Message(m) => Message::new((**m).clone()),
                UpdateContent::EditedMessage(m) => Message::new((**m).clone()),
                UpdateContent::ChannelPost(m) => Message::new((**m).clone()),
                UpdateContent::EditedChannelPost(m) => Message::new((**m).clone()),
                _ => return HandlerResult::Skip,
            };

            match handler(bot, msg).await {
                Ok(()) => HandlerResult::ok(),
                Err(e) => {
                    worker::console_error!("Handler error: {:?}", e);
                    HandlerResult::ok()
                }
            }
        }
        .boxed_local()
    })
}

/// Create an endpoint handler for raw updates.
///
/// Use this when you need access to the full Update object.
pub fn endpoint_update<F, Fut>(handler: F) -> Handler<'static>
where
    F: Fn(Bot, Update) -> Fut + 'static,
    Fut: Future<Output = BotResult<()>> + 'static,
{
    let handler = Rc::new(handler);

    Handler::new(move |update: Update, _env, bot: Bot| {
        let handler = handler.clone();
        async move {
            match handler(bot, update).await {
                Ok(()) => HandlerResult::ok(),
                Err(e) => {
                    worker::console_error!("Handler error: {:?}", e);
                    HandlerResult::ok()
                }
            }
        }
        .boxed_local()
    })
}

/// Create an endpoint handler for callback queries.
pub fn endpoint_callback<F, Fut>(handler: F) -> Handler<'static>
where
    F: Fn(Bot, crate::CallbackQuery) -> Fut + 'static,
    Fut: Future<Output = BotResult<()>> + 'static,
{
    let handler = Rc::new(handler);

    Handler::new(move |update: Update, _env, bot: Bot| {
        let handler = handler.clone();
        async move {
            let query = match &update.content {
                UpdateContent::CallbackQuery(q) => crate::CallbackQuery::new((**q).clone()),
                _ => return HandlerResult::Skip,
            };

            match handler(bot, query).await {
                Ok(()) => HandlerResult::ok(),
                Err(e) => {
                    worker::console_error!("Callback handler error: {:?}", e);
                    HandlerResult::ok()
                }
            }
        }
        .boxed_local()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_creation() {
        async fn dummy_handler(_bot: Bot, _msg: Message) -> BotResult<()> {
            Ok(())
        }
        let _handler = endpoint(dummy_handler);
    }
}
