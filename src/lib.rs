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
use tower_service::Service;
use worker::*;
pub use wrapped_response::wrapped_response;

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(route::axum_router(env).await.call(req).await?)
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}
