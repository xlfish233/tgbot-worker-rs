use crate::service::TelegramService;
use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use frankenstein::{DeleteWebhookParams, SetWebhookParams};
use serde_json::json;
use worker::{console_error, console_log, Env};

// 定义错误处理
#[derive(Debug)]
pub enum ApiError {
    ServiceError(String),
    JsonError(String),
}

// 实现 IntoResponse
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::ServiceError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::JsonError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = json!({
            "status": "error",
            "code": status.as_u16(),
            "message": "Error occurred",
            "data": error_message,
        });

        (status, Json(body)).into_response()
    }
}

// 处理函数现在直接返回实现了 IntoResponse 的类型
#[worker::send]
pub async fn set_webhook(
    State(env): State<Env>,
    Json(params): Json<SetWebhookParams>,
) -> impl IntoResponse {
    match TelegramService::set_webhook(&params, &env).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": resp
            })),
        )
            .into_response(),
        Err(e) => {
            console_error!("Error with backtrace: {:#?}", e);
            ApiError::ServiceError(format!("{:#?}", e)).into_response()
        }
    }
}

#[debug_handler]
#[worker::send]
pub async fn get_webhook_info(State(env): State<Env>) -> impl axum::response::IntoResponse {
    match TelegramService::get_webhook_info(&env).await {
        Ok(info) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": info
            })),
        ),
        Err(e) => {
            tracing::error!("Error retrieving webhook info: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                })),
            )
        }
    }
}
#[worker::send]
pub async fn delete_webhook(State(env): State<Env>) -> impl IntoResponse {
    let params = DeleteWebhookParams::builder()
        .drop_pending_updates(true)
        .build();

    match TelegramService::delete_webhook(&params, &env).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": resp
            })),
        )
            .into_response(),
        Err(e) => {
            console_error!("Error with backtrace: {:#?}", e);
            ApiError::ServiceError(format!("{:#?}", e)).into_response()
        }
    }
}
