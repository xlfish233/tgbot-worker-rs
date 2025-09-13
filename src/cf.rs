use worker::{Env, Result, d1::D1Database, kv::KvStore};

pub fn kv(env: &Env, name: &str) -> Result<KvStore> {
    env.kv(name)
}

pub fn d1(env: &Env, name: &str) -> Result<D1Database> {
    env.d1(name)
}
