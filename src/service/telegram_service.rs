
use frankenstein::{AsyncTelegramApi, SetWebhookParams};

use worker::*;

use super::*;
use crate::*;

pub struct TelegramService {}

impl TelegramService {
    pub async fn set_webhook(params: &SetWebhookParams, env: &Env) -> AnyhowResult<bool> {
        let api = get_cli_from_env(env).context("Failed to get telegram api")?;
        let r = api.set_webhook(params).await?;
        if r.result {
            Ok(true)
        } else {
            Err(anyhow!("Failed to set webhook"))
        }
    }
}
