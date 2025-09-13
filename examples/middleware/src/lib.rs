use std::rc::Rc;

use futures_util::FutureExt;
use tgbot_worker_rs::App;
use tgbot_worker_rs::frankenstein::{
    AsyncApi, AsyncTelegramApi, ReplyParameters, SendMessageParams, UpdateContent,
};
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Middleware 1: 轻量日志（不打印敏感内容）
    // 展示如何在进入 handler 前后包裹逻辑
    app.use_middleware(Rc::new(|update, env, next| {
        async move {
            if let UpdateContent::Message(msg) = &update.content
                && let Some(text) = &msg.text
            {
                // 避免打印完整负载，仅打印最小必要信息
                console_log!(
                    "[mw:log] chat={} msg_id={} text={}",
                    msg.chat.id,
                    msg.message_id,
                    text
                );
            }

            // 放行到下一环节（其它中间件/业务 handler）
            let out = next(update, env).await?;
            Ok(out)
        }
        .boxed_local()
    }));

    // Middleware 2: 条件短路
    // 当消息以 "/block" 开头时，中间件直接回复并阻止后续 handler 运行
    app.use_middleware(Rc::new(|update, env, next| {
        async move {
            if let UpdateContent::Message(msg) = &update.content
                && let Some(text) = &msg.text
                && text.trim_start().starts_with("/block")
            {
                let api_key = match env.secret("API_KEY") {
                    Ok(s) => s.to_string(),
                    Err(_) => String::new(),
                };
                if !api_key.is_empty() {
                    let tg = AsyncApi::new(&api_key);
                    let params = SendMessageParams::builder()
                        .chat_id(msg.chat.id)
                        .text("Blocked by middleware")
                        .build();
                    let _ = tg.send_message(&params).await;
                }
                return Response::ok("").map(core::ops::ControlFlow::Break);
            }
            next(update, env).await
        }
        .boxed_local()
    }));

    // 业务示例 1：/reply —— 使用 reply 参数对消息进行“回复”
    app.on_command("reply", |update, env: Env| async move {
        let api_key = match env.secret("API_KEY") {
            Ok(s) => s.to_string(),
            Err(_) => return Response::error("API_KEY not found", 500).map(Some),
        };

        if let UpdateContent::Message(message) = update.content.clone() {
            let tg = AsyncApi::new(&api_key);

            // 使用 ReplyParameters 指定要回复的 message_id（frankenstein >= 0.35）
            let reply_params = ReplyParameters::builder()
                .message_id(message.message_id)
                .build();

            let params = SendMessageParams::builder()
                .chat_id(message.chat.id)
                .text("This is a reply via SendMessage")
                .reply_parameters(reply_params)
                .build();

            if let Err(e) = tg.send_message(&params).await {
                console_error!("send_message error: {}", e);
            }
            return Response::ok("").map(Some);
        }

        Ok(None)
    });

    // 业务示例 2：/echo <text> —— 简单回声（非 reply 格式）
    app.on_command("echo", |update, env: Env| async move {
        let api_key = match env.secret("API_KEY") {
            Ok(s) => s.to_string(),
            Err(_) => return Response::error("API_KEY not found", 500).map(Some),
        };

        if let UpdateContent::Message(message) = update.content.clone()
            && let Some(text) = message.text
        {
            let payload = text
                .split_once(' ')
                .map(|(_, rest)| rest)
                .unwrap_or("")
                .to_string();

            if payload.is_empty() {
                return Response::error("Usage: /echo <text>", 400).map(Some);
            }

            let tg = AsyncApi::new(&api_key);
            let params = SendMessageParams::builder()
                .chat_id(message.chat.id)
                .text(format!("Echo: {}", payload))
                .build();
            let _ = tg.send_message(&params).await;

            return Response::ok("").map(Some);
        }

        Ok(None)
    });

    app.on_fetch(req, env, ctx)
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}
