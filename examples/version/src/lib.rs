use tgbot_worker_rs::frankenstein::{AsyncApi, AsyncTelegramApi, SendMessageParams, UpdateContent};
use tgbot_worker_rs::storage::d1::D1Client;
use tgbot_worker_rs::storage::kv::KvClient;
use tgbot_worker_rs::App;
use worker::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QueueJob {
    chat_id: i64,
    text: String,
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();

    // Plugin-style: route the "/version" command
    app.on_command("version", |update, env: Env| async move {
        let api_key = match env.secret("API_KEY") {
            Ok(secret) => secret.to_string(),
            Err(_) => return Response::error("API_KEY not found", 500).map(Some),
        };

        if let UpdateContent::Message(message) = update.content
            && let Some(text) = message.text
        {
            // exact match is already handled by on_command, but we keep safe guards
            if text.split_whitespace().next().unwrap_or("") == "/version" {
                let tg_api = AsyncApi::new(&api_key);
                let response = format!(
                    "tgbot-worker-rs version: {}",
                    env!("CARGO_PKG_VERSION")
                );
                let reply = SendMessageParams::builder()
                    .chat_id(message.chat.id)
                    .text(response)
                    .build();
                if let Err(e) = tg_api.send_message(&reply).await {
                    console_error!("Error sending message: {}", e);
                }
                return Response::ok("").map(Some);
            }
        }
        Ok(None)
    });

    // Demonstrate Queue: /queue_echo <text>
    app.on_command("queue_echo", |update, env: Env| async move {
        if let UpdateContent::Message(message) = update.content.clone()
            && let Some(text) = message.text
        {
            let payload = text.split_once(' ').map(|x| x.1).unwrap_or("").to_string();
            if payload.is_empty() {
                return Response::error("Usage: /queue_echo <text>", 400).map(Some);
            }

            let queue = match env.queue("QUEUE") {
                Ok(q) => q,
                Err(e) => return Response::error(format!("QUEUE binding error: {}", e), 500).map(Some),
            };
            let job = QueueJob { chat_id: message.chat.id, text: payload };
            if let Err(e) = queue.send(job).await {
                return Response::error(format!("queue send error: {}", e), 500).map(Some);
            }
            return Response::ok("").map(Some);
        }
        Ok(None)
    });

    // Demonstrate KV: /kv_set <key> <value>
    app.on_command("kv_set", |update, env: Env| async move {
        if let UpdateContent::Message(message) = update.content.clone()
            && let Some(text) = message.text
        {
                let mut parts = text.splitn(3, ' ');
                let _cmd = parts.next(); // /kv_set
                let key = match parts.next() { Some(k) => k, None => return Response::error("Usage: /kv_set <key> <value>", 400).map(Some) };
                let val = match parts.next() { Some(v) => v, None => return Response::error("Usage: /kv_set <key> <value>", 400).map(Some) };

                let kv = match KvClient::from_env(&env, "KV") {
                    Ok(kv) => kv.with_prefix("demo"),
                    Err(_) => return Response::error("KV binding 'KV' not found", 500).map(Some),
                };
                if let Err(e) = kv.put_text(key, val, None).await {
                    return Response::error(format!("KV put error: {}", e), 500).map(Some);
                }

                let api_key = match env.secret("API_KEY") { Ok(s) => s.to_string(), Err(_) => String::new() };
                if !api_key.is_empty() {
                    let tg = AsyncApi::new(&api_key);
                    let reply = SendMessageParams::builder()
                        .chat_id(message.chat.id)
                        .text(format!("KV set ok: {}", key))
                        .build();
                    let _ = tg.send_message(&reply).await;
                }
                return Response::ok("").map(Some);
        }
        Ok(None)
    });

    // Demonstrate KV: /kv_get <key>
    app.on_command("kv_get", |update, env: Env| async move {
        if let UpdateContent::Message(message) = update.content.clone()
            && let Some(text) = message.text
        {
                let mut parts = text.splitn(2, ' ');
                let _cmd = parts.next(); // /kv_get
                let key = match parts.next() { Some(k) => k, None => return Response::error("Usage: /kv_get <key>", 400).map(Some) };

                let kv = match KvClient::from_env(&env, "KV") {
                    Ok(kv) => kv.with_prefix("demo"),
                    Err(_) => return Response::error("KV binding 'KV' not found", 500).map(Some),
                };
                let value = kv.get_text(key).await.map_err(|e| worker::Error::from(e.to_string()))?;

                let msg = match value { Some(v) => format!("KV[{}] = {}", key, v), None => format!("KV[{}] = <missing>", key) };
                let api_key = match env.secret("API_KEY") { Ok(s) => s.to_string(), Err(_) => String::new() };
                if !api_key.is_empty() {
                    let tg = AsyncApi::new(&api_key);
                    let reply = SendMessageParams::builder()
                        .chat_id(message.chat.id)
                        .text(msg)
                        .build();
                    let _ = tg.send_message(&reply).await;
                }
                return Response::ok("").map(Some);
        }
        Ok(None)
    });

    // Demonstrate D1: /d1_ping -> SELECT 1 as n
    app.on_command("d1_ping", |update, env: Env| async move {
        if let UpdateContent::Message(message) = update.content.clone() {
            let db = match D1Client::from_env(&env, "DB") {
                Ok(db) => db,
                Err(_) => return Response::error("D1 binding 'DB' not found", 500).map(Some),
            };

            // Simple query without bindings
            let stmt = db.db().prepare("SELECT 1 as n");
            let result = match stmt.all().await {
                Ok(r) => r,
                Err(e) => return Response::error(format!("query error: {}", e), 500).map(Some),
            };

            // Render result compactly as JSON text
            let text = match result.results::<serde_json::Value>() {
                Ok(rows) => match serde_json::to_string(&rows) {
                    Ok(s) => s,
                    Err(e) => format!("serialize error: {}", e),
                },
                Err(e) => format!("result error: {}", e),
            };

            let api_key = match env.secret("API_KEY") { Ok(s) => s.to_string(), Err(_) => String::new() };
            if !api_key.is_empty() {
                let tg = AsyncApi::new(&api_key);
                let reply = SendMessageParams::builder()
                    .chat_id(message.chat.id)
                    .text(format!("D1 ping => {}", text))
                    .build();
                let _ = tg.send_message(&reply).await;
            }

            return Response::ok("").map(Some);
        }
        Ok(None)
    });
    app.on_fetch(req, env.clone(), ctx)
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}

// Consume queue messages and reply in background
#[event(queue)]
pub async fn queue_consumer(batch: worker::MessageBatch<QueueJob>, env: Env, _ctx: Context) -> Result<()> {
    let api_key = match env.secret("API_KEY") { Ok(s) => s.to_string(), Err(_) => String::new() };
    if api_key.is_empty() {
        console_warn!("API_KEY missing; queue messages will be dropped");
        return Ok(());
    }
    let tg = AsyncApi::new(&api_key);
    for msg in batch.iter() {
        let msg = msg?;
        let chat_id = msg.body().chat_id;
        let text = msg.body().text.clone();
        let reply = SendMessageParams::builder().chat_id(chat_id).text(text).build();
        match tg.send_message(&reply).await {
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
