use crate::wrapped_response::wrapped_response; 

use crate::service::TelegramService;
use frankenstein::{SetWebhookParams, DeleteWebhookParams}; 
use worker::{Response, Request, RouteContext,Result}; 
use worker::console_error; 

macro_rules! handle_request {
    ($req:ident, $ctx:ident, $param_type:path, $service_call:expr, $builder:expr) => {{
        let params = $builder; // Use the provided builder expression
        let response = $service_call(&params, &$ctx.env).await;
        match response {
            Ok(resp) => Ok(Response::from_json(&resp)?), // Convert to Response
            Err(e) => {
                console_error!("Error with backtrace: {:#?}", e);
                Response::ok(wrapped_response(
                    500,
                    "Error with backtrace",
                    Some(&format!("{:#?}", e)),
                )) // Return Response directly
            }
        }
    }};
    // Overload for cases with no parameters
    ($req:ident, $ctx:ident, $service_call:expr) => {{
        let response = $service_call(&$ctx.env).await;
        match response {
            Ok(resp) => Ok(Response::from_json(&resp)?), // Convert to Response
            Err(e) => {
                console_error!("Error with backtrace: {:#?}", e);
                Response::ok(wrapped_response(
                    500,
                    "Error with backtrace",
                    Some(&format!("{:#?}", e)),
                )) // Return Response directly
            }
        }
    }};
}

pub async fn set_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let params: SetWebhookParams = req.json().await?;
    handle_request!(req, ctx, SetWebhookParams, TelegramService::set_webhook, params) 
}

pub async fn get_webhook_info(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handle_request!(req, ctx, TelegramService::get_webhook_info) 
}

pub async fn delete_webhook(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handle_request!(req, ctx, DeleteWebhookParams, TelegramService::delete_webhook, DeleteWebhookParams::builder().drop_pending_updates(true).build()) 
}
