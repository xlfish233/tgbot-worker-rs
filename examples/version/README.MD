<!-- 查看中文：README_zh.MD -->

# Usage Instructions

This example demonstrates command routing, Cloudflare KV, D1, and Queues integration.

## Commands

- `/version` — replies with the package version.
- `/kv_set <key> <value>` — stores a key/value in KV with prefix `demo`.
- `/kv_get <key>` — reads a key from KV.
- `/d1_ping` — runs `SELECT 1 AS n` on D1 and replies with JSON.
- `/queue_echo <text>` — enqueues a job; the queue consumer replies asynchronously.

## Prerequisites

- WASM target (required)
  - Target: `wasm32-unknown-unknown`
  - Install: `rustup target add wasm32-unknown-unknown`
- Bot token as secret
  - `wrangler secret put API_KEY`

## Bindings (as needed)

- KV
  - Create: `wrangler kv namespace create tgbot-worker-rs-demo`
  - Copy its `id` and `preview_id` into `[[kv_namespaces]]` in `examples/version/wrangler.toml` (binding `KV`).
- D1
  - Create: `wrangler d1 create example_db`
  - Put its `database_id` into `[[d1_databases]]` in `wrangler.toml` (binding `DB`).
  - Optional migrations: `wrangler d1 migrations apply DB` (uses `migrations/0001_init.sql`).
- Queues
  - Create: `wrangler queues create demo-queue`
  - Ensure `[[queues.producers]]` has `binding = "QUEUE"`, `queue = "demo-queue"`.
  - Ensure `[[queues.consumers]]` has `queue = "demo-queue"`.

## Run locally

- `wrangler dev`
  - Visit `http://127.0.0.1:8787/` → `Bot is running!`
  - For D1/Queues, prefer `wrangler dev --remote` to use Cloudflare backend
  - Simulate a Telegram update:
    - `curl -sS -X POST http://127.0.0.1:8787/telegramMessage -H 'content-type: application/json' -d '{"update_id":1,"message":{"message_id":1,"chat":{"id":123,"type":"private"},"text":"/kv_set foo bar"}}'`

## Publish and set webhook

- `wrangler publish`
- Set webhook (replace placeholders):
  - `curl "https://api.telegram.org/bot<API_KEY>/setWebhook?url=<your_worker_url>/telegramMessage"`

Security note: Do not log tokens or the full Update payload; only log minimal necessary info.
