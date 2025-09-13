pub use frankenstein;
use core::ops::ControlFlow;
use frankenstein::Update;
use frankenstein::UpdateContent;
use futures_util::FutureExt;
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;

use worker::*;

pub mod cf;
#[cfg(feature = "queue")]
pub mod queue;
pub mod storage;

// Core result alias to reduce verbosity
pub type AppResult<T = ()> = Result<T>;

// HTTP request handler types
pub type RequestFuture = LocalBoxFuture<'static, AppResult<Response>>;
pub type AsyncHandlerFn = Rc<dyn Fn(Request) -> RequestFuture>;

// Legacy update handler output: None to continue, Some(Response) to short-circuit
pub type UpdateOutcome = Option<Response>;
pub type UpdateFuture = LocalBoxFuture<'static, AppResult<UpdateOutcome>>;
pub type UpdateHandlerFn = Rc<dyn Fn(Update, Env) -> UpdateFuture>;

// Preferred flow-based handler: Continue or Break(Response)
pub type Flow = ControlFlow<Response>;
pub type FlowFuture = LocalBoxFuture<'static, AppResult<Flow>>;
pub type UpdateHandler = Rc<dyn Fn(Update, Env) -> FlowFuture>;

// Middleware pipeline types
pub type NextFn = Rc<dyn Fn(Update, Env) -> FlowFuture>;
pub type MiddlewareFn = Rc<dyn Fn(Update, Env, NextFn) -> FlowFuture>;


#[derive(Clone, Default)]
struct AppData {
    raw_handler: Option<AsyncHandlerFn>,
    update_handlers: Vec<UpdateHandler>,
    middlewares: Vec<MiddlewareFn>,
    webhook_path: String,
}

#[derive(Default, Clone)]
pub struct App {
    env: Option<Env>,
    raw_handler: Option<AsyncHandlerFn>,
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
            raw_handler: self.raw_handler.clone(),
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
    // Deprecated: prefer registering update handlers via `on_update`
    pub fn set_env(&mut self, env: Env) {
        self.env = Some(env);
    }
    // Deprecated: prefer `on_update`
    pub fn set_on_update(&mut self, handler: AsyncHandlerFn) {
        self.raw_handler = Some(handler);
    }
    // Register a plugin-style update handler (legacy Option-based output)
    pub fn on_update(&mut self, handler: UpdateHandlerFn) {
        let wrapped: UpdateHandler = Rc::new(move |u, e| {
            let h = handler.clone();
            async move {
                match h(u, e).await? {
                    Some(resp) => Ok(ControlFlow::Break(resp)),
                    None => Ok(ControlFlow::Continue(())),
                }
            }
            .boxed_local()
        });
        self.update_handlers.push(wrapped);
    }

    // Register a flow-based handler directly
    pub fn on_update_flow(&mut self, handler: UpdateHandler) {
        self.update_handlers.push(handler);
    }

    // Register middleware to run before/after handlers. Can short-circuit with a Response.
    pub fn use_middleware(&mut self, mw: MiddlewareFn) {
        self.middlewares.push(mw);
    }

    // Ergonomic helper: register an async closure/function without manual boxing
    pub fn on_update_async<F, Fut>(&mut self, f: F)
    where
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = AppResult<UpdateOutcome>> + 'static,
    {
        let wrapped: UpdateHandlerFn = Rc::new(move |u, e| f(u, e).boxed_local());
        self.on_update(wrapped);
    }

    // Conditional handler: run only when `pred(&update)` is true
    pub fn on_update_when<P, F, Fut>(&mut self, pred: P, f: F)
    where
        P: Fn(&Update) -> bool + 'static,
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = AppResult<UpdateOutcome>> + 'static,
    {
        let f = Rc::new(f);
        let wrapped: UpdateHandlerFn = Rc::new(move |u, e| {
            let run = pred(&u);
            let f = f.clone();
            async move {
                if run {
                    f(u, e).await
                } else {
                    Ok(None)
                }
            }
            .boxed_local()
        });
        self.on_update(wrapped);
    }

    // Convenience: route a specific Telegram command (e.g., "/version")
    pub fn on_command<F, Fut>(&mut self, command: &'static str, f: F)
    where
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = AppResult<UpdateOutcome>> + 'static,
    {
        let cmd = if command.starts_with('/') {
            command.to_string()
        } else {
            format!("/{}", command)
        };
        self.on_update_when(
            move |u: &Update| match &u.content {
                UpdateContent::Message(m) => match &m.text {
                    Some(text) => {
                        let first = text.split_whitespace().next().unwrap_or("");
                        first == cmd
                    }
                    None => false,
                },
                _ => false,
            },
            f,
        );
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

    // Prefer plugin-style update handlers
    if !data.update_handlers.is_empty() {
        match req.json::<Update>().await {
            Ok(update) => {
                let env = ctx.env.clone();
                // Build base "next" that executes handlers in order
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
                    ControlFlow::Break(resp) => return Ok(resp),
                    ControlFlow::Continue(()) => return Response::ok(""),
                }
            }
            Err(_) => return Response::error("parse update error.", 400),
        }
    }

    // Fallback to raw request handler if present
    match data.raw_handler {
        Some(h) => h(req).await,
        None => Response::error("Update handler not set", 500),
    }
}

// Optional Plugin trait for ergonomic registration
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn setup(&self, app: &mut App);
}

// Lightweight prelude to make imports simpler for users
pub mod prelude {
    pub use crate::{
        App, AppResult, Flow, MiddlewareFn, NextFn, UpdateHandler, UpdateHandlerFn, UpdateOutcome,
    };
    pub use crate::frankenstein::{Update, UpdateContent};
    pub use worker::{Env, Request, Response, Result};
}
