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
        .route("/init", post(init));
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
    State(state): State<AppState>,
    Json(update): Json<Update>,
) -> impl axum::response::IntoResponse {
    console_log!("telegram_message: {:?}", update);
    let message = match update.content {
        UpdateContent::Message(ref message) => Some(message),
        _ => None,
    };
    if let Some(m) = message {
        let _chat_id = m.chat.id;
        let _text = m.text.clone().unwrap_or_default();
    }
    if let Err(e) = state.app.on_update(update, state.env.clone()).await {
        console_log!("Error handling update: {:?}", e);
    }
    (StatusCode::OK, Json(json!({"status": "success"})))
}

pub async fn root() -> &'static str {
    "Bot is running!"
}
