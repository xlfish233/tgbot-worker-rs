use futures_util::future::FutureExt;
use tgbot_worker_rs::frankenstein::{AsyncApi, AsyncTelegramApi, SendMessageParams, UpdateContent};
use tgbot_worker_rs::{App, UpdateHandlerFn};
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();

    // Plugin-style: register an update handler
    let handler: UpdateHandlerFn = std::rc::Rc::new(move |update, env: Env| {
        async move {
            let api_key = match env.secret("API_KEY") {
                Ok(secret) => secret.to_string(),
                Err(_) => return Response::error("API_KEY not found", 500).map(Some),
            };

            if let UpdateContent::Message(message) = update.content {
                if let Some(text) = message.text {
                    if text == "/version" {
                        let tg_api = AsyncApi::new(&api_key);
                        let response = format!(
                            "tgbot-worker-rs version: {}",
                            env!("CARGO_PKG_VERSION")
                        );
                        let reply = SendMessageParams::builder()
                            .chat_id(message.chat.id)
                            .text(response)
                            .build();
                        // Best-effort send; acknowledge regardless
                        if let Err(e) = tg_api.send_message(&reply).await {
                            console_error!("Error sending message: {}", e);
                        }
                        return Response::ok("").map(Some);
                    }
                }
            }

            Ok(None)
        }
        .boxed_local()
    });

    app.on_update(handler);
    app.on_fetch(req, env.clone(), ctx)
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}
