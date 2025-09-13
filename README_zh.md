# Telegram Bot Worker (Rust)

![构建状态](https://img.shields.io/badge/build-passing-brightgreen)
![许可证](https://img.shields.io/badge/license-WTFPL-blue)
![版本](https://img.shields.io/badge/version-0.2.0-orange)

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
- 通过 `app.on_command(...)`、`app.on_update(...)` 注册消息处理器，或使用 `app.use_middleware(...)` 挂载中间件。
- 在处理器里直接使用 `frankenstein::AsyncApi` 调用 Telegram API。
- 如果您想了解更多使用细节，请参见 `examples` 目录。

**项目状态：** 该项目仍在开发中，目前仅具有基本实现。如果您有任何好的建议，请随时提出，我们将考虑实施。

注意：本项目固定目标为 `wasm32-unknown-unknown`（通过 `.cargo/config.toml` 设置）。请先安装目标：`rustup target add wasm32-unknown-unknown`，并优先使用固定工具链版本（例如 `+1.89.0`）运行相关命令。

## 功能

- **Cloudflare Workers 集成：** 利用 Cloudflare Workers 的强大和可扩展性。
- **Rust 开发：** 使用 Rust 构建，注重性能和安全性。
- **Telegram Bot API：** 与 Telegram Bot API 交互以接收更新和发送响应。
- **Webhook 支持：** 支持 webhook 设置，以便与 Telegram 进行无缝通信。
- **可扩展架构：** 设计为易于扩展，支持自定义命令和功能。

## 使用示例

请参考以下示例：

- `examples/version`：展示命令路由、KV、D1 与消息队列（Queues）的集成。参见使用说明：[examples/version/README.MD](examples/version/README.MD)
- `examples/middleware`：展示如何使用中间件（`use_middleware`）以及如何发送“回复消息”。英文版：[examples/middleware/README.MD](examples/middleware/README.MD) · 中文版：[examples/middleware/README_zh.MD](examples/middleware/README_zh.MD)

**示例可用命令**

- `/version` — 输出包版本
- `/kv_set <key> <value>` — 在 KV 中写入（前缀 `demo`）
- `/kv_get <key>` — 从 KV 读取
- `/d1_ping` — 在 D1 上执行 `SELECT 1 AS n` 并输出 JSON
- `/queue_echo <text>` — 入队一个任务，由队列消费者异步回发

**快速开始**

- 安装工具链与目标
  - `rustup toolchain install 1.89.0`
  - `rustup target add wasm32-unknown-unknown --toolchain 1.89.0`
- 安装 Wrangler（v3）
  - `npm i -g wrangler`
- 格式/检查
  - `cargo +1.89.0 fmt`
  - `cargo +1.89.0 clippy --all-targets -- -D warnings`

**为示例配置绑定**

- Secret
  - `cd examples/version`
  - `wrangler secret put API_KEY`（Telegram 机器人 Token）
- KV（在 `examples/version/wrangler.toml` 中替换 ID）
  - `wrangler kv namespace create tgbot-worker-rs-demo`
  - 将生成的 `id`、`preview_id` 填入 `[[kv_namespaces]]`，`binding = "KV"`
- D1
  - `wrangler d1 create example_db`
  - 将生成的 `database_id` 填入 `[[d1_databases]]`，`binding = "DB"`
  - 可选：`wrangler d1 migrations apply DB`（使用 `migrations/0001_init.sql`）
- 队列（Queues）
  - `wrangler queues create demo-queue`
  - 确保 `[[queues.producers]]` 里 `binding = "QUEUE"`，`queue = "demo-queue"`
  - 确保 `[[queues.consumers]]` 里 `queue = "demo-queue"`

**本地运行**

- `cd examples/version && wrangler dev`
  - 打开 `http://127.0.0.1:8787/` → `Bot is running!`
  - D1/Queues 建议使用 `wrangler dev --remote` 以使用云端后端
  - 本地模拟 Telegram 更新：
    - `curl -sS -X POST http://127.0.0.1:8787/telegramMessage -H 'content-type: application/json' -d '{"update_id":1,"message":{"message_id":1,"chat":{"id":123,"type":"private"},"text":"/kv_set foo bar"}}'`

**发布并设置 Telegram Webhook**

- `cd examples/version && wrangler publish`
- 设置 webhook（替换占位符）：
  - `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`
- 在 Telegram 中给你的机器人发送上述命令进行验证

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
