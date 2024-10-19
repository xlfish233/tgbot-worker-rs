mod route;

mod extractor;

mod controller;
mod wrapped_response;
mod service;

pub use self::extractor::*;
pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;
pub use wrapped_response::wrapped_response;
pub use serde::{Deserialize, Serialize};


use worker::*;
// entrance of the worker
#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    route::route(req, env, ctx).await
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, _env: Env,_ctx:ScheduleContext)  {
    console_log!("Scheduled event: {:?}", event);
}
