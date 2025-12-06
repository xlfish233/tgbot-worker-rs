use core::ops::ControlFlow;
use std::rc::Rc;

use futures_util::FutureExt;
use tgbot_worker_rs::frankenstein::client_reqwest::Bot;
use tgbot_worker_rs::frankenstein::methods::SendMessageParams;
use tgbot_worker_rs::frankenstein::types::ReplyParameters;
use tgbot_worker_rs::frankenstein::updates::UpdateContent;
use tgbot_worker_rs::frankenstein::AsyncTelegramApi;
use tgbot_worker_rs::App;
use worker::*;

/// Helper to check if update is a specific command
fn is_command(update: &tgbot_worker_rs::frankenstein::updates::Update, cmd: &str) -> bool {
    if let UpdateContent::Message(m) = &update.content {
        if let Some(text) = &m.text {
            return text.split_whitespace().next().unwrap_or("") == cmd;
        }
    }
    false
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Middleware 1: Lightweight logging (no sensitive data)
    app.use_middleware(Rc::new(|update, env, next| {
        async move {
            if let UpdateContent::Message(msg) = &update.content {
                if let Some(text) = &msg.text {
                    console_log!(
                        "[mw:log] chat={} msg_id={} text={}",
                        msg.chat.id,
                        msg.message_id,
                        text
                    );
                }
            }
            next(update, env).await
        }
        .boxed_local()
    }));

    // Middleware 2: Conditional short-circuit
    // When message starts with "/block", middleware replies and stops further handlers
    app.use_middleware(Rc::new(|update, env, next| {
        async move {
            if let UpdateContent::Message(msg) = &update.content {
                if let Some(text) = &msg.text {
                    if text.trim_start().starts_with("/block") {
                        if let Ok(secret) = env.secret("API_KEY") {
                            let bot = Bot::new(&secret.to_string());
                            let params = SendMessageParams::builder()
                                .chat_id(msg.chat.id)
                                .text("Blocked by middleware")
                                .build();
                            let _ = bot.send_message(&params).await;
                        }
                        return Ok(ControlFlow::Break(Response::ok("")?));
                    }
                }
            }
            next(update, env).await
        }
        .boxed_local()
    }));

    // /reply - Demonstrate reply with ReplyParameters
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/reply") {
                return Ok(ControlFlow::Continue(()));
            }

            let api_key = env
                .secret("API_KEY")
                .map_err(|_| Error::RustError("API_KEY not found".into()))?
                .to_string();

            if let UpdateContent::Message(message) = &update.content {
                let bot = Bot::new(&api_key);

                let reply_params = ReplyParameters::builder()
                    .message_id(message.message_id)
                    .build();

                let params = SendMessageParams::builder()
                    .chat_id(message.chat.id)
                    .text("This is a reply via SendMessage")
                    .reply_parameters(reply_params)
                    .build();

                if let Err(e) = bot.send_message(&params).await {
                    console_error!("send_message error: {}", e);
                }
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    // /echo <text> - Simple echo (not a reply format)
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/echo") {
                return Ok(ControlFlow::Continue(()));
            }

            let api_key = env
                .secret("API_KEY")
                .map_err(|_| Error::RustError("API_KEY not found".into()))?
                .to_string();

            if let UpdateContent::Message(message) = &update.content {
                let text = message.text.as_deref().unwrap_or("");
                let payload = text.split_once(' ').map(|(_, rest)| rest).unwrap_or("");

                if payload.is_empty() {
                    return Ok(ControlFlow::Break(Response::error(
                        "Usage: /echo <text>",
                        400,
                    )?));
                }

                let bot = Bot::new(&api_key);
                let params = SendMessageParams::builder()
                    .chat_id(message.chat.id)
                    .text(format!("Echo: {}", payload))
                    .build();
                let _ = bot.send_message(&params).await;
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    app.on_fetch(req, env, ctx)
        .await
        .map_err(|e| Error::from(e.to_string()))
}
