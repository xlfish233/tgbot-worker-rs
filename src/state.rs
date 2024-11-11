use crate::App;
use worker::Env;

#[derive(Clone)]
pub struct AppState {
    pub env: Env,
    pub app: App,
}

impl AppState {
    pub fn new(env: Env) -> Self {
        Self {
            env,
            app: App::default(),
        }
    }
    pub fn is_test(&self) -> bool {
        if let Ok(key) = self.env.var("TEST") {
            return key.to_string() == "1";
        }
        false
    }
}
