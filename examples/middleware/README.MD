<!-- View Chinese: README_zh.MD -->

# Middleware & Reply Example

This example demonstrates:

- How to attach middlewares via `app.use_middleware(...)` (logging, short-circuit).
- How to send a reply message using `SendMessageParams` + `ReplyParameters`.

## Commands

- `/reply` — replies to the triggering message using `ReplyParameters`.
- `/echo <text>` — simple echo (not a reply).
- `/block` — triggers middleware short-circuit; middleware replies immediately and prevents subsequent handlers.

## Run

1) Set Bot Token (as Secret)
- `cd examples/middleware`
- `wrangler secret put API_KEY`

2) WASM Target (Required)
- This project targets `wasm32-unknown-unknown`.
- Install once: `rustup target add wasm32-unknown-unknown`.

3) Run locally
- `wrangler dev`
- Visit `http://127.0.0.1:8787/` → `Bot is running!`

4) Publish and set Telegram webhook (replace placeholders)
- `wrangler publish`
- `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`

## Key snippets

1) Middleware: log then forward

```
app.use_middleware(Rc::new(|update, env, next| {
    async move {
        // log minimal info, avoid sensitive data
        let out = next(update, env).await?;
        Ok(out)
    }.boxed_local()
}));
```

2) Middleware: short-circuit and reply

```
app.use_middleware(Rc::new(|update, env, next| {
    async move {
        if /* matched */ { /* call Telegram API */
            return Response::ok("").map(ControlFlow::Break);
        }
        next(update, env).await
    }.boxed_local()
}));
```

3) Reply to a message

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

Security note: Avoid logging tokens or full update payloads in middlewares or handlers; only log minimal necessary info.
