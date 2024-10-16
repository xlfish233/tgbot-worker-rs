use worker::*;

pub async fn route(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Response::ok("Running on Cloudflare Workers!")
}
