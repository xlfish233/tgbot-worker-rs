use crate::plugin::command_handler::{handle_command, Command};
use crate::state::AppState;
use crate::*;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Json;
use controller::init::*;
use controller::settings::*;
use frankenstein::{Update, UpdateContent};
use serde_json::json;

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
    Json(update): Json<Update>,
) -> impl axum::response::IntoResponse {
    console_log!("telegram_message: {:?}", update);
    let message = match update.content {
        UpdateContent::Message(message) => Some(message),
        _ => None,
    };
    if let Some(m) = message {
        let chat_id = m.chat.id;
        let text = m.text.unwrap_or_default();

        if let Some(command) = Command::from_text(&text) {
            handle_command(command, chat_id, &env).await; // 调用命令处理插件
            return (
                StatusCode::OK,
                Json(json!({"status": "success", "chat_id": chat_id})),
            );
        }
    }
    (StatusCode::OK, Json(json!({"status": "success"})))
}

pub async fn root() -> &'static str {
    "Bot is running!"
}
