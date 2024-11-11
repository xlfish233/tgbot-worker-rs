mod route;

mod controller;
mod service;
mod state;

pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;
use frankenstein::objects::*;
use tower_service::Service;
use worker::*;
use crate::state::AppState;
use std::sync::Arc;

type UpdateHandler = Arc<dyn Fn(Update, Env) -> AnyhowResult<()> + Send + Sync>;

#[derive(Default)]
pub struct App {
    on_update: Option<UpdateHandler>,
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            on_update: self.on_update.clone(),
        }
    }
}

impl App {
    async fn on_fetch(&self, req: HttpRequest, env: Env, _ctx: Context) -> AnyhowResult<axum::http::Response<axum::body::Body>> {
        console_error_panic_hook::set_once();
        Ok(route::axum_router(env).await.call(req).await?)
    }

    async fn on_update(&mut self, update: Update, env: Env) -> AnyhowResult<()> {
        if let Some(handler) = self.on_update.as_ref() {
            handler(update, env)
        } else {
            Ok(())
        }
    }

    pub fn set_on_update(&mut self, handler: impl Fn(Update, Env) -> AnyhowResult<()> + Clone + Send + Sync + 'static) {
        self.on_update = Some(Arc::new(handler));
    }
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    ctx: Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    let app_state = AppState::new(env.clone());
    app_state.app.on_fetch(req, env, ctx).await.map_err(|e| worker::Error::from(e.to_string()))
}

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}
