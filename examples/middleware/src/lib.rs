//! Middleware example: Demonstrates middleware and simplified API
//!
//! Commands:
//! - /reply - Reply to the message (quote)
//! - /echo <text> - Echo the text back
//! - /block - Blocked by middleware (never reaches handler)

use std::rc::Rc;

use futures_util::FutureExt;
use tgbot_worker_rs::prelude::*;
use tgbot_worker_rs::frankenstein::updates::UpdateContent;
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Middleware 1: Logging (lightweight, no sensitive data)
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

    // Middleware 2: Block /block command
    app.use_middleware(Rc::new(|update, env, next| {
        async move {
            if let UpdateContent::Message(msg) = &update.content {
                if let Some(text) = &msg.text {
                    if text.trim_start().starts_with("/block") {
                        if let Ok(bot) = Bot::from_env(&env) {
                            let _ = bot.send_message(msg.chat.id, "Blocked by middleware!").await;
                        }
                        return Ok(core::ops::ControlFlow::Break(Response::ok("")?));
                    }
                }
            }
            next(update, env).await
        }
        .boxed_local()
    }));

    // /reply - Reply to the message (quote)
    app.command("reply", |bot, msg| async move {
        bot.reply(&msg, "This is a reply!").await
    });

    // /echo <text> - Echo the text
    app.command("echo", |bot, msg| async move {
        let text = msg.command_args().unwrap_or("");
        if text.is_empty() {
            return bot.send_message(msg.chat_id(), "Usage: /echo <text>").await;
        }
        bot.send_message(msg.chat_id(), &format!("Echo: {}", text)).await
    });

    // /help - Show commands
    app.command("help", |bot, msg| async move {
        let help = r#"Commands:
/reply - Reply to your message
/echo <text> - Echo text back
/block - Will be blocked by middleware
/help - Show this help"#;
        bot.send_message(msg.chat_id(), help).await
    });

    app.run(req, env, ctx).await
}
