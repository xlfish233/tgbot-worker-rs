mod route;

mod extractor;

mod controller;
mod service;
mod wrapped_response;

pub use self::extractor::*;
pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;
pub use serde::{Deserialize, Serialize};
pub use wrapped_response::wrapped_response;

use worker::*;
#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    route::route(req, env, ctx).await
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
}
