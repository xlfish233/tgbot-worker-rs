use futures_util::future::FutureExt;
use std::rc::Rc;
use tgbot_worker_rs::frankenstein::{
    AsyncApi, AsyncTelegramApi, SendMessageParams, Update, UpdateContent,
};
use tgbot_worker_rs::{App, AsyncHandlerFn};
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();
    app.set_env(env.clone());

    let env_clone = env.clone();
    let handler: AsyncHandlerFn = Rc::new(move |req: Request| {
        let env = env_clone.clone();
        handle_echo(req, env).boxed_local()
    });

    app.set_on_update(handler.clone());
    app.on_fetch(req, env.clone(), ctx)
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}

async fn handle_echo(mut req: Request, env: Env) -> Result<Response> {
    let api_key = match env.secret("API_KEY") {
        Ok(secret) => secret.to_string(),
        Err(_) => return Response::error("API_KEY not found", 500),
    };
    let tg_api = AsyncApi::new(&api_key);
    if let Ok(update) = req.json::<Update>().await {
        match update.content {
            UpdateContent::Message(message) => {
                if let Some(text) = message.text {
                    if text == "/version" {
                        let response = "tgbot-worker-rs version: 0.1.0".to_string();
                        let reply = SendMessageParams::builder()
                            .chat_id(message.chat.id)
                            .text(response)
                            .build();
                        match tg_api.send_message(&reply).await {
                            Ok(_) => {
                                console_log!("Message sent successfully.");
                            }
                            Err(e) => {
                                console_error!("Error sending message: {}", e);
                            }
                        }
                    }
                }
                Response::ok("")
            }
            _ => Response::ok(""),
        }
    } else {
        Response::error("parse update error.", 500)
    }
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}
