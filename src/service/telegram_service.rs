use frankenstein::{
    AsyncTelegramApi, DeleteWebhookParams, MethodResponse, SetWebhookParams, WebhookInfo,
}; // Added WebhookInfo import

use worker::*;

use super::*;

pub struct TelegramService {}

macro_rules! define_telegram_method {
    ($name:ident, $method:ident, $params:ty) => {
        pub async fn $name(params: &$params, env: &Env) -> AnyhowResult<MethodResponse<bool>> {
            let api = get_cli_from_env(env).context("Failed to get telegram api")?;
            api.$method(params).await.context("request fail")
        }
    };

    ($name:ident, $method:ident) => {
        pub async fn $name(env: &Env) -> AnyhowResult<MethodResponse<WebhookInfo>> {
            let api = get_cli_from_env(env).context("Failed to get telegram api")?;
            api.$method().await.context("request fail")
        }
    };
}

impl TelegramService {
    define_telegram_method!(set_webhook, set_webhook, SetWebhookParams);
    define_telegram_method!(delete_webhook, delete_webhook, DeleteWebhookParams);
    define_telegram_method!(get_webhook_info, get_webhook_info);
}
