# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.3.0-orange)

A lightweight, serverless Telegram bot framework for Cloudflare Workers, built with Rust.

[查看中文说明](README_zh.md)

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [API Overview](#api-overview)
- [Session Management](#session-management)
- [Filter Combinators](#filter-combinators)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Serverless-first**: Designed for Cloudflare Workers with zero cold-start overhead
- **Session Management**: Built-in KV-based session storage with type-safe state
- **Simplified API**: `Context::done()`, `Context::skip()`, `reply_and_done()` for clean handler code
- **Filter Combinators**: Composable filters like `is_message`, `text_contains`, `callback_data_equals`
- **Rich Telegram Methods**: `reply`, `reply_html`, `edit_text`, `delete_message`, `answer_callback`, `send_photo`
- **Middleware Support**: Request/response pipeline with short-circuit capability
- **Type Safety**: Leverages Rust's type system with `BotError` and `BotResult`

**Project Status:** Active development. Contributions welcome!

Note: This project targets `wasm32-unknown-unknown` (pinned via `.cargo/config.toml`). Install the target with `rustup target add wasm32-unknown-unknown` and prefer running commands with the pinned toolchain (`+1.89.0`).

## Quick Start

```rust
use serde::{Deserialize, Serialize};
use tgbot_worker_rs::prelude::*;
use tgbot_worker_rs::session::{Context, KvStorage};
use worker::*;

#[derive(Default, Clone, Serialize, Deserialize)]
struct MyState {
    counter: u32,
}

type Ctx = Context<MyState, KvStorage>;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    let mut app = App::new();
    let storage = KvStorage::from_env(&env, "SESSION_KV", "session")?;

    // Simple command handler with session
    app.on_command_ctx::<MyState, _, _, _>("count", storage.clone(), |ctx| async move {
        let mut state = ctx.session.get();
        state.counter += 1;
        ctx.session.set(state.clone());
        ctx.reply_and_done(&format!("Count: {}", state.counter)).await
    });

    // Handle all text messages
    app.on_update_ctx::<MyState, _, _, _>(storage, |ctx| async move {
        match ctx.text() {
            Some(text) if !text.starts_with('/') => {
                ctx.reply_and_done(&format!("You said: {}", text)).await
            }
            _ => Ctx::skip(), // Not a text message, skip to next handler
        }
    });

    app.on_fetch(req, env, ctx).await.map_err(|e| e.into())
}
```

## API Overview

### Context Methods

| Method | Description |
|--------|-------------|
| `ctx.reply(text)` | Send a text message |
| `ctx.reply_html(text)` | Send HTML-formatted message |
| `ctx.reply_to(text)` | Reply to the current message (quote) |
| `ctx.reply_and_done(text)` | Reply and end handler |
| `ctx.edit_text(text)` | Edit message text |
| `ctx.delete_message()` | Delete the current message |
| `ctx.answer_callback(text, show_alert)` | Answer callback query |
| `ctx.send_photo(photo)` | Send a photo |
| `Context::done()` | End handler processing |
| `Context::skip()` | Skip to next handler |

### Accessor Methods

| Method | Description |
|--------|-------------|
| `ctx.chat_id()` | Get chat ID |
| `ctx.user_id()` | Get user ID |
| `ctx.message_id()` | Get message ID |
| `ctx.text()` | Get message text |
| `ctx.command()` | Get command name (without `/`) |
| `ctx.command_args()` | Get command arguments |
| `ctx.callback_data()` | Get callback query data |
| `ctx.telegram_api()` | Get raw Telegram API client |

## Session Management

Sessions are automatically loaded and saved per chat. Use KV storage for persistence:

```rust
// Define your state type
#[derive(Default, Clone, Serialize, Deserialize)]
struct UserState {
    step: String,
    data: Option<String>,
}

// Create storage from KV binding
let storage = KvStorage::from_env(&env, "SESSION_KV", "prefix")?;

// Access session in handler
app.on_command_ctx::<UserState, _, _, _>("start", storage, |ctx| async move {
    ctx.session.set(UserState {
        step: "awaiting_input".into(),
        data: None,
    });
    ctx.reply_and_done("Please enter your name:").await
});
```

## Filter Combinators

Use filters with `on_update_when` or check conditions in handlers:

```rust
use tgbot_worker_rs::filter::*;

// Available filters
is_message(&update)           // Is a message
is_callback_query(&update)    // Is a callback query
has_text(&update)             // Has text content
is_command(&update)           // Is a command (starts with /)
text_contains("hello")        // Text contains substring
text_starts_with("hi")        // Text starts with prefix
callback_data_equals("btn1")  // Callback data matches
from_chat(chat_id)            // From specific chat
from_user(user_id)            // From specific user

// Combinators
and(is_message, has_text)     // Both conditions
or(is_message, is_callback_query)  // Either condition
not(is_command)               // Negate filter
```

## Examples

See the examples:

- `examples/version`: command routing, Cloudflare KV, D1, and Queues integration. [Guide](examples/version/README.MD)
- `examples/middleware`: middleware usage and reply messages. [Guide](examples/middleware/README.MD)
- `examples/session`: multi-step registration flow with session state. 

### Running Examples

```bash
# Install toolchain
rustup toolchain install 1.89.0
rustup target add wasm32-unknown-unknown --toolchain 1.89.0

# Install Wrangler
npm i -g wrangler

# Run example locally
cd examples/session
wrangler secret put API_KEY  # Your Telegram bot token
wrangler dev

# Deploy
wrangler publish

# Set webhook
curl "https://api.telegram.org/bot<TOKEN>/setWebhook?url=<WORKER_URL>/telegramMessage"
```

## Contributing

Contributions are welcome! If you'd like to contribute to this project, please
follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure they adhere to the project's coding style.
4. Submit a pull request with a clear description of your changes.

### Code Review Process

To ensure code quality, all contributions will be reviewed by the maintainers.
Please be patient during this process.

## License

This project is licensed under the WTFPL License - see the [LICENSE](LICENSE)
file for details.
