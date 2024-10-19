use crate::*;
use controller::settings::set_webhook;
use worker::*;

pub async fn route(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", root)
        .post_async("/set_webhook", set_webhook)
        .run(_req, _env)
        .await
}
pub async fn root(_req: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok("Bot is running!")
}
