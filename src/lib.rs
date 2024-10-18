mod route;

mod extractor;

mod controller;
mod service;

use std::borrow::Cow;
pub use self::extractor::*;
pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;
use worker::*;
pub use serde::{Deserialize, Serialize};
pub use frankenstein::*;

// entrance of the worker
#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    route::route(req, env, ctx).await
}


#[derive(Serialize)]
pub struct WrappedResponse<'a> {
    code: u16,
    msg: Cow<'a, str>,
    detail: Option<Cow<'a, str>>,
}

pub fn wrapped_response<'a>(code: u16, msg: &'a str, detail: Option<&'a str>) -> String {
    let response = WrappedResponse {
        code,
        msg: Cow::Borrowed(msg),
        detail: detail.map(Cow::Borrowed),
    };
    serde_json::to_string(&response).unwrap()
}