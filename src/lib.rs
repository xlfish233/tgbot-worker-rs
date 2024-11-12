mod controller;
mod route;
pub mod service;
mod state;

use crate::state::AppState;
pub use anyhow::anyhow;
pub use anyhow::Context as AnyhowContext;
pub use anyhow::Result as AnyhowResult;
pub use frankenstein;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use frankenstein::Update;
use tower_service::Service;
use worker::*;

type UpdateFuture = Pin<Box<dyn Future<Output=AnyhowResult<()>> + Send>>;
type UpdateHandler = Arc<dyn Fn(Update, Env) -> UpdateFuture + Send + Sync>;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn on_fetch(
        &self,
        req: HttpRequest,
        env: Env,
        _ctx: Context,
    ) -> AnyhowResult<axum::http::Response<axum::body::Body>> {
        console_error_panic_hook::set_once();
        Ok(route::axum_router(env).await.call(req).await?)
    }

    async fn on_update(&self, update: Update, env: Env) -> AnyhowResult<()> {
        if let Some(handler) = self.on_update.as_ref() {
            handler(update, env).await
        } else {
            Ok(())
        }
    }


    pub fn set_on_update<F, Fut>(&mut self, handler: F)
    where
        F: Fn(Update, Env) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=AnyhowResult<()>> + Send + 'static,
    {
        self.on_update = Some(Arc::new(move |update, env| {
            Box::pin(handler(update, env)) as UpdateFuture
        }));
    }
}
