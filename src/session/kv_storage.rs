use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use worker::{Env, Error as WorkerError, kv::KvStore};

use super::storage::SessionStorage;

/// KV-based session storage (eventual consistency)
#[derive(Clone)]
pub struct KvStorage {
    kv: KvStore,
    prefix: String,
}

impl KvStorage {
    pub fn new(kv: KvStore, prefix: impl Into<String>) -> Self {
        Self {
            kv,
            prefix: prefix.into(),
        }
    }

    pub fn from_env(
        env: &Env,
        binding: &str,
        prefix: impl Into<String>,
    ) -> Result<Self, WorkerError> {
        Ok(Self::new(env.kv(binding)?, prefix))
    }

    fn full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.prefix, key)
        }
    }
}

#[async_trait(?Send)]
impl SessionStorage for KvStorage {
    type Error = WorkerError;

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        self.kv
            .get(&self.full_key(key))
            .json::<T>()
            .await
            .map_err(|e| WorkerError::RustError(format!("{:?}", e)))
    }

    async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<u64>,
    ) -> Result<(), Self::Error> {
        let json =
            serde_json::to_string(value).map_err(|e| WorkerError::RustError(e.to_string()))?;
        let mut put = self.kv.put(&self.full_key(key), json)?;
        if let Some(ttl) = ttl {
            put = put.expiration_ttl(ttl);
        }
        put.execute()
            .await
            .map(|_| ())
            .map_err(|e| WorkerError::RustError(format!("{:?}", e)))
    }

    async fn delete(&self, key: &str) -> Result<(), Self::Error> {
        self.kv
            .delete(&self.full_key(key))
            .await
            .map_err(|e| WorkerError::RustError(format!("{:?}", e)))
    }
}
