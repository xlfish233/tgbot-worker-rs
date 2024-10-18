use frankenstein::{AsyncTelegramApi, SetWebhookParams};

use worker::*;

use super::*;
use crate::*;

pub struct TelegramService {}

impl TelegramService {
    pub async fn set_webhook(params: &SetWebhookParams, env: &Env) -> AnyhowResult<MethodResponse<bool>> {
        let api = get_cli_from_env(env).context("Failed to get telegram api")?;
        api.set_webhook(params).await.context("request fail")
    }
}
