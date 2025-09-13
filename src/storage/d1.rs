use worker::{Env, Result, d1::D1Database};

/// Minimal D1 helper; keep API thin to avoid locking into patterns.
pub struct D1Client {
    db: D1Database,
}

impl D1Client {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    pub fn from_env(env: &Env, binding: &str) -> Result<Self> {
        Ok(Self::new(env.d1(binding)?))
    }

    pub fn db(&self) -> &D1Database {
        &self.db
    }
}
