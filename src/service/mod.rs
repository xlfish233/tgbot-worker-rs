mod telegram_service;

pub use telegram_service::*;

use frankenstein::AsyncApi;
use worker::*;

pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;

pub fn get_cli_from_env(env: &Env) -> AnyhowResult<AsyncApi> {
    let api_key = env
        .secret("BOT_TOKEN")
        .context("BOT_TOKEN is not set")?
        .to_string();
    let cli = AsyncApi::new(&api_key);
    Ok(cli)
}
