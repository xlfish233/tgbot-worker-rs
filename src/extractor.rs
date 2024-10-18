use crate::*;
use serde::de::DeserializeOwned;
use worker::async_trait::async_trait;
use worker::*;

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
