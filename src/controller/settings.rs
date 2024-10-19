use wrapped_response::wrapped_response;

use crate::service::TelegramService;
use crate::*;
use frankenstein::SetWebhookParams;


pub async fn set_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match SetWebhookParams::extract_from_request(&mut req).await {
        Ok(param) => {
            let response = TelegramService::set_webhook(&param, &ctx.env).await;
            match response {
                Ok(resp) => Ok(Response::from_json(&resp)?),
                Err(e) => {
                    console_error!("Error with backtrace: {:#?}", e);
                    Ok(Response::from_json(&e.to_string())?)
                },
            }
        }
        Err(e) => {
            Response::ok(wrapped_response(500, "Error with backtrace", Some(&format!("{:#?}", e))))
        }
    }
}





