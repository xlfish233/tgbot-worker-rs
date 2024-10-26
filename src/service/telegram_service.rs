use anyhow::anyhow;
use frankenstein::{
    AsyncTelegramApi, DeleteWebhookParams, MethodResponse, SetMyCommandsParams, SetWebhookParams,
    WebhookInfo, BotCommand,
};

use worker::*;
use crate::state::AppState;
use super::*;

pub struct TelegramService {}

macro_rules! define_telegram_method {
    ($name:ident, $method:ident, $params:ty) => {
        pub async fn $name(params: &$params, state: &AppState) -> AnyhowResult<MethodResponse<bool>> {
            let api = get_cli_from_env(&state.env).context("Failed to get telegram api")?;
            api.$method(params).await.context("request fail")
        }
    };

    ($name:ident, $method:ident) => {
        pub async fn $name(state: &AppState) -> AnyhowResult<MethodResponse<WebhookInfo>> {
            let api = get_cli_from_env(&state.env).context("Failed to get telegram api")?;
            api.$method().await.context("request fail")
        }
    };
}

macro_rules! define_telegram_method_no_params {
    ($name:ident) => {
        pub async fn $name(state: &AppState) -> AnyhowResult<WebhookInfo> {
            let api = get_cli_from_env(&state.env).context("Failed to get telegram api")?;
            console_log!("{}", api.api_url);
            let result = api.get_webhook_info().await?;
            if result.ok {
                Ok(result.result)
            } else {
                Err(anyhow!("Failed to get webhook info"))
            }
        }
    };
}

impl TelegramService {
    define_telegram_method!(set_webhook, set_webhook, SetWebhookParams);
    define_telegram_method!(delete_webhook, delete_webhook, DeleteWebhookParams);
    define_telegram_method!(set_my_commands, set_my_commands, SetMyCommandsParams);
    
    // 使用宏重构的init_commands函数
    pub async fn init_commands(commands: Vec<BotCommand>, state: &AppState) -> AnyhowResult<bool> {
        let params = SetMyCommandsParams {
            commands,
            scope: None,
            language_code: None,
        };
        
        // 修复调用并返回 bool
        Self::set_my_commands(&params, state).await.map(|_| true)
    }

    // 使用宏重构的get_webhook_info函数
    define_telegram_method_no_params!(get_webhook_info);
}
