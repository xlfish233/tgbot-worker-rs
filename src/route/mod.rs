use crate::controller::set_webhook;
use worker::*;

pub async fn route(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/set_webhook", set_webhook)
        .run(_req, _env)
        .await
}
