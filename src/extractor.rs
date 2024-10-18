use anyhow::Context;

use serde::de::DeserializeOwned;
use serde_json;
use worker::async_trait::async_trait;
use worker::*;
use frankenstein::SetWebhookParams;

#[async_trait(?Send)]
pub trait ExtractFromRequest {
     async fn extract_from_request(req: &mut Request) -> anyhow::Result<Self>
    where
        Self: Sized + DeserializeOwned;
}

#[async_trait(?Send)]
impl<T> ExtractFromRequest for T
where
    T: DeserializeOwned,
{
     async fn extract_from_request(req: &mut Request) -> anyhow::Result<Self> {
        req.json().await.context("failed to parse request body")
    }
}

trait ExtractFromEnv {
    fn extract_from_env(env: &Env) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl ExtractFromEnv for SetWebhookParams {
    fn extract_from_env(env: &Env) -> anyhow::Result<Self> {
        let url = env
            .var("WORKER_URL")
            .context("WORKER_URL is not set")?
            .to_string(); // Convert Secret to String
        let ip_address = env.var("IP_ADDRESS").ok().map(|s| s.to_string()); // Convert Secret to String
        let max_connections = env
            .var("MAX_CONNECTIONS")
            .ok()
            .map(|s| s.to_string().parse::<u32>().ok())
            .flatten();
        let allowed_updates = env
            .var("ALLOWED_UPDATES")
            .ok()
            .map(|s| serde_json::from_str(&s.to_string()).ok())
            .flatten(); // Assuming JSON format
        let drop_pending_updates = env
            .var("DROP_PENDING_UPDATES")
            .ok()
            .map(|s| s.to_string().parse::<bool>().ok())
            .flatten();
        let secret_token = env.var("SECRET_TOKEN").ok().map(|s| s.to_string()); // Convert Secret to String

        Ok(SetWebhookParams {
            url,
            certificate: None, // Omit certificate handling
            ip_address,
            max_connections,
            allowed_updates,
            drop_pending_updates,
            secret_token,
        })
    }
}
