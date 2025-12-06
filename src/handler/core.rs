//! Core handler types and composition methods.

use crate::Bot;
use frankenstein::updates::Update;
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;
use worker::{Env, Response};

/// Result of handler execution.
#[derive(Debug)]
pub enum HandlerResult {
    /// Continue to next handler in chain.
    Continue,
    /// Skip this branch, try next branch.
    Skip,
    /// Stop chain and return response.
    Break(Response),
}

impl HandlerResult {
    /// Create a break result with OK response.
    pub fn ok() -> Self {
        HandlerResult::Break(Response::ok("ok").unwrap())
    }

    /// Create a skip result.
    pub fn skip() -> Self {
        HandlerResult::Skip
    }

    /// Create a break result with custom response.
    pub fn response(resp: Response) -> Self {
        HandlerResult::Break(resp)
    }

    /// Check if this is a Continue result.
    pub fn is_continue(&self) -> bool {
        matches!(self, HandlerResult::Continue)
    }

    /// Check if this is a Skip result.
    pub fn is_skip(&self) -> bool {
        matches!(self, HandlerResult::Skip)
    }

    /// Check if this is a Break result.
    pub fn is_break(&self) -> bool {
        matches!(self, HandlerResult::Break(_))
    }
}

/// Handler function type (non-Send for WASM compatibility).
pub type HandlerFn<'a> = dyn Fn(Update, Env, Bot) -> LocalBoxFuture<'a, HandlerResult> + 'a;

/// A composable async handler for processing updates.
///
/// Handlers can be composed using `chain()` for sequential processing
/// and `branch()` for parallel alternatives.
pub struct Handler<'a> {
    f: Rc<HandlerFn<'a>>,
}

impl<'a> Clone for Handler<'a> {
    fn clone(&self) -> Self {
        Handler { f: self.f.clone() }
    }
}

impl<'a> Handler<'a> {
    /// Create a new handler from a function.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Update, Env, Bot) -> LocalBoxFuture<'a, HandlerResult> + 'a,
    {
        Handler { f: Rc::new(f) }
    }

    /// Execute this handler.
    pub async fn dispatch(&self, update: Update, env: Env, bot: Bot) -> HandlerResult {
        (self.f)(update, env, bot).await
    }

    /// Chain another handler after this one.
    ///
    /// - `Continue` -> execute next handler
    /// - `Skip` -> propagate skip (branch will try next)
    /// - `Break` -> stop chain
    ///
    /// # Example
    /// ```ignore
    /// let handler = dptree::filter(is_text)
    ///     .chain(dptree::endpoint(handle_text));
    /// ```
    pub fn chain(self, next: Handler<'a>) -> Handler<'a>
    where
        'a: 'static,
    {
        let current = self.f;
        let next_f = next.f;

        Handler::new(move |update: Update, env: Env, bot: Bot| {
            let current = current.clone();
            let next_f = next_f.clone();

            Box::pin(async move {
                let result = current(update.clone(), env.clone(), bot.clone()).await;
                match result {
                    HandlerResult::Continue => next_f(update, env, bot).await,
                    HandlerResult::Skip => HandlerResult::Skip,
                    HandlerResult::Break(resp) => HandlerResult::Break(resp),
                }
            })
        })
    }

    /// Add a branch handler.
    ///
    /// - `Break` -> return result immediately
    /// - `Skip` -> try the branch handler
    /// - `Continue` -> try the branch handler
    ///
    /// # Example
    /// ```ignore
    /// let handler = dptree::entry()
    ///     .branch(dptree::filter_command("start").chain(handle_start))
    ///     .branch(dptree::filter_command("help").chain(handle_help))
    ///     .branch(dptree::endpoint(handle_fallback));
    /// ```
    pub fn branch(self, alt: Handler<'a>) -> Handler<'a>
    where
        'a: 'static,
    {
        let current = self.f;
        let alt_f = alt.f;

        Handler::new(move |update: Update, env: Env, bot: Bot| {
            let current = current.clone();
            let alt_f = alt_f.clone();

            Box::pin(async move {
                let result = current(update.clone(), env.clone(), bot.clone()).await;
                match result {
                    HandlerResult::Break(resp) => HandlerResult::Break(resp),
                    HandlerResult::Skip | HandlerResult::Continue => {
                        alt_f(update, env, bot).await
                    }
                }
            })
        })
    }

    /// Map the result of this handler.
    pub fn map_result<F>(self, mapper: F) -> Handler<'a>
    where
        F: Fn(HandlerResult) -> HandlerResult + Send + Sync + 'a + Clone,
        'a: 'static,
    {
        let current = self.f;

        Handler::new(move |update: Update, env: Env, bot: Bot| {
            let current = current.clone();
            let mapper = mapper.clone();

            Box::pin(async move {
                let result = current(update, env, bot).await;
                mapper(result)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_result_ok() {
        let result = HandlerResult::ok();
        assert!(result.is_break());
    }

    #[test]
    fn test_handler_result_continue() {
        let result = HandlerResult::Continue;
        assert!(result.is_continue());
    }
}
