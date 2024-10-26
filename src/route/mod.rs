use crate::*;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Json;
use controller::settings::*;
use controller::init::*;
use frankenstein::Message; // 导入 SetMyCommandsParams
use serde_json::json;
use crate::state::AppState;
use crate::plugin::command_handler::{Command, handle_command}; // 修正导入为逗号分隔

pub async fn axum_router(env: Env) -> axum::Router {
    let state = AppState::new(env);
    let mut router = axum::Router::new()
        .route("/", get(root))
        .route("/telegramMessage", post(telegram_message))
        .route("/init", post(init)); // 添加新的路由
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
        .route("/setMyCommands", post(set_my_commands))
}

#[worker::send]
pub async fn telegram_message(
    State(env): State<AppState>,
    Json(message): Json<Message>,
) -> impl axum::response::IntoResponse {
    let chat_id = message.chat.id;
    let text = message.text.unwrap_or_default();

    if let Some(command) = Command::from_text(&text) {
        handle_command(command, chat_id, &env).await; // 调用命令处理插件
        return (StatusCode::OK, Json(json!({"status": "success", "chat_id": chat_id})));
    }

    // 构建默认的 JSON 响应
    let response_body = json!({
        "status": "success",
        "message": format!("Hello, {}", text),
        "chat_id": chat_id
    });

    (StatusCode::OK, Json(response_body))
}

pub async fn root() -> &'static str {
    "Bot is running!"
}
