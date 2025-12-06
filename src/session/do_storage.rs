//! Durable Object based session storage (strong consistency)
//!
//! This module provides `DurableObjectStorage` client that communicates with a
//! SessionDO Durable Object for strongly consistent session storage.
//!
//! Users need to define their own SessionDO in their worker code. Example:
//!
//! ```rust,ignore
//! use worker::*;
//!
//! #[durable_object]
//! pub struct SessionDO {
//!     state: State,
//! }
//!
//! impl DurableObject for SessionDO {
//!     fn new(state: State, _env: Env) -> Self {
//!         Self { state }
//!     }
//!
//!     async fn fetch(&mut self, req: Request) -> Result<Response> {
//!         let url = req.url()?;
//!         let path = url.pathname();
//!         let key = url.search_params().get("key").unwrap_or("state".into());
//!         let storage = self.state.storage();
//!
//!         match (req.method(), path.as_str()) {
//!             (Method::Get, "/get") => {
//!                 let value: Option<String> = storage.get(&key).await?;
//!                 match value {
//!                     Some(v) => Response::ok(v),
//!                     None => Response::error("not found", 404),
//!                 }
//!             }
//!             (Method::Post, "/set") => {
//!                 let mut req = req;
//!                 let body = req.text().await?;
//!                 storage.put(&key, body).await?;
//!                 Response::ok("ok")
//!             }
//!             (Method::Delete, "/delete") => {
//!                 storage.delete(&key).await?;
//!                 Response::ok("ok")
//!             }
//!             _ => Response::error("not found", 404),
//!         }
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use worker::{
    Env, Error as WorkerError, Method, Request, RequestInit,
    durable::{ObjectNamespace, Stub},
    wasm_bindgen,
};

use super::storage::SessionStorage;

/// Durable Object based session storage client (strong consistency)
///
/// This is a client that communicates with a SessionDO Durable Object.
/// Users must define their own SessionDO in their worker code.
#[derive(Clone)]
pub struct DurableObjectStorage {
    namespace: ObjectNamespace,
}

impl DurableObjectStorage {
    pub fn new(namespace: ObjectNamespace) -> Self {
        Self { namespace }
    }

    pub fn from_env(env: &Env, binding: &str) -> Result<Self, WorkerError> {
        Ok(Self::new(env.durable_object(binding)?))
    }

    fn get_stub(&self, chat_id: i64) -> Result<Stub, WorkerError> {
        let id = self.namespace.id_from_name(&chat_id.to_string())?;
        id.get_stub()
    }

    fn parse_key(key: &str) -> (i64, String) {
        let parts: Vec<&str> = key.splitn(2, ':').collect();
        if parts.len() == 2 {
            let chat_id = parts[0].parse().unwrap_or(0);
            (chat_id, parts[1].to_string())
        } else {
            let chat_id = key.parse().unwrap_or(0);
            (chat_id, "state".to_string())
        }
    }
}

#[async_trait(?Send)]
impl SessionStorage for DurableObjectStorage {
    type Error = WorkerError;

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        let (chat_id, sub_key) = Self::parse_key(key);
        let stub = self.get_stub(chat_id)?;

        let url = format!("http://session-do/get?key={}", sub_key);
        let req = Request::new(&url, Method::Get)?;
        let mut resp = stub.fetch_with_request(req).await?;

        if resp.status_code() == 404 {
            return Ok(None);
        }

        let text = resp.text().await?;
        if text.is_empty() || text == "null" {
            return Ok(None);
        }

        serde_json::from_str(&text).map_err(|e| WorkerError::RustError(e.to_string()))
    }

    async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        _ttl: Option<u64>,
    ) -> Result<(), Self::Error> {
        let (chat_id, sub_key) = Self::parse_key(key);
        let stub = self.get_stub(chat_id)?;

        let body =
            serde_json::to_string(value).map_err(|e| WorkerError::RustError(e.to_string()))?;

        let url = format!("http://session-do/set?key={}", sub_key);
        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(wasm_bindgen::JsValue::from_str(&body)));

        let req = Request::new_with_init(&url, &init)?;
        stub.fetch_with_request(req).await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Self::Error> {
        let (chat_id, sub_key) = Self::parse_key(key);
        let stub = self.get_stub(chat_id)?;

        let url = format!("http://session-do/delete?key={}", sub_key);
        let req = Request::new(&url, Method::Delete)?;
        stub.fetch_with_request(req).await?;
        Ok(())
    }
}
