use serde::Serialize;
use serde::de::DeserializeOwned;
use worker::{Env, Result, kv::KvStore};

/// Simple KV helper with optional namespace prefix and TTL support.
#[derive(Clone)]
pub struct KvClient {
    kv: KvStore,
    prefix: Option<String>,
}

impl KvClient {
    pub fn new(kv: KvStore) -> Self {
        Self { kv, prefix: None }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn from_env(env: &Env, binding: &str) -> Result<Self> {
        Ok(Self::new(env.kv(binding)?))
    }

    fn k(&self, key: &str) -> String {
        match &self.prefix {
            Some(p) if !p.is_empty() => format!("{}:{}", p, key),
            _ => key.to_string(),
        }
    }

    pub async fn get_text(&self, key: &str) -> Result<Option<String>> {
        self.kv
            .get(&self.k(key))
            .text()
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        self.kv
            .get(&self.k(key))
            .json::<T>()
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))
    }

    pub async fn put_text(&self, key: &str, val: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let mut put = self.kv.put(&self.k(key), val)?;
        if let Some(ttl) = ttl_seconds {
            put = put.expiration_ttl(ttl);
        }
        put.execute()
            .await
            .map(|_| ())
            .map_err(|e| worker::Error::RustError(e.to_string()))
    }

    pub async fn put_json<T: Serialize>(
        &self,
        key: &str,
        val: &T,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let json =
            serde_json::to_string(val).map_err(|e| worker::Error::RustError(e.to_string()))?;
        self.put_text(key, &json, ttl_seconds).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.kv
            .delete(&self.k(key))
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))
    }
}
