use worker::*;

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    ctx: Context,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    match route(req, env, ctx).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            on_error(e).await;
            Response::error("Internal Server Error",500)
        }
    }
       
}
pub async fn route(_req: Request, _env: Env, _ctx: Context) -> Result<Response>{
    Response::ok("Running on Cloudflare Workers!")
}

pub async fn on_error(e: Error) {
    console_log!("Error: {:?}", e);
}
 
