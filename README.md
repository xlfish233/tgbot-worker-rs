# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.2.0-orange)

This project is a Telegram bot running on Cloudflare Workers, built using Rust.

[查看中文说明](README_zh.md)

## Table of Contents

- [Telegram Bot Worker (Rust)](#telegram-bot-worker-rust)

    - [Description](#description)
    - [Features](#features)
    - [Usage Example](#usage-example)
    - [Project Details](#project-details)
    - [Contributing](#contributing)
        - [Special Notes](#special-notes)
        - [Code Review Process](#code-review-process)
    - [License](#license)

## Description

- This project provides a basic Telegram bot framework built on Cloudflare Workers
  using Rust and frankenstein's api.
- Register update handlers via `app.on_command(...)`, `app.on_update(...)`, or middleware `app.use_middleware(...)`.
- Use `frankenstein::AsyncApi` directly to call Telegram API from handlers.
- If you want known more details of usage please see the `examples` directory.

**Project Status:** This project is still under development and currently only has basic implementations. If you have
any good suggestions, please feel free to propose them, and they will be considered for implementation.

## Features

- **Cloudflare Workers Integration:** Leverages the power and scalability of
  Cloudflare Workers.
- **Rust Development:** Built with Rust for performance and safety.
- **Telegram Bot API:** Interacts with the Telegram Bot API for receiving
  updates and sending responses.
- **Webhook Support:** Supports webhook setup for seamless communication with
  Telegram.
- **Extensible Architecture:** Designed to be easily extended with custom
  commands and functionalities.

## Usage Example

To see a practical example of how to use this bot, please refer to `examples/version`. It shows command routing, Cloudflare KV, D1, and Queues integration.

**Available commands in the example**

- `/version` — replies with package version
- `/kv_set <key> <value>` — stores a key/value in KV (prefix `demo`)
- `/kv_get <key>` — reads the value from KV
- `/d1_ping` — runs `SELECT 1 AS n` on D1 and echoes rows as JSON
- `/queue_echo <text>` — enqueues a job and replies from the queue consumer

**Quick start**

- Install toolchain and target
  - `rustup toolchain install 1.89.0`
  - `rustup target add wasm32-unknown-unknown --toolchain 1.89.0`
- Install Wrangler (v3)
  - `npm i -g wrangler`
- Format/lint
  - `cargo +1.89.0 fmt`
  - `cargo +1.89.0 clippy --all-targets -- -D warnings`

**Configure example bindings**

- Secrets
  - `cd examples/version`
  - `wrangler secret put API_KEY` (Telegram Bot token)
- KV (replace IDs in `examples/version/wrangler.toml`)
  - `wrangler kv namespace create tgbot-worker-rs-demo`
  - Copy the `id` and `preview_id` into `[[kv_namespaces]]` with `binding = "KV"`
- D1
  - `wrangler d1 create example_db`
  - Put its `database_id` into `[[d1_databases]]` with `binding = "DB"`
  - Optional: `wrangler d1 migrations apply DB` (uses `migrations/0001_init.sql`)
- Queues
  - `wrangler queues create demo-queue`
  - Ensure `[[queues.producers]]` has `binding = "QUEUE"`, `queue = "demo-queue"`
  - Ensure `[[queues.consumers]]` has `queue = "demo-queue"`

**Run locally**

- `cd examples/version && wrangler dev`
  - Visit `http://127.0.0.1:8787/` → `Bot is running!`
  - For D1/Queues, prefer `wrangler dev --remote` to run against Cloudflare backend
  - To simulate a Telegram update locally:
    - `curl -sS -X POST http://127.0.0.1:8787/telegramMessage -H 'content-type: application/json' -d '{"update_id":1,"message":{"message_id":1,"chat":{"id":123,"type":"private"},"text":"/kv_set foo bar"}}'`

**Publish and set Telegram webhook**

- `cd examples/version && wrangler publish`
- Set webhook (replace placeholders):
  - `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`
- Send commands to your bot in Telegram to verify behavior

## Project Details

- **Project Name:** `tgbot-worker-rs`
- **Version:** `0.1.0`
- **Author:** `xiaolin <446304319@qq.com>`

**Dependencies:**

- `worker`: Provides the core functionality for building Cloudflare Workers.
- `worker-macros`: Provides macros for simplifying the development of Cloudflare
  Workers.
- `console_error_panic_hook`: Captures and logs panic messages to the console.
- `serde`: Enables serialization and deserialization of data structures.
- `serde_json`: Provides JSON serialization and deserialization support. 
- `frankenstein`: A library for interacting with the Telegram Bot API.

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
