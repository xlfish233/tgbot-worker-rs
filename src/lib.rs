use futures_util::future::LocalBoxFuture;
pub use frankenstein;
use std::rc::Rc;

use worker::*;

pub type AsyncHandlerFn<'a> = Rc<dyn 'a + Fn(Request) -> LocalBoxFuture<'a, Result<Response>>>;

#[derive(Default, Clone)]
pub struct App {
    env: Option<Env>,
    on_update: Option<AsyncHandlerFn<'static>>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    #[worker::send]
    pub async fn on_fetch(&self, req: Request, env: Env, _ctx: Context) -> Result<Response> {
        console_error_panic_hook::set_once();
        worker_route(req, env, self.on_update.clone()).await
    }
    pub fn set_env(&mut self, env: Env) {
        self.env = Some(env);
    }
    pub fn set_on_update(&mut self, handler: AsyncHandlerFn<'static>) {
        self.on_update = Some(handler);
    }
}

fn root<T>(_: Request, _: RouteContext<T>) -> Result<Response> {
    Response::ok("Bot is running!")
}

async fn worker_route(
    req: Request,
    env: Env,
    handler: Option<AsyncHandlerFn<'static>>,
) -> Result<Response> {
    Router::with_data(handler)
        .get("/", root)
        .post_async("/telegramMessage", telegram_message)
        .run(req, env)
        .await
}

async fn telegram_message(
    req: Request,
    ctx: RouteContext<Option<AsyncHandlerFn<'static>>>,
) -> Result<Response> {
    let handler = ctx.data;
    match handler {
        None => Response::error("Update handler not set", 500),
        Some(h) => h(req).await,
    }
}
