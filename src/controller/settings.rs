use crate::service::TelegramService;
use axum::{
    debug_handler,
    extract::State,
    response::{IntoResponse, Json},
};
use frankenstein::{DeleteWebhookParams, SetWebhookParams};
use worker::{console_error, Env};
use crate::controller::response_helpers::{success_response, error_response};

// 处理函数现在直接返回实现了 IntoResponse 的类型
#[worker::send]
pub async fn set_webhook(
    State(env): State<Env>,
    Json(params): Json<SetWebhookParams>,
) -> impl IntoResponse {
    match TelegramService::set_webhook(&params, &env).await {
        Ok(resp) => success_response(resp),
        Err(e) => error_response(e),
    }
}

#[debug_handler]
#[worker::send]
pub async fn get_webhook_info(State(env): State<Env>) -> impl axum::response::IntoResponse {
    match TelegramService::get_webhook_info(&env).await {
        Ok(info) => success_response(info),
        Err(e) => {
            console_error!("Error retrieving webhook info: {:?}", e);
            error_response(e)
        }
    }
}

#[worker::send]
pub async fn delete_webhook(State(env): State<Env>) -> impl IntoResponse {
    let params = DeleteWebhookParams::builder()
        .drop_pending_updates(true)
        .build();

    match TelegramService::delete_webhook(&params, &env).await {
        Ok(resp) => success_response(resp),
        Err(e) => error_response(e),
    }
}
