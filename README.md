# Telegram Bot Worker (Rust)

This project is a Telegram bot running on Cloudflare Workers, built using Rust.

## Table of Contents

- [Description](#description)
- [Features](#features)
- [Project Details](#project-details)
- [Get Started](#get-started)
- [Contributing](#contributing)


## Description

This project provides a basic Telegram bot framework built on Cloudflare Workers using Rust. It handles incoming updates from Telegram and allows you to define custom logic for responding to different commands and events.


## Features

- **Cloudflare Workers Integration:** Leverages the power and scalability of Cloudflare Workers.
- **Rust Development:** Built with Rust for performance and safety.
- **Telegram Bot API:** Interacts with the Telegram Bot API for receiving updates and sending responses.
- **Webhook Support:** Supports webhook setup for seamless communication with Telegram.
- **Extensible Architecture:** Designed to be easily extended with custom commands and functionalities.


## Project Details

- **Project Name:** `tgbot-worker-rs`
- **Version:** `0.1.0`
- **Author:** `xiaolin <446304319@qq.com>`

**Dependencies:**

- `worker`: Provides the core functionality for building Cloudflare Workers.
- `worker-macros`: Provides macros for simplifying the development of Cloudflare Workers.
- `console_error_panic_hook`: Captures and logs panic messages to the console.
- `serde`: Enables serialization and deserialization of data structures.
- `serde_json`: Provides JSON serialization and deserialization support.
- `frankenstein`: A library for interacting with the Telegram Bot API.
- `anyhow`: Provides a convenient way to handle errors.


## Get Started

### Setup Instructions

Before using this project, you need to set the `BOT_TOKEN` Wrangler secret:

```bash
wrangler secret put BOT_TOKEN 
```

Follow the prompts to securely store your bot's secret token.


### Webhook Setup

Before using this bot, you must set a webhook using the official Telegram API to your Cloudflare Worker. You can do this in two ways:

1. **Using the Telegram API:** Set the webhook to your Cloudflare Worker URL through the official Telegram API.
2. **Using the Bot's Endpoint:** Call the `set_webhook` endpoint of this bot to configure the webhook.

**Important:** Failure to set a webhook will prevent the bot from functioning correctly.


## Contributing

Contributions are welcome! If you'd like to contribute to this project, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure they adhere to the project's coding style.
4. Submit a pull request with a clear description of your changes.
