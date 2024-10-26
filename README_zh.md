# Telegram 大善人机器人 (Rust)

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-WTFPL-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)

这个项目是一个在 Cloudflare Workers 上运行的 Telegram 机器人，使用 Rust 构建。

## 目录

- [Telegram 大善人机器人 (Rust)](#telegram-大善人机器人-rust)
  - [目录](#目录)
  - [描述](#描述)
  - [功能](#功能)
  - [使用示例](#使用示例)
  - [项目细节](#项目细节)
  - [开始使用](#开始使用)
    - [设置说明](#设置说明)
    - [Webhook 设置](#webhook-设置)
  - [API 文档](#api-文档)
    - [端点](#端点)
  - [贡献](#贡献)
    - [特别说明](#特别说明)
    - [代码审查流程](#代码审查流程)
  - [许可证](#许可证)

## 描述

这个项目提供了一个基于 Cloudflare Workers 的基本 Telegram 机器人框架，使用 Rust。它处理来自 Telegram 的更新，并允许您定义自定义逻辑以响应不同的命令和事件。

**注意**：`telegramApi` 旨在帮助那些直接访问 Telegram 有困难的用户进行调试。当您将 `TEST_ENV` 设置为 `1` 时，它将直接从密钥中检索 API，因此用户无需自己使用 API。它不应该在生产环境中使用。因此，请不要在生产环境中将 `TEST_ENV` 设置为 `1`。

## 功能

- **Cloudflare Workers 集成**：利用 Cloudflare Workers 的强大和可扩展性。
- **Rust 开发**：使用 Rust 构建以提高性能和安全性。
- **Telegram Bot API**：与 Telegram Bot API 交互以接收更新和发送响应。
- **Webhook 支持**：支持 webhook 设置，以便与 Telegram 无缝通信。
- **可扩展架构**：设计为易于通过自定义命令和功能进行扩展。

## 使用示例

以下是如何使用机器人的简单示例：

```rust
// 处理命令的示例代码
fn handle_command(command: &str) {
    match command {
        "/start" => println!("机器人已启动！"),
        _ => println!("未知命令！"),
    }
}
```

## 项目细节

- **项目名称**：`tgbot-worker-rs`
- **版本**：`0.1.0`
- **作者**：`xiaolin <446304319@qq.com>`

**依赖关系**：

- `worker`：提供构建 Cloudflare Workers 的核心功能。
- `worker-macros`：提供简化 Cloudflare Workers 开发的宏。
- `console_error_panic_hook`：捕获并记录控制台中的恐慌消息。
- `serde`：启用数据结构的序列化和反序列化。
- `serde_json`：提供 JSON 序列化和反序列化支持。
- `frankenstein`：一个与 Telegram Bot API 交互的库。
- `anyhow`：提供方便的错误处理方式。
- `convert_case`：用于转换大小写风格的实用工具。
- `axum`：一个用于构建 Rust API 的网络框架。
- `tower-service`：用于构建基于 tower 的应用程序的服务抽象。
- `tracing`：一个用于给 Rust 程序添加仪器的框架。

## 开始使用

### 设置说明

使用此项目之前，您需要设置 `BOT_TOKEN` Wrangler 密钥：

```bash
wrangler secret put BOT_TOKEN
```

按照提示安全存储您的机器人的密钥。

### Webhook 设置

在使用此机器人之前，您必须使用官方 Telegram API 将 webhook 设置为您的 Cloudflare Worker。您可以通过两种方式进行：

1. **使用 Telegram API**：通过官方 Telegram API 将 webhook 设置为您的 Cloudflare Worker URL。
2. **使用机器人的端点**：调用此机器人的 `/telegramApi/setWebhook` 端点以配置 webhook。

**重要**：未设置 webhook 将导致机器人无法正常工作。

## API 文档

### 端点

- **GET /**：返回一条简单的消息，指示机器人正在运行。
- **POST /telegramMessage**：处理传入的 Telegram 消息。
- **POST /telegramApi/setWebhook**：为机器人设置 webhook。
- **POST /telegramApi/getWebhookInfo**：检索当前 webhook 信息。
- **POST /telegramApi/deleteWebhook**：删除当前 webhook。
- **POST /telegramApi/setMyCommands**：为机器人设置自定义命令。

**注意**：上述端点是官方 Telegram Bot API 的一部分。有关更多详细信息，请参阅 [官方 Telegram Bot API 文档](https://core.telegram.org/bots/api)。

## 贡献

欢迎贡献！如果您想为此项目做出贡献，请遵循以下步骤：

1. Fork 该仓库。
2. 为您的功能或错误修复创建一个新分支。
3. 进行更改，并确保遵循项目的编码风格。
4. 提交一个拉取请求，简要描述您的更改。

### 特别说明

您可以通过在 `src/plugin/command_handler.rs` 中添加逻辑来实现简单功能。如果您需要其他功能或有特定请求，请联系作者或打开问题；作者将尽力考虑并满足它们。

### 代码审查流程

为了确保代码质量，所有贡献将由维护者审查。请在此过程中耐心等待。

## 许可证

该项目根据 WTFPL 许可证发布 - 请参阅 [LICENSE](LICENSE) 文件以获取详细信息。
