use core::ops::ControlFlow;
pub use frankenstein;
use frankenstein::updates::Update;
use frankenstein::updates::UpdateContent;
use futures_util::FutureExt;
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;

use worker::*;

pub mod cf;
pub mod error;
pub mod filter;
#[cfg(feature = "queue")]
pub mod queue;
pub mod session;
pub mod storage;

pub use error::{BotError, BotResult};
pub use filter::*;

// Core result alias to reduce verbosity
pub type AppResult<T = ()> = Result<T>;

// Flow-based handler types
pub type Flow = ControlFlow<Response>;
pub type FlowFuture = LocalBoxFuture<'static, AppResult<Flow>>;
pub type UpdateHandler = Rc<dyn Fn(Update, Env) -> FlowFuture>;

// Middleware pipeline types
pub type NextFn = Rc<dyn Fn(Update, Env) -> FlowFuture>;
pub type MiddlewareFn = Rc<dyn Fn(Update, Env, NextFn) -> FlowFuture>;

#[derive(Clone, Default)]
struct AppData {
    update_handlers: Vec<UpdateHandler>,
    middlewares: Vec<MiddlewareFn>,
    webhook_path: String,
}

#[derive(Default, Clone)]
pub struct App {
    update_handlers: Vec<UpdateHandler>,
    middlewares: Vec<MiddlewareFn>,
    webhook_path: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            webhook_path: "/telegramMessage".to_string(),
            ..Default::default()
        }
    }

    pub fn with_webhook_path(mut self, path: impl Into<String>) -> Self {
        self.webhook_path = path.into();
        self
    }

    fn as_data(&self) -> AppData {
        AppData {
            update_handlers: self.update_handlers.clone(),
            middlewares: self.middlewares.clone(),
            webhook_path: self.webhook_path.clone(),
        }
    }

    #[worker::send]
    pub async fn on_fetch(&self, req: Request, env: Env, _ctx: Context) -> Result<Response> {
        console_error_panic_hook::set_once();
        worker_route(req, env, self.as_data()).await
    }

    /// Register a flow-based handler directly
    pub fn on_update_flow(&mut self, handler: UpdateHandler) {
        self.update_handlers.push(handler);
    }

    /// Register middleware to run before/after handlers. Can short-circuit with a Response.
    pub fn use_middleware(&mut self, mw: MiddlewareFn) {
        self.middlewares.push(mw);
    }

    /// Register a context-based handler with session support
    pub fn on_update_ctx<T, S, F, Fut>(&mut self, storage: S, f: F)
    where
        T: Default + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
        S: session::SessionStorage + 'static,
        F: Fn(session::Context<T, S>) -> Fut + 'static,
        Fut: Future<Output = BotResult<Flow>> + 'static,
    {
        let f = Rc::new(f);
        self.on_update_flow(Rc::new(move |update, env| {
            let f = f.clone();
            let storage = storage.clone();
            async move {
                let chat_id = session::extract_chat_id(&update).unwrap_or(0);
                let user_id = session::extract_user_id(&update);
                let sess = session::Session::load(storage, chat_id, user_id)
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;
                let ctx = session::Context::new(update, env, sess);
                let result = f(ctx.clone()).await.map_err(|e| e.into());
                ctx.session
                    .save(Some(3600))
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;
                result
            }
            .boxed_local()
        }));
    }

    /// Register a context-based command handler with session support
    pub fn on_command_ctx<T, S, F, Fut>(&mut self, command: &'static str, storage: S, f: F)
    where
        T: Default + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
        S: session::SessionStorage + 'static,
        F: Fn(session::Context<T, S>) -> Fut + 'static,
        Fut: Future<Output = BotResult<Flow>> + 'static,
    {
        let cmd = if command.starts_with('/') {
            command.to_string()
        } else {
            format!("/{}", command)
        };
        let f = Rc::new(f);
        self.on_update_flow(Rc::new(move |update, env| {
            let f = f.clone();
            let storage = storage.clone();
            let cmd = cmd.clone();
            async move {
                // Check if this is the target command
                let is_match = match &update.content {
                    UpdateContent::Message(m) => match &m.text {
                        Some(text) => text.split_whitespace().next().unwrap_or("") == cmd,
                        None => false,
                    },
                    _ => false,
                };

                if !is_match {
                    return Ok(ControlFlow::Continue(()));
                }

                let chat_id = session::extract_chat_id(&update).unwrap_or(0);
                let user_id = session::extract_user_id(&update);
                let sess = session::Session::load(storage, chat_id, user_id)
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;
                let ctx = session::Context::new(update, env, sess);
                let result = f(ctx.clone()).await.map_err(|e| e.into());
                ctx.session
                    .save(Some(3600))
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;
                result
            }
            .boxed_local()
        }));
    }
}

fn root<T>(_: Request, _: RouteContext<T>) -> Result<Response> {
    Response::ok("Bot is running!")
}

async fn worker_route(req: Request, env: Env, data: AppData) -> Result<Response> {
    let path = data.webhook_path.clone();
    Router::with_data(data)
        .get("/", root)
        .post_async(&path, telegram_message)
        .run(req, env)
        .await
}

async fn telegram_message(mut req: Request, ctx: RouteContext<AppData>) -> Result<Response> {
    let data = ctx.data;

    match req.json::<Update>().await {
        Ok(update) => {
            let env = ctx.env.clone();
            // Build base "next" that executes handlers in order (may be empty)
            let handlers = data.update_handlers.clone();
            let base_next: NextFn = Rc::new(move |u, e| {
                let handlers = handlers.clone();
                async move {
                    for h in handlers.iter() {
                        match h(u.clone(), e.clone()).await? {
                            ControlFlow::Break(resp) => return Ok(ControlFlow::Break(resp)),
                            ControlFlow::Continue(()) => (),
                        }
                    }
                    Ok(ControlFlow::Continue(()))
                }
                .boxed_local()
            });

            // Wrap with middlewares in reverse order
            let mut next = base_next;
            for mw in data.middlewares.iter().cloned().rev() {
                let prev = next.clone();
                next = Rc::new(move |u, e| mw(u, e, prev.clone()));
            }

            // Execute pipeline
            match next(update, env).await? {
                ControlFlow::Break(resp) => Ok(resp),
                ControlFlow::Continue(()) => Response::ok(""),
            }
        }
        Err(_) => Response::error("parse update error.", 400),
    }
}

// Optional Plugin trait for ergonomic registration
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn setup(&self, app: &mut App);
}

// Lightweight prelude to make imports simpler for users
pub mod prelude {
    pub use crate::error::{BotError, BotResult};
    pub use crate::filter;
    pub use crate::frankenstein::updates::{Update, UpdateContent};
    #[cfg(feature = "session")]
    pub use crate::session::DurableObjectStorage;
    pub use crate::session::{Context, KvStorage, Session, SessionStorage};
    pub use crate::{App, AppResult, Flow, MiddlewareFn, NextFn, UpdateHandler};
    pub use worker::{Env, Request, Response, Result};
}
