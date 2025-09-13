# Middleware & Reply 示例

本示例演示：

- 如何使用 `app.use_middleware(...)` 挂载中间件（记录日志、条件短路）。
- 如何发送“回复消息”（`SendMessageParams` + `ReplyParameters`）。

## 可用命令

- `/reply` — 使用 `ReplyParameters` 对收到的消息进行“回复”。
- `/echo <text>` — 简单回声（非 reply 形式）。
- `/block` — 触发中间件短路，中间件直接回复并阻止后续 handler。

## 运行

1. 设置 Bot Token（作为 Secret）：
   - `cd examples/middleware`
   - `wrangler secret put API_KEY`

2. 安装 WASM 目标（必须）
   - `rustup target add wasm32-unknown-unknown`

3. 本地运行（建议）
   - `wrangler dev`
   - 访问 `http://127.0.0.1:8787/` 应返回 `Bot is running!`

4. 配置 Telegram Webhook（替换占位符）：
   - `wrangler publish`
   - `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`

## 关键代码片段

1) 中间件：日志 + 放行

```
app.use_middleware(Rc::new(|update, env, next| {
    async move {
        // 打印最小必要信息，避免敏感数据泄露
        // ...
        let out = next(update, env).await?;
        Ok(out)
    }.boxed_local()
}));
```

2) 中间件：条件短路并回复

```
app.use_middleware(Rc::new(|update, env, next| {
    async move {
        if /* 条件命中 */ { /* 调用 Telegram API 发送消息 */
            return Response::ok("").map(|resp| ControlFlow::Break(resp));
        }
        next(update, env).await
    }.boxed_local()
}));
```

3) 回复消息（reply）

```
let reply_params = ReplyParameters::builder()
    .message_id(message.message_id)
    .build();

let params = SendMessageParams::builder()
    .chat_id(message.chat.id)
    .text("This is a reply via SendMessage")
    .reply_parameters(reply_params)
    .build();
let _ = tg.send_message(&params).await;
```

注意：中间件中请避免打印完整 Update 负载或任何敏感字段（例如 Token、用户隐私信息）。

