# Repository Guidelines

## Project Structure & Module Organization
- Core library in `src/lib.rs` exposes `App` and routes: `GET /` health and `POST /telegramMessage` (delegates to your update handler).
- Key modules:
  - `src/error.rs` - `BotError` enum and `BotResult<T>` type alias
  - `src/filter.rs` - Filter combinators (`is_message`, `text_contains`, `and`, `or`, `not`, etc.)
  - `src/handler/` - dptree-style handler composition (`Handler`, `filter`, `endpoint`, `chain`, `branch`)
  - `src/dialogue/` - FSM/Dialogue system for multi-step conversations (`Dialogue<S, St>`)
  - `src/session/` - Session management with `Context<T, S>`, `KvStorage`, `SessionStorage` trait
  - `src/storage/` - KV and D1 storage helpers
  - `src/keyboard.rs` - Keyboard builders (`InlineKeyboard`, `ReplyKeyboard`)
  - `src/command.rs` - Command parsing utilities (`CommandParser`, `parse_duration`)
  - `src/retry.rs` - Retry utilities (`RetryPolicy`, `RetryContext`)
- Runnable Workers live under `examples/<name>/` with their own `Cargo.toml` and `wrangler.toml`:
  - `examples/version/` - KV, D1, Queues integration
  - `examples/middleware/` - Middleware usage
  - `examples/session/` - Multi-step registration flow with session state
- Tooling: `.cargo/config.toml` pins target `wasm32-unknown-unknown`; top‑level `wrangler.toml` defines build/publish settings.

Notice: Always use the `wasm32-unknown-unknown` target for builds and examples. Ensure the target is installed via `rustup target add wasm32-unknown-unknown`. Prefer running format/lint with the pinned toolchain (e.g., `cargo +1.91.1 fmt`, `cargo +1.91.1 clippy --all-targets -- -D warnings`). Avoid adding features or crates that require OS-level `std` functionality unavailable in Cloudflare Workers.

## Preferred API Patterns (v0.4.0+)

### Teloxide-style Simple API (Recommended)
```rust
// Simple command handler
app.command("start", |bot, msg| async move {
    bot.send_message(msg.chat_id(), "Hello!").await
});

// Message handler
app.on_message(|bot, msg| async move {
    bot.reply(&msg, "Got it!").await
});

// Callback query handler
app.on_callback_query(|bot, query| async move {
    bot.answer_callback(query.id(), Some("Clicked!"), false).await
});
```

### dptree-style Handler Composition (Advanced)
```rust
use tgbot_worker_rs::dptree;

let handler = dptree::entry()
    .branch(dptree::filter_command("start").chain(dptree::endpoint(handle_start)))
    .branch(dptree::endpoint_callback(handle_callback))
    .branch(dptree::endpoint(handle_fallback));

// Handler control flow:
// - Continue → proceed to next in chain
// - Skip → try next branch  
// - Break(Response) → stop processing
```

### Dialogue/FSM for Multi-step Flows
```rust
use tgbot_worker_rs::dialogue::Dialogue;

let dialogue = Dialogue::<MyState, KvStorage>::new(storage, chat_id);
dialogue.load().await.ok();
dialogue.update(MyState::NextStep);
dialogue.save().await.ok();
```

## Build, Test, and Development Commands
- Install target: `rustup target add wasm32-unknown-unknown`.
- Format/lint: `cargo fmt` and `cargo clippy --all-targets -- -D warnings`.
- Build library: `cargo build` or `cargo build --release` (WASM target set via `.cargo/config.toml`).
- Run example locally: `cd examples/version && wrangler dev` (visit `http://127.0.0.1:8787/` → "Bot is running!").
- Deploy example: `wrangler publish`.
- Secrets: `wrangler secret put API_KEY` (Telegram bot token; required by examples).

## Coding Style & Naming Conventions
- Rust 2024 edition; 4‑space indentation; always run `cargo fmt`.
- Naming: snake_case (functions/vars/modules), PascalCase (types/traits), SCREAMING_SNAKE_CASE (consts).
- Keep public surface minimal; group related logic into small modules; prefer pure functions for testability.

## Testing Guidelines
- Use `cargo test` for unit tests (inline with `#[cfg(test)]`).
- Integration tests (optional) in `tests/`.
- Test logic decoupled from Cloudflare bindings; mock Telegram API usage where possible.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (e.g., `feat:`, `fix:`, `refactor:`, `docs(scope): ...`) as used in history.
- PRs must include: clear summary, rationale, linked issues, and any relevant logs/screenshots. Run `fmt` and `clippy` before opening.

## Security & Configuration Tips
- Never hardcode secrets; use `wrangler secret` and access via `Env::secret`.
- Avoid logging tokens or full update payloads; redact sensitive fields.
- Keep `compatibility_date` current in `wrangler.toml` and review Worker permissions when adding features.

## Agent-Specific Notes
- New runnable examples belong in `examples/<name>/` with a scoped `wrangler.toml`.
- Do not change directory layout or crate names without prior discussion.
