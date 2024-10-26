use crate::controller::response_helpers::{error_response, success_response};
use crate::service::TelegramService;
use crate::state::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use frankenstein::{DeleteWebhookParams, SetMyCommandsParams, SetWebhookParams};
use worker::console_error;
macro_rules! define_telegram_function {
    ($func_name:ident, $service_func:ident, $params_type:ty) => {
        #[worker::send]
        pub async fn $func_name(
            State(env): State<AppState>,
            Json(params): Json<$params_type>,
        ) -> impl IntoResponse {
            match TelegramService::$service_func(&params, &env).await {
                Ok(resp) => success_response(resp),
                Err(e) => error_response(e),
            }
        }
    };

    ($func_name:ident, $service_func:ident) => {
        #[worker::send]
        pub async fn $func_name(State(env): State<AppState>) -> impl IntoResponse {
            match TelegramService::$service_func(&env).await {
                Ok(info) => success_response(info),
                Err(e) => {
                    console_error!("Error retrieving info: {:?}", e);
                    error_response(e)
                }
            }
        }
    };
}

define_telegram_function!(set_webhook, set_webhook, SetWebhookParams);
define_telegram_function!(get_webhook_info, get_webhook_info);
define_telegram_function!(delete_webhook, delete_webhook, DeleteWebhookParams);
define_telegram_function!(set_my_commands, set_my_commands, SetMyCommandsParams);
