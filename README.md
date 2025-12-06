# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.4.0-orange)

A lightweight, serverless Telegram bot framework for Cloudflare Workers, built with Rust.

[查看中文说明](README_zh.md)

## Table of Contents

- [Features](#features)
- [Roadmap](#roadmap)
- [Quick Start](#quick-start)
- [API Overview](#api-overview)
- [Filter Combinators](#filter-combinators)
- [Keyboard Builder](#keyboard-builder)
- [Command Parsing](#command-parsing)
- [dptree-style Handler](#dptree-style-handler)
- [Dialogue/FSM System](#dialoguefsm-system)
- [Retry Utilities](#retry-utilities)
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
| **High** | dptree Handler | Composable handler chains inspired by teloxide's dptree | ✅ Done |
| **High** | Dialogue/FSM | Multi-step conversation flows with state persistence | ✅ Done |
| **Medium** | Guard Filters | `is_private()`, `is_group()`, `is_user_in()` chat type guards | ✅ Done |
| **Medium** | Menu System | Interactive inline button menus with pagination | ✅ Done |
| **Medium** | Ignore Old Updates | Skip stale updates older than N seconds via `is_recent()` | ✅ Done |
| **Low** | I18n Support | Lightweight JSON-based internationalization | ✅ Done |
| **Low** | Metrics/Logging | Structured logging macros and metrics counter | ✅ Done |
| **Low** | Bot Commands Menu | Auto-register commands with Telegram via `setMyCommands` | ✅ Done |

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

### Admin Methods

| Method | Description |
|--------|-------------|
| `bot.ban_chat_member(chat_id, user_id, until_date, revoke)` | Ban a user |
| `bot.unban_chat_member(chat_id, user_id, only_if_banned)` | Unban a user |
| `bot.kick_chat_member(chat_id, user_id)` | Kick (ban + unban) |
| `bot.mute_chat_member(chat_id, user_id, until_date)` | Mute a user |
| `bot.unmute_chat_member(chat_id, user_id)` | Unmute a user |
| `bot.restrict_chat_member(chat_id, user_id, perms, until)` | Restrict permissions |
| `bot.promote_chat_member(chat_id, user_id, rights)` | Promote to admin |
| `bot.pin_message(chat_id, msg_id, silent)` | Pin a message |
| `bot.unpin_message(chat_id, msg_id)` | Unpin a message |
| `bot.unpin_all_messages(chat_id)` | Unpin all messages |
| `bot.set_chat_title(chat_id, title)` | Set chat title |
| `bot.set_chat_description(chat_id, desc)` | Set chat description |

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

## Filter Combinators

Use filters to check conditions in handlers:

```rust
use tgbot_worker_rs::filter::*;

// Update type filters
is_message(&update)           // Is a message
is_callback_query(&update)    // Is a callback query
has_text(&update)             // Has text content
is_command(&update)           // Is a command (starts with /)

// Text filters
text_contains("hello")        // Text contains substring
text_starts_with("hi")        // Text starts with prefix
callback_data_equals("btn1")  // Callback data matches
from_chat(chat_id)            // From specific chat
from_user(user_id)            // From specific user

// Chat type guards
is_private(&update)           // Private chat only
is_group(&update)             // Group or supergroup
is_channel(&update)           // Channel only
is_user_in(&[123, 456])       // User whitelist (admin check)

// Time filter
is_recent(60)                 // Skip updates older than 60 seconds

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

## Menu System

Build paginated inline menus for navigation:

```rust
use tgbot_worker_rs::menu::Menu;

// Create a paginated menu
let menu = Menu::new("settings")
    .item("Profile", "profile")
    .item("Notifications", "notifications")
    .item("Privacy", "privacy")
    .item("Language", "language")
    .item("Help", "help")
    .page_size(3);

// Send first page
let keyboard = menu.build_page(0);
bot.send_inline_keyboard(chat_id, "Settings:", keyboard).await?;

// Handle pagination callback
if let Some(page) = Menu::parse_page(callback_data, "settings") {
    let keyboard = menu.build_page(page);
    bot.edit_with_keyboard(chat_id, msg_id, "Settings:", keyboard.build()).await?;
}
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

## dptree-style Handler

Build composable handler chains inspired by [teloxide's dptree](https://github.com/teloxide/dptree):

```rust
use tgbot_worker_rs::dptree;

// Compose handlers with chain() and branch()
let handler = dptree::entry()
    // Try /start command first
    .branch(
        dptree::filter_command("start")
            .chain(dptree::endpoint(handle_start))
    )
    // Try /help command
    .branch(
        dptree::filter_command("help")
            .chain(dptree::endpoint(handle_help))
    )
    // Handle callback queries
    .branch(
        dptree::filter(|upd| matches!(upd.content, UpdateContent::CallbackQuery(_)))
            .chain(dptree::endpoint_callback(handle_callback))
    )
    // Fallback for other messages
    .branch(dptree::endpoint(handle_fallback));

async fn handle_start(bot: Bot, msg: Message) -> BotResult<()> {
    bot.send_message(msg.chat_id(), "Welcome!").await
}
```

Handler control flow:
- `Continue` → proceed to next handler in chain
- `Skip` → try next branch
- `Break(Response)` → stop processing

## Dialogue/FSM System

Build multi-step conversation flows with state persistence:

```rust
use tgbot_worker_rs::dialogue::Dialogue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
enum RegState {
    #[default]
    Start,
    AwaitingName,
    AwaitingEmail { name: String },
}

async fn handle_registration(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<RegState, KvStorage>,
) -> BotResult<()> {
    // Load state from storage
    dialogue.load().await.ok();

    match dialogue.get() {
        RegState::Start => {
            bot.send_message(msg.chat_id(), "What's your name?").await?;
            dialogue.update(RegState::AwaitingName);
        }
        RegState::AwaitingName => {
            let name = msg.text().unwrap_or("").to_string();
            bot.send_message(msg.chat_id(), "What's your email?").await?;
            dialogue.update(RegState::AwaitingEmail { name });
        }
        RegState::AwaitingEmail { name } => {
            let email = msg.text().unwrap_or("");
            bot.send_message(
                msg.chat_id(),
                &format!("Done! {} <{}>", name, email)
            ).await?;
            dialogue.exit().await.ok();
        }
    }
    dialogue.save().await.ok();
    Ok(())
}
```

## Retry Utilities

Detect retryable errors and extract retry timing:

```rust
use tgbot_worker_rs::retry::{is_retryable, get_retry_after};

match bot.send_message(chat_id, "Hello").await {
    Ok(_) => { /* success */ }
    Err(e) => {
        let err = e.to_string();
        if is_retryable(&err) {
            // Get Telegram's suggested delay (for 429 errors)
            let delay = get_retry_after(&err).unwrap_or(30);
            // Handle retry as needed
        }
    }
}
```

## Bot Commands Menu

Register commands with Telegram's command menu:

```rust
// Set commands
bot.set_my_commands(&[
    ("start", "Start the bot"),
    ("help", "Show help message"),
    ("settings", "Open settings"),
]).await?;

// Get current commands
let commands = bot.get_my_commands().await?;

// Delete all commands
bot.delete_my_commands().await?;
```

## I18n (Internationalization)

Lightweight JSON-based translation system:

```rust
use tgbot_worker_rs::i18n::I18n;

let mut i18n = I18n::new("en");

// Load translations
i18n.load_json("en", r#"{"hello": "Hello, {name}!", "bye": "Goodbye!"}"#)?;
i18n.load_json("zh", r#"{"hello": "你好，{name}！", "bye": "再见！"}"#)?;

// Get translation
let text = i18n.t("en", "bye"); // "Goodbye!"

// With argument substitution
let text = i18n.t_args("zh", "hello", &[("name", "世界")]); // "你好，世界！"
```

## Logging & Metrics

Structured logging macros for Cloudflare Workers:

```rust
use tgbot_worker_rs::{log_info, log_error, log_warn, log_debug};
use tgbot_worker_rs::logging::Metrics;

log_info!("Bot started");
log_error!("Failed to process: {}", error);

// Track metrics
let mut metrics = Metrics::new();
metrics.record_update();
metrics.record_command("start");
log_info!("{}", metrics.summary());
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
