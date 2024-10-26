use axum::{extract::State, response::{IntoResponse, Json}};
use axum::http::StatusCode; // 导入 StatusCode
use frankenstein::BotCommand;
use crate::service::TelegramService;
use crate::state::AppState;
use serde_json::json;
use worker::console_error;

#[worker::send]
pub async fn init(State(env): State<AppState>) -> impl IntoResponse {
    let commands = vec![
        BotCommand {
            command: "/version".to_string(),
            description: "显示版本信息".to_string(),
        },
    ];

    match TelegramService::init_commands(commands, &env).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success", "message": "Commands set successfully."}))),
        Err(e) => {
            console_error!("Error setting commands: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "error", "message": "Failed to set commands."})))
        }
    }
}
