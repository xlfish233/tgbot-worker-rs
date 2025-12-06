//! Version example: Demonstrates the simplified teloxide-style API
//!
//! Commands:
//! - /version - Show package version
//! - /kv_set <key> <value> - Store a value in KV
//! - /kv_get <key> - Retrieve a value from KV
//! - /d1_ping - Test D1 database connection
//! - /queue_echo <text> - Echo via queue

use serde::{Deserialize, Serialize};
use tgbot_worker_rs::prelude::*;
use tgbot_worker_rs::storage::d1::D1Client;
use tgbot_worker_rs::storage::kv::KvClient;
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct QueueJob {
    chat_id: i64,
    text: String,
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();

    // /version - Simple command using new teloxide-style API
    app.command("version", |bot, msg| async move {
        let version = env!("CARGO_PKG_VERSION");
        bot.send_message(msg.chat_id(), &format!("tgbot-worker-rs version: {}", version))
            .await
    });

    // /kv_set <key> <value> - Store in KV
    app.command("kv_set", |bot, msg| async move {
        let args = msg.command_args().unwrap_or("");
        let mut parts = args.splitn(2, ' ');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        if key.is_empty() || value.is_empty() {
            return bot
                .send_message(msg.chat_id(), "Usage: /kv_set <key> <value>")
                .await;
        }

        // Note: We need env access here, so we use inner() to get raw message
        // For complex cases, consider using on_update_flow or session API
        bot.send_message(msg.chat_id(), &format!("Would set KV[{}] = {} (env not available in simple handler)", key, value))
            .await
    });

    // /kv_get <key> - Get from KV
    app.command("kv_get", |bot, msg| async move {
        let key = msg.command_args().unwrap_or("");
        if key.is_empty() {
            return bot.send_message(msg.chat_id(), "Usage: /kv_get <key>").await;
        }

        bot.send_message(msg.chat_id(), &format!("Would get KV[{}] (env not available in simple handler)", key))
            .await
    });

    // /help - Show available commands
    app.command("help", |bot, msg| async move {
        let help = r#"Available commands:
/version - Show bot version
/kv_set <key> <value> - Store a value
/kv_get <key> - Retrieve a value
/d1_ping - Test database
/help - Show this help"#;
        bot.send_message(msg.chat_id(), help).await
    });

    // Fallback for unknown messages
    app.on_message(|bot, msg| async move {
        // Skip commands (already handled above)
        if msg.text().map(|t| t.starts_with('/')).unwrap_or(false) {
            return Err(BotError::Skip);
        }

        bot.send_message(msg.chat_id(), "Use /help to see available commands")
            .await
    });

    app.run(req, env, ctx).await
}

// Queue consumer (unchanged - uses raw API)
#[event(queue)]
pub async fn queue_consumer(
    batch: worker::MessageBatch<QueueJob>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let bot = match Bot::from_env(&env) {
        Ok(b) => b,
        Err(_) => {
            console_warn!("API_KEY missing; queue messages will be dropped");
            return Ok(());
        }
    };

    for msg in batch.iter() {
        let msg = msg?;
        let chat_id = msg.body().chat_id;
        let text = msg.body().text.clone();
        match bot.send_message(chat_id, &text).await {
            Ok(_) => msg.ack(),
            Err(e) => {
                console_error!("queue send error: {}", e);
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
