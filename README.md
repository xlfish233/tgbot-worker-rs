# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)

This project is a Telegram bot running on Cloudflare Workers, built using Rust.

[查看中文说明](README_ZH.md)

## Table of Contents

- [Telegram Bot Worker (Rust)](#telegram-bot-worker-rust)
    - [Table of Contents](#table-of-contents)
    - [Description](#description)
    - [Features](#features)
    - [Usage Example](#usage-example)
    - [Version Command](#version-command)
    - [Project Details](#project-details)
    - [Get Started](#get-started)
        - [Setup Instructions](#setup-instructions)
        - [Webhook Setup](#webhook-setup)
    - [API Documentation](#api-documentation)
        - [Endpoints](#endpoints)
    - [Contributing](#contributing)
        - [Special Notes](#special-notes)
        - [Code Review Process](#code-review-process)
    - [License](#license)

## Description

This project provides a basic Telegram bot framework built on Cloudflare Workers
using Rust. It handles incoming updates from Telegram and allows you to define
custom logic for responding to different commands and events.

**Note:** The `telegramApi` is designed for users who have difficulty accessing
Telegram directly for debugging purposes. It will directly retrieve the API from
the secret when you set `TEST_ENV` to `1`, so users do not need to use the API
themselves. It should not be used in production. Therefore, do not set
`TEST_ENV` to `1` in production environments.

**Project Status:** This project is still under development and currently only has basic implementations. If you have any good suggestions, please feel free to propose them, and they will be considered for implementation.

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

Here is a simple example of how to use the bot:

```rust
// Example code to handle a command
fn handle_command(command: &str) {
    match command {
        "/start" => println!("Bot started!"),
        _ => println!("Unknown command!"),
    }
}
```

## Version Command

If everything is configured correctly, sending the `/version` command to your bot will reply with `Current version: 0.1.0`.

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
- `serde_json`: Provides JSON serialization and deserialization support. # Added
  serde_json dependency
- `frankenstein`: A library for interacting with the Telegram Bot API.
- `anyhow`: Provides a convenient way to handle errors.
- `convert_case`: A utility for converting case styles.
- `axum`: A web framework for building APIs with Rust.
- `tower-service`: A service abstraction for building tower-based applications.
- `tracing`: A framework for instrumenting Rust programs.

## Get Started

### Setup Instructions

Before using this project, you need to set the `BOT_TOKEN` Wrangler secret:

```bash
wrangler secret put BOT_TOKEN
```

Follow the prompts to securely store your bot's secret token.

### Webhook Setup

Before using this bot, you must set a webhook using the official Telegram API to
your Cloudflare Worker. You can do this in two ways:

1. **Using the Telegram API:** Set the webhook to your Cloudflare Worker URL
   through the official Telegram API.
2. **Using the Bot's Endpoint:** Call the `/telegramApi/setWebhook` endpoint of
   this bot to configure the webhook.

**Important:** Failure to set a webhook will prevent the bot from functioning
correctly.

## API Documentation

### Endpoints

- **GET /**: Returns a simple message indicating the bot is running.
- **POST /telegramMessage**: Handles incoming Telegram messages.
- **POST /telegramApi/setWebhook**: Sets the webhook for the bot.
- **POST /telegramApi/getWebhookInfo**: Retrieves the current webhook
  information.
- **POST /telegramApi/deleteWebhook**: Deletes the current webhook.
- **POST /telegramApi/setMyCommands**: Sets custom commands for the bot.

**Note:** The endpoints listed above are part of the official Telegram Bot API.
Please refer to the
[official Telegram Bot API documentation](https://core.telegram.org/bots/api)
for more details.

## Contributing

Contributions are welcome! If you'd like to contribute to this project, please
follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure they adhere to the project's coding style.
4. Submit a pull request with a clear description of your changes.

### Special Notes

You can implement simple functionalities by adding logic in
`src/plugin/command_handler.rs`. If you need additional features or have
specific requests, please contact the author or open an issue; the author will
do their best to consider and fulfill them.

### Code Review Process

To ensure code quality, all contributions will be reviewed by the maintainers.
Please be patient during this process.

## License

This project is licensed under the WTFPL License - see the [LICENSE](LICENSE)
file for details.
