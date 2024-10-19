use crate::*;
use controller::settings::*;
use worker::*;

pub async fn route(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", root)
        .post_async("/set_webhook", set_webhook)
        .get_async("/telegramApi", telegram_get_api)
        .post_async("/telegramApi", telegram_post_api)
        .run(req, env)
        .await
}
pub async fn telegram_get_api(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Router::new()
        .get_async("/getWebhookInfo", get_webhook_info)
        .run(req, ctx.env)
        .await
}
use convert_case::{Case, Casing};





pub async fn root(_req: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok("Bot is running!")
}

pub async fn telegram_post_api(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Router::new()
        .post_async("/setWebhook", set_webhook)
        .post_async("/deleteWebhook", delete_webhook)
        .post_async("/getWebhookInfo", get_webhook_info)
        .run(req, ctx.env)
        .await
}