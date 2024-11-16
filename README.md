# Telegram Bot Worker (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)

This project is a Telegram bot running on Cloudflare Workers, built using Rust.

[查看中文说明](README_ZH.md)

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
- You can write your logic and calling set_on_update `App::set_on_update` to handle incoming message or etc.
- You can use `App::api` to access async api like using `frankenstein` crate.
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

To see a practical example of how to use this bot, please refer to the `examples` directory. It contains sample code
demonstrating how to handle commands and interact with the Telegram Bot API.

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
