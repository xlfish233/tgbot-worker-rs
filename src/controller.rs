use crate::service::TelegramService;
use crate::ExtractFromRequest;
use frankenstein::{AsyncTelegramApi, SetWebhookParams};
use worker::*;

pub async fn set_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let param = SetWebhookParams::extract_from_request(&mut req).await;

    if let Err(_) = param {
        return Response::error("Failed to extract parameters", 500);
    }

    let r = TelegramService::set_webhook(&param.unwrap(), &ctx.env).await;
    match r {
        Ok(true) => Response::ok("ok"),
        Ok(false) => Response::error("Webhook setup failed", 500),
        Err(e) => {
            console_error!("{:?}", e);
            Response::error("Internal Server Error", 500)
        }
    }
}
