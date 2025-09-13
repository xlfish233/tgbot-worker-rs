pub use frankenstein;
use frankenstein::Update;
use frankenstein::UpdateContent;
use futures_util::FutureExt;
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;

use worker::*;

pub mod cf;
#[cfg(feature = "queues")]
pub mod queue;
pub mod storage;

// Backward-compatible raw request handler (used if no update handlers are set)
pub type AsyncHandlerFn = Rc<dyn Fn(Request) -> LocalBoxFuture<'static, Result<Response>>>;

// Preferred plugin-style update handler: returns Some(Response) if handled, None to continue pipeline
pub type UpdateHandlerFn =
    Rc<dyn Fn(Update, Env) -> LocalBoxFuture<'static, Result<Option<Response>>>>;

#[derive(Clone, Default)]
struct AppData {
    raw_handler: Option<AsyncHandlerFn>,
    update_handlers: Vec<UpdateHandlerFn>,
    webhook_path: String,
}

#[derive(Default, Clone)]
pub struct App {
    env: Option<Env>,
    raw_handler: Option<AsyncHandlerFn>,
    update_handlers: Vec<UpdateHandlerFn>,
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
    // Register a plugin-style update handler
    pub fn on_update(&mut self, handler: UpdateHandlerFn) {
        self.update_handlers.push(handler);
    }

    // Ergonomic helper: register an async closure/function without manual boxing
    pub fn on_update_async<F, Fut>(&mut self, f: F)
    where
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = Result<Option<Response>>> + 'static,
    {
        let wrapped: UpdateHandlerFn = Rc::new(move |u, e| f(u, e).boxed_local());
        self.update_handlers.push(wrapped);
    }

    // Conditional handler: run only when `pred(&update)` is true
    pub fn on_update_when<P, F, Fut>(&mut self, pred: P, f: F)
    where
        P: Fn(&Update) -> bool + 'static,
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = Result<Option<Response>>> + 'static,
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
        self.update_handlers.push(wrapped);
    }

    // Convenience: route a specific Telegram command (e.g., "/version")
    pub fn on_command<F, Fut>(&mut self, command: &'static str, f: F)
    where
        F: Fn(Update, Env) -> Fut + 'static,
        Fut: core::future::Future<Output = Result<Option<Response>>> + 'static,
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
                for handler in data.update_handlers.iter() {
                    if let Some(resp) = handler(update.clone(), env.clone()).await? {
                        return Ok(resp);
                    }
                }
                // Not handled by any plugin; acknowledge update
                return Response::ok("");
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
