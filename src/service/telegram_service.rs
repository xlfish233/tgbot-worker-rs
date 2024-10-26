use anyhow::anyhow;
use frankenstein::{
    AsyncTelegramApi, DeleteWebhookParams, MethodResponse, SetMyCommandsParams, SetWebhookParams,
    WebhookInfo,
};

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
    // define_telegram_method!(get_webhook_info, get_webhook_info);
    define_telegram_method!(set_my_commands, set_my_commands, SetMyCommandsParams);
    pub async fn get_webhook_info(env: &Env) -> AnyhowResult<WebhookInfo> {
        let api = get_cli_from_env(env).context("Failed to get telegram api")?;
        //print api uri
        console_log!("{}", api.api_url);
        let result = api.get_webhook_info().await?;
        if result.ok {
            Ok(result.result)
        } else {
            Err(anyhow!("Failed to get webhook info"))
        }
    }
}
