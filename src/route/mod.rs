use crate::*;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Json;
use controller::settings::*;
use frankenstein::Message;
use serde_json::json;
use crate::state::AppState;

pub async fn axum_router(env: Env) -> axum::Router {
    let state = AppState::new(env);
    let mut router = axum::Router::new()
        .route("/", get(root))
        .route("/telegramMessage", post(telegram_message));
    if state.is_test() {
        router = router.nest("/telegramApi", telegram_api_router());
    }
    router.with_state(state)
}

pub fn telegram_api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/getWebhookInfo", get(get_webhook_info))
        .route("/setWebhook", post(set_webhook))
        .route("/deleteWebhook", post(delete_webhook))
        //setMyCommands
        .route("/setMyCommands", post(set_my_commands))
}

pub async fn telegram_message(
    State(_env): State<AppState>,
    Json(message): Json<Message>,
) -> impl axum::response::IntoResponse {
    let chat_id = message.chat.id;
    let text = message.text.unwrap_or_default();

    // 构建 JSON 响应
    let response_body = json!({
        "status": "success",
        "message": format!("Hello, {}", text),
        "chat_id": chat_id
    });

    // 返回 JSON 响应
    (StatusCode::OK, Json(response_body))
}

pub async fn root() -> &'static str {
    "Bot is running!"
}
