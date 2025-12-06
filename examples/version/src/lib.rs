use core::ops::ControlFlow;
use std::rc::Rc;

use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use tgbot_worker_rs::frankenstein::client_reqwest::Bot;
use tgbot_worker_rs::frankenstein::methods::SendMessageParams;
use tgbot_worker_rs::frankenstein::updates::UpdateContent;
use tgbot_worker_rs::frankenstein::AsyncTelegramApi;
use tgbot_worker_rs::storage::d1::D1Client;
use tgbot_worker_rs::storage::kv::KvClient;
use tgbot_worker_rs::App;
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct QueueJob {
    chat_id: i64,
    text: String,
}

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
    let mut app = App::new();

    // /version - Show package version
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/version") {
                return Ok(ControlFlow::Continue(()));
            }

            let api_key = env
                .secret("API_KEY")
                .map_err(|_| Error::RustError("API_KEY not found".into()))?
                .to_string();

            if let UpdateContent::Message(message) = &update.content {
                let bot = Bot::new(&api_key);
                let response = format!("tgbot-worker-rs version: {}", env!("CARGO_PKG_VERSION"));
                let reply = SendMessageParams::builder()
                    .chat_id(message.chat.id)
                    .text(response)
                    .build();
                if let Err(e) = bot.send_message(&reply).await {
                    console_error!("Error sending message: {}", e);
                }
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    // /queue_echo <text> - Demonstrate Queue
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/queue_echo") {
                return Ok(ControlFlow::Continue(()));
            }

            if let UpdateContent::Message(message) = &update.content {
                let text = message.text.as_deref().unwrap_or("");
                let payload = text.split_once(' ').map(|x| x.1).unwrap_or("").to_string();
                if payload.is_empty() {
                    return Ok(ControlFlow::Break(Response::error(
                        "Usage: /queue_echo <text>",
                        400,
                    )?));
                }

                let queue = env
                    .queue("QUEUE")
                    .map_err(|e| Error::RustError(format!("QUEUE binding error: {}", e)))?;
                let job = QueueJob {
                    chat_id: message.chat.id,
                    text: payload,
                };
                queue
                    .send(job)
                    .await
                    .map_err(|e| Error::RustError(format!("queue send error: {}", e)))?;
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    // /kv_set <key> <value> - Demonstrate KV
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/kv_set") {
                return Ok(ControlFlow::Continue(()));
            }

            if let UpdateContent::Message(message) = &update.content {
                let text = message.text.as_deref().unwrap_or("");
                let mut parts = text.splitn(3, ' ');
                let _cmd = parts.next();
                let key = parts.next().ok_or_else(|| {
                    Error::RustError("Usage: /kv_set <key> <value>".into())
                })?;
                let val = parts.next().ok_or_else(|| {
                    Error::RustError("Usage: /kv_set <key> <value>".into())
                })?;

                let kv = KvClient::from_env(&env, "KV")
                    .map_err(|_| Error::RustError("KV binding 'KV' not found".into()))?
                    .with_prefix("demo");
                kv.put_text(key, val, None)
                    .await
                    .map_err(|e| Error::RustError(format!("KV put error: {}", e)))?;

                if let Ok(secret) = env.secret("API_KEY") {
                    let bot = Bot::new(&secret.to_string());
                    let reply = SendMessageParams::builder()
                        .chat_id(message.chat.id)
                        .text(format!("KV set ok: {}", key))
                        .build();
                    let _ = bot.send_message(&reply).await;
                }
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    // /kv_get <key> - Demonstrate KV
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/kv_get") {
                return Ok(ControlFlow::Continue(()));
            }

            if let UpdateContent::Message(message) = &update.content {
                let text = message.text.as_deref().unwrap_or("");
                let mut parts = text.splitn(2, ' ');
                let _cmd = parts.next();
                let key = parts
                    .next()
                    .ok_or_else(|| Error::RustError("Usage: /kv_get <key>".into()))?;

                let kv = KvClient::from_env(&env, "KV")
                    .map_err(|_| Error::RustError("KV binding 'KV' not found".into()))?
                    .with_prefix("demo");
                let value = kv
                    .get_text(key)
                    .await
                    .map_err(|e| Error::RustError(e.to_string()))?;

                let msg = match value {
                    Some(v) => format!("KV[{}] = {}", key, v),
                    None => format!("KV[{}] = <missing>", key),
                };

                if let Ok(secret) = env.secret("API_KEY") {
                    let bot = Bot::new(&secret.to_string());
                    let reply = SendMessageParams::builder()
                        .chat_id(message.chat.id)
                        .text(msg)
                        .build();
                    let _ = bot.send_message(&reply).await;
                }
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    // /d1_ping - Demonstrate D1
    app.on_update_flow(Rc::new(|update, env| {
        async move {
            if !is_command(&update, "/d1_ping") {
                return Ok(ControlFlow::Continue(()));
            }

            if let UpdateContent::Message(message) = &update.content {
                let db = D1Client::from_env(&env, "DB")
                    .map_err(|_| Error::RustError("D1 binding 'DB' not found".into()))?;

                let stmt = db.db().prepare("SELECT 1 as n");
                let result = stmt
                    .all()
                    .await
                    .map_err(|e| Error::RustError(format!("query error: {}", e)))?;

                let text = match result.results::<serde_json::Value>() {
                    Ok(rows) => serde_json::to_string(&rows).unwrap_or_else(|e| format!("serialize error: {}", e)),
                    Err(e) => format!("result error: {}", e),
                };

                if let Ok(secret) = env.secret("API_KEY") {
                    let bot = Bot::new(&secret.to_string());
                    let reply = SendMessageParams::builder()
                        .chat_id(message.chat.id)
                        .text(format!("D1 ping => {}", text))
                        .build();
                    let _ = bot.send_message(&reply).await;
                }
            }
            Ok(ControlFlow::Break(Response::ok("")?))
        }
        .boxed_local()
    }));

    app.on_fetch(req, env, ctx)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

// Consume queue messages and reply in background
#[event(queue)]
pub async fn queue_consumer(
    batch: worker::MessageBatch<QueueJob>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let api_key = match env.secret("API_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => {
            console_warn!("API_KEY missing; queue messages will be dropped");
            return Ok(());
        }
    };
    let bot = Bot::new(&api_key);
    for msg in batch.iter() {
        let msg = msg?;
        let chat_id = msg.body().chat_id;
        let text = msg.body().text.clone();
        let reply = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .build();
        match bot.send_message(&reply).await {
            Ok(_) => msg.ack(),
            Err(e) => {
                console_error!("queue send tg error: {}", e);
                msg.retry();
            }
        }
    }
    Ok(())
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("Scheduled event: {:?}", event);
}
