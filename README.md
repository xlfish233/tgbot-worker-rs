# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.3.0-orange)

A lightweight, serverless Telegram bot framework for Cloudflare Workers, built with Rust.

[查看中文说明](README_zh.md)

## Table of Contents

- [Features](#features)
- [Roadmap](#roadmap)
- [Quick Start](#quick-start)
- [API Overview](#api-overview)
- [Session Management](#session-management)
- [Filter Combinators](#filter-combinators)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Serverless-first**: Designed for Cloudflare Workers with zero cold-start overhead
- **Teloxide-style API**: Simple handler signatures like `|bot, msg| async { bot.send_message(...) }`
- **Session Management**: Built-in KV-based session storage with type-safe state
- **Filter Combinators**: Composable filters like `is_message`, `text_contains`, `callback_data_equals`
- **Rich Telegram Methods**: `reply`, `reply_html`, `edit_text`, `delete_message`, `answer_callback`, `send_photo`
- **Middleware Support**: Request/response pipeline with short-circuit capability
- **Type Safety**: Leverages Rust's type system with `BotError` and `BotResult`

**Project Status:** Active development. Contributions welcome!

Note: This project targets `wasm32-unknown-unknown` (pinned via `.cargo/config.toml`). Install the target with `rustup target add wasm32-unknown-unknown` and prefer running commands with the pinned toolchain (`+1.91.1`).

## Roadmap

Planned features and improvements (contributions welcome!):

| Priority | Feature | Description | Status |
|----------|---------|-------------|--------|
| **High** | Keyboard Builder | Type-safe inline/reply keyboard construction API | ✅ Done |
| **High** | Command Argument Parsing | Structured parsing: `/remind 30m "text"` → `(Duration, String)` | ✅ Done |
| **High** | Rate Limiting | Auto-retry with exponential backoff, flood wait handling | ✅ Done |
| **Medium** | Guard Middleware | `only_admin()`, `only_private()`, `only_group()` permission guards | 🔲 TODO |
| **Medium** | Conversation/Wizard | Multi-step conversation flows with branching logic | 🔲 TODO |
| **Medium** | Menu System | Interactive inline button menus with pagination | 🔲 TODO |
| **Medium** | Ignore Old Updates | Skip stale updates older than N seconds | 🔲 TODO |
| **Low** | I18n Support | Internationalization/localization helpers | 🔲 TODO |
| **Low** | Metrics/Logging | Structured logging and update processing metrics | 🔲 TODO |
| **Low** | Bot Commands Menu | Auto-register commands with Telegram via `setMyCommands` | 🔲 TODO |

> Inspired by mainstream frameworks: [teloxide](https://github.com/teloxide/teloxide), [grammY](https://grammy.dev/), [python-telegram-bot](https://python-telegram-bot.org/)

## Quick Start

### Simple API (teloxide-style)

```rust
use tgbot_worker_rs::prelude::*;
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();

    // Handle /start command
    app.command("start", |bot, msg| async move {
        bot.send_message(msg.chat_id(), "Hello! I'm a bot.").await
    });

    // Handle /echo <text> command
    app.command("echo", |bot, msg| async move {
        let text = msg.command_args().unwrap_or("nothing");
        bot.send_message(msg.chat_id(), &format!("Echo: {}", text)).await
    });

    // Handle callback queries
    app.on_callback_query(|bot, query| async move {
        bot.answer_callback(query.id(), Some("Clicked!"), false).await
    });

    // Fallback for other messages
    app.on_message(|bot, msg| async move {
        // Skip commands (already handled)
        if msg.text().map(|t| t.starts_with('/')).unwrap_or(false) {
            return Err(BotError::Skip);
        }
        bot.send_message(msg.chat_id(), "Use /start to begin").await
    });

    app.run(req, env, ctx).await
}
```

### Session API (for stateful handlers)

```rust
use serde::{Deserialize, Serialize};
use tgbot_worker_rs::prelude::*;
use worker::*;

#[derive(Default, Clone, Serialize, Deserialize)]
struct MyState { counter: u32 }

type Ctx = Context<MyState, KvStorage>;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    let mut app = App::new();
    let storage = KvStorage::from_env(&env, "SESSION_KV", "session")?;

    app.on_command_ctx::<MyState, _, _, _>("count", storage, |ctx| async move {
        let mut state = ctx.session.get();
        state.counter += 1;
        ctx.session.set(state.clone());
        ctx.reply_and_done(&format!("Count: {}", state.counter)).await
    });

    app.on_fetch(req, env, ctx).await
}
```

## API Overview

### Simple API (App methods)

| Method | Description |
|--------|-------------|
| `app.command("cmd", \|bot, msg\|)` | Handle `/cmd` command |
| `app.on_message(\|bot, msg\|)` | Handle all messages |
| `app.on_callback_query(\|bot, query\|)` | Handle callback queries |
| `app.run(req, env, ctx)` | Run the bot |

### Bot Methods

| Method | Description |
|--------|-------------|
| `bot.send_message(chat_id, text)` | Send a text message |
| `bot.send_html(chat_id, text)` | Send HTML-formatted message |
| `bot.reply(&msg, text)` | Reply to a message (quote) |
| `bot.reply_html(&msg, text)` | Reply with HTML formatting |
| `bot.answer_callback(id, text, alert)` | Answer callback query |
| `bot.edit_message(chat_id, msg_id, text)` | Edit message text |
| `bot.delete_message(chat_id, msg_id)` | Delete a message |
| `bot.send_photo(chat_id, photo)` | Send a photo |

### Message Accessors

| Method | Description |
|--------|-------------|
| `msg.chat_id()` | Get chat ID |
| `msg.message_id()` | Get message ID |
| `msg.text()` | Get message text |
| `msg.from()` | Get sender User |
| `msg.command()` | Get command name (without `/`) |
| `msg.command_args()` | Get command arguments |

### CallbackQuery Accessors

| Method | Description |
|--------|-------------|
| `query.id()` | Get callback query ID |
| `query.data()` | Get callback data |
| `query.from()` | Get user who clicked |
| `query.chat_id()` | Get chat ID |
| `query.message_id()` | Get message ID |

### Context Methods (Session API)

| Method | Description |
|--------|-------------|
| `ctx.reply(text)` | Send a text message |
| `ctx.reply_and_done(text)` | Reply and end handler |
| `ctx.edit_text(text)` | Edit message text |
| `ctx.delete_message()` | Delete the current message |
| `Context::done()` | End handler processing |
| `Context::skip()` | Skip to next handler |

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

## Keyboard Builder

Build inline and reply keyboards with a fluent API:

```rust
use tgbot_worker_rs::prelude::*;

// Inline keyboard (buttons below message)
let keyboard = InlineKeyboard::new()
    .row([
        InlineButton::callback("Yes", "yes"),
        InlineButton::callback("No", "no"),
    ])
    .button(InlineButton::url("Visit", "https://example.com"));

bot.send_inline_keyboard(chat_id, "Choose an option:", keyboard).await?;

// Reply keyboard (custom keyboard replacing default)
let keyboard = ReplyKeyboard::new()
    .text("Option 1")
    .text("Option 2")
    .resize()
    .one_time();

bot.send_reply_keyboard(chat_id, "Select:", keyboard).await?;
```

## Command Parsing

Parse command arguments with type safety:

```rust
use tgbot_worker_rs::prelude::*;

// Parse "/ban 123 1h spam reason"
let parser = CommandParser::new(msg.text().unwrap_or(""));
if parser.is_command("ban") {
    let user_id: u64 = parser.arg(0)?;      // 123
    let duration: String = parser.arg(1)?;   // "1h"
    let reason = parser.rest(2);             // Some("spam reason")
}

// Parse duration strings
use tgbot_worker_rs::command::parse_duration;
let seconds = parse_duration("30m")?;  // 1800
let seconds = parse_duration("1h")?;   // 3600
let seconds = parse_duration("1d")?;   // 86400
```

## Retry Utilities

Handle rate limits and transient errors:

```rust
use tgbot_worker_rs::prelude::*;

let policy = RetryPolicy::new(3, 1000)  // 3 retries, 1s base delay
    .with_max_delay(30000);              // cap at 30s

let mut ctx = RetryContext::new(policy);
while ctx.can_retry() {
    match bot.send_message(chat_id, "Hello").await {
        Ok(_) => break,
        Err(e) => {
            ctx.record_failure(&e.to_string());
            // In serverless: schedule retry via queue after ctx.next_delay_seconds()
        }
    }
}
```

## Examples

See the examples:

- `examples/version`: command routing, Cloudflare KV, D1, and Queues integration. [Guide](examples/version/README.MD)
- `examples/middleware`: middleware usage and reply messages. [Guide](examples/middleware/README.MD)
- `examples/session`: multi-step registration flow with session state. 

### Running Examples

```bash
# Install toolchain
rustup toolchain install 1.91.1
rustup target add wasm32-unknown-unknown --toolchain 1.91.1

# Install Wrangler
npm i -g wrangler

# Run example locally
cd examples/version
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

## Acknowledgements

This project is heavily inspired by [teloxide](https://github.com/teloxide/teloxide), an elegant Telegram bot framework for Rust. Many API designs, including the simplified handler signatures, keyboard builders, and command parsing patterns, are adapted from teloxide's excellent architecture.

Special thanks to the teloxide team for creating such a well-designed framework that serves as a reference for the Rust Telegram bot ecosystem.

## License

This project is licensed under the WTFPL License - see the [LICENSE](LICENSE)
file for details.
