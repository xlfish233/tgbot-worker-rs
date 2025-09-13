# Repository Guidelines

## Project Structure & Module Organization
- Core library in `src/lib.rs` exposes `App` and routes: `GET /` health and `POST /telegramMessage` (delegates to your update handler).
- Runnable Workers live under `examples/<name>/` (see `examples/version/`) with their own `Cargo.toml` and `wrangler.toml`.
- Tooling: `.cargo/config.toml` pins target `wasm32-unknown-unknown`; top‑level `wrangler.toml` defines build/publish settings.

## Build, Test, and Development Commands
- Install target: `rustup target add wasm32-unknown-unknown`.
- Format/lint: `cargo fmt` and `cargo clippy --all-targets -- -D warnings`.
- Build library: `cargo build` or `cargo build --release` (WASM target set via `.cargo/config.toml`).
- Run example locally: `cd examples/version && wrangler dev` (visit `http://127.0.0.1:8787/` → "Bot is running!").
- Deploy example: `wrangler publish`.
- Secrets: `wrangler secret put API_KEY` (Telegram bot token; required by examples).

## Coding Style & Naming Conventions
- Rust 2021 edition; 4‑space indentation; always run `cargo fmt`.
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
