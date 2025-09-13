# 版本示例使用说明（version）

本示例展示：命令路由、Cloudflare KV、D1 数据库与消息队列（Queues）的集成用法。

## 可用命令

- `/version` — 输出包版本。
- `/kv_set <key> <value>` — 在 KV 中写入键值（前缀 `demo`）。
- `/kv_get <key>` — 从 KV 读取键值。
- `/d1_ping` — 在 D1 执行 `SELECT 1 AS n` 并返回 JSON。
- `/queue_echo <text>` — 入队一个任务，由队列消费者异步回发。

## 运行前准备

1) 安装 WASM 目标（必须）
- 项目目标为 `wasm32-unknown-unknown`。
- 一次性安装：`rustup target add wasm32-unknown-unknown`。

2) 设置 Bot Token（Secret）
- `cd examples/version`
- `wrangler secret put API_KEY`（Telegram 机器人 Token）

3) 绑定配置（按需）
- KV：在 `examples/version/wrangler.toml` 中替换 `[[kv_namespaces]]` 的 `id` 与 `preview_id`。
  - 可用命令：`wrangler kv namespace create tgbot-worker-rs-demo`
- D1：在 `[[d1_databases]]` 中填入 `database_id`，并可使用迁移目录 `migrations/`。
  - 可用命令：`wrangler d1 create example_db`
  - 迁移（可选）：`wrangler d1 migrations apply DB`
- Queues：确保 `[[queues.producers]]` 与 `[[queues.consumers]]` 指向同一个队列名。
  - 可用命令：`wrangler queues create demo-queue`

## 本地运行

- `wrangler dev`
  - 打开 `http://127.0.0.1:8787/` → 应返回 `Bot is running!`
  - 建议对 D1/Queues 使用 `wrangler dev --remote` 以使用云端后端
  - 本地模拟 Telegram 更新（示例）：
    - `curl -sS -X POST http://127.0.0.1:8787/telegramMessage -H 'content-type: application/json' -d '{"update_id":1,"message":{"message_id":1,"chat":{"id":123,"type":"private"},"text":"/kv_set foo bar"}}'`

## 发布与设置 Webhook

- `wrangler publish`
- 设置 webhook（替换占位符）：
  - `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`

安全提示：不要在日志中打印 Token 或完整 Update 负载；仅输出必要信息，避免泄露隐私与敏感数据。

