use worker::*;

mod route;
mod error_handle;

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    ctx: Context,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    match route::route(req, env, ctx).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error_handle::on_error(e).await;
            Response::error("Internal Server Error",500)
        }
    }
       
}
