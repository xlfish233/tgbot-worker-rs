use crate::ExtractFromRequest;
use anyhow::Context as AnyhowContext;
use frankenstein::{AsyncApi, AsyncTelegramApi, SetWebhookParams};
use worker::*;

pub async fn route(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/set_webhook", set_webhook)
        .run(_req, _env)
        .await
}

pub async fn set_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let param = SetWebhookParams::extract_from_request(&mut req).await;

    if let Err(_) = param {
        return Response::error("Failed to extract parameters", 500);
    }

    let cli = get_cli_from_env(&ctx.env).map_err(|e| {
        console_log!("get_cli_from_env fail {:?}", e);
        worker::Error::from("Internal Server Error")
    })?;

    let r = cli.set_webhook(&param.unwrap()).await.map_err(|e| {
        console_log!("set_webhook fail {:?}", e);
        worker::Error::from("Internal Server Error")
    })?;

    if r.result {
        Response::ok("ok")
    } else {
        console_error!("{:?}", r.description);
        Response::error("Internal Server Error", 500)
    }
}

fn get_cli_from_env(env: &Env) -> anyhow::Result<AsyncApi> {
    let api_key = env.secret("BOT_SECRET_TOKEN").context("BOT_SECRET_TOKEN is not set")?.to_string();
    let cli = AsyncApi::new(&api_key);
    Ok(cli)
}
