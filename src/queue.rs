#![cfg(feature = "queues")]
//! Queue facade (behind `queues` feature). The concrete CF queue bindings will be wired here.
use worker::{Env, Result};

pub struct QueueClient {
    env: Env,
    binding: String,
}

impl QueueClient {
    pub fn new(env: Env, binding: impl Into<String>) -> Self {
        Self {
            env,
            binding: binding.into(),
        }
    }

    pub fn from_env(env: &Env, binding: &str) -> Self {
        Self {
            env: env.clone(),
            binding: binding.to_string(),
        }
    }

    /// Send a message to the queue. Placeholder until worker crate exposes queue API.
    pub async fn send(&self, _payload: &[u8]) -> Result<()> {
        // TODO: integrate with worker's queue API when available.
        // For now, this is a no-op stub so enabling the feature compiles.
        Ok(())
    }
}
