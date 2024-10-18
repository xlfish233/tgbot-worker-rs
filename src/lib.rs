use worker::*;

mod route;

mod extractor;

mod controller;

mod service;

pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;

pub use self::extractor::*;

// entrance of the worker
#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    route::route(req, env, ctx).await
}
