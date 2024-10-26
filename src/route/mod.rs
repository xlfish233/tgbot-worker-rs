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
use crate::service::TelegramService;

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

    if text == "/version" {
        // 从状态中获取版本信息
        let version_info = AppState::version(); // 使用 AppState 中的 version 方法
        let response_body = json!({
            "status": "success",
            "message": format!("Current version: {}", version_info),
            "chat_id": chat_id
        });

        // 发送回复
        let reply_params = frankenstein::SendMessageParams {
            business_connection_id: None,
            chat_id: frankenstein::ChatId::Integer(chat_id),
            message_thread_id: None,
            text: format!("Current version: {}", version_info),
            parse_mode: None,
            entities: None,
            link_preview_options: None,
            disable_notification: None,
            protect_content: None,
            message_effect_id: None,
            reply_parameters: None,
            reply_markup: None,
        };

        let _ = TelegramService::send_message(&reply_params, &env).await; // 发送消息

        return (StatusCode::OK, Json(response_body));
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
