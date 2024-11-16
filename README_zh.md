# Telegram Bot Worker (Rust)

![构建状态](https://img.shields.io/badge/build-passing-brightgreen)
![许可证](https://img.shields.io/badge/license-WTFPL-blue)
![版本](https://img.shields.io/badge/version-0.1.0-orange)

该项目是一个运行在 Cloudflare Workers 上的 Telegram 机器人，使用 Rust 构建。

[View English Description](README.md)

## 目录

- [Telegram Bot Worker (Rust)](#telegram-bot-worker-rust)

    - [描述](#描述)
    - [功能](#功能)
    - [使用示例](#使用示例)
    - [项目详情](#项目详情)
    - [贡献](#贡献)
        - [特别说明](#特别说明)
        - [代码审查流程](#代码审查流程)
    - [许可证](#许可证)

## 描述

- 该项目提供了一个基于 Cloudflare Workers 的基本 Telegram 机器人框架，使用 Rust 和 frankenstein 的 API。
- 您可以编写自己的逻辑并调用 `App::set_on_update` 来处理传入消息等。
- 您可以使用 `App::api` 访问异步 API，如使用 `frankenstein` crate。
- 如果您想了解更多使用细节，请参见 `examples` 目录。

**项目状态：** 该项目仍在开发中，目前仅具有基本实现。如果您有任何好的建议，请随时提出，我们将考虑实施。

## 功能

- **Cloudflare Workers 集成：** 利用 Cloudflare Workers 的强大和可扩展性。
- **Rust 开发：** 使用 Rust 构建，注重性能和安全性。
- **Telegram Bot API：** 与 Telegram Bot API 交互以接收更新和发送响应。
- **Webhook 支持：** 支持 webhook 设置，以便与 Telegram 进行无缝通信。
- **可扩展架构：** 设计为易于扩展，支持自定义命令和功能。

## 使用示例

要查看如何使用此机器人的实际示例，请参考 `examples` 目录。它包含示例代码，演示如何处理命令并与 Telegram Bot API 交互。

## 项目详情

- **项目名称：** `tgbot-worker-rs`
- **版本：** `0.1.0`
- **作者：** `xiaolin <446304319@qq.com>`

**依赖：**

- `worker`：提供构建 Cloudflare Workers 的核心功能。
- `worker-macros`：提供简化 Cloudflare Workers 开发的宏。
- `console_error_panic_hook`：捕获并记录 panic 消息到控制台。
- `serde`：启用数据结构的序列化和反序列化。
- `serde_json`：提供 JSON 序列化和反序列化支持。
- `frankenstein`：与 Telegram Bot API 交互的库。

## 贡献

欢迎贡献！如果您想为此项目做出贡献，请按照以下步骤操作：

1. Fork 该仓库。
2. 为您的功能或 bug 修复创建一个新分支。
3. 进行更改并确保遵循项目的编码风格。
4. 提交一个清晰描述您更改的 pull request。

### 代码审查流程

为确保代码质量，所有贡献将由维护者进行审查。请在此过程中保持耐心。

## 许可证

该项目根据 WTFPL 许可证进行许可 - 详见 [LICENSE](LICENSE) 文件。
