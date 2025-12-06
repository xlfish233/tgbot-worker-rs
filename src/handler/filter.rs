//! Filter handlers for conditional processing.

use super::core::{Handler, HandlerResult};
use frankenstein::updates::Update;
use futures_util::FutureExt;
use std::rc::Rc;

/// Create a filter handler.
///
/// - Returns `true` -> `Continue` (proceed to next in chain)
/// - Returns `false` -> `Skip` (try next branch)
///
/// # Example
/// ```ignore
/// use tgbot_worker_rs::handler::dptree;
///
/// let handler = dptree::filter(|upd: &Update| {
///     matches!(&upd.content, UpdateContent::Message(_))
/// }).chain(dptree::endpoint(handle_message));
/// ```
pub fn filter<F>(pred: F) -> Handler<'static>
where
    F: Fn(&Update) -> bool + 'static,
{
    let pred = Rc::new(pred);

    Handler::new(move |update: Update, _env, _bot| {
        let pred = pred.clone();
        async move {
            if pred(&update) {
                HandlerResult::Continue
            } else {
                HandlerResult::Skip
            }
        }
        .boxed_local()
    })
}

/// Create a filter handler with async predicate.
pub fn filter_async<F, Fut>(pred: F) -> Handler<'static>
where
    F: Fn(Update) -> Fut + 'static,
    Fut: std::future::Future<Output = bool> + 'static,
{
    let pred = Rc::new(pred);

    Handler::new(move |update: Update, _env, _bot| {
        let pred = pred.clone();
        async move {
            if pred(update).await {
                HandlerResult::Continue
            } else {
                HandlerResult::Skip
            }
        }
        .boxed_local()
    })
}

/// Create a filter_map handler that extracts data from update.
///
/// - Returns `Some(_)` -> `Continue` (proceed to next in chain)
/// - Returns `None` -> `Skip` (try next branch)
///
/// # Example
/// ```ignore
/// use tgbot_worker_rs::handler::dptree;
///
/// let handler = dptree::filter_map(|upd: &Update| {
///     match &upd.content {
///         UpdateContent::Message(msg) => Some(msg.clone()),
///         _ => None,
///     }
/// }).chain(dptree::endpoint(handle_message));
/// ```
pub fn filter_map<F, T>(mapper: F) -> Handler<'static>
where
    F: Fn(&Update) -> Option<T> + 'static,
    T: 'static,
{
    let mapper = Rc::new(mapper);

    Handler::new(move |update: Update, _env, _bot| {
        let mapper = mapper.clone();
        async move {
            if mapper(&update).is_some() {
                HandlerResult::Continue
            } else {
                HandlerResult::Skip
            }
        }
        .boxed_local()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation() {
        let _handler = filter(|_upd: &Update| true);
    }

    #[test]
    fn test_filter_map_creation() {
        let _handler = filter_map(|_upd: &Update| Some(42));
    }
}
