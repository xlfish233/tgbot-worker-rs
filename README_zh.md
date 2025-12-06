# Telegram Bot Worker (Rust)

![构建状态](https://img.shields.io/badge/build-passing-brightgreen)
![许可证](https://img.shields.io/badge/license-WTFPL-blue)
![版本](https://img.shields.io/badge/version-0.3.0-orange)

轻量级、无服务器的 Telegram 机器人框架，专为 Cloudflare Workers 设计，使用 Rust 构建。

[View English Description](README.md)

## 目录

- [功能特性](#功能特性)
- [开发路线图](#开发路线图)
- [快速开始](#快速开始)
- [API 概览](#api-概览)
- [会话管理](#会话管理)
- [过滤器组合](#过滤器组合)
- [示例](#示例)
- [贡献](#贡献)
- [许可证](#许可证)

## 功能特性

- **无服务器优先**：专为 Cloudflare Workers 设计，零冷启动开销
- **会话管理**：内置基于 KV 的会话存储，支持类型安全的状态管理
- **简化 API**：`Context::done()`、`Context::skip()`、`reply_and_done()` 让处理器代码更简洁
- **过滤器组合**：可组合的过滤器如 `is_message`、`text_contains`、`callback_data_equals`
- **丰富的 Telegram 方法**：`reply`、`reply_html`、`edit_text`、`delete_message`、`answer_callback`、`send_photo`
- **中间件支持**：请求/响应管道，支持短路返回
- **类型安全**：利用 Rust 类型系统，提供 `BotError` 和 `BotResult`

**项目状态：** 活跃开发中，欢迎贡献！

注意：本项目固定目标为 `wasm32-unknown-unknown`（通过 `.cargo/config.toml` 设置）。请先安装目标：`rustup target add wasm32-unknown-unknown`，并优先使用固定工具链版本（例如 `+1.89.0`）运行相关命令。

## 开发路线图

计划中的功能和改进（欢迎贡献！）：

| 优先级 | 功能 | 描述 | 状态 |
|--------|------|------|------|
| **高** | 键盘构建器 | 类型安全的内联/回复键盘构建 API | 🔲 待开发 |
| **高** | 命令参数解析 | 结构化解析：`/remind 30m "文本"` → `(Duration, String)` | 🔲 待开发 |
| **高** | 速率限制 | 自动重试 + 指数退避 + flood wait 处理 | 🔲 待开发 |
| **中** | Guard 中间件 | `only_admin()`、`only_private()`、`only_group()` 权限守卫 | 🔲 待开发 |
| **中** | 对话/向导 | 支持分支逻辑的多步对话流程 | 🔲 待开发 |
| **中** | 菜单系统 | 支持分页的交互式内联按钮菜单 | 🔲 待开发 |
| **中** | 忽略旧更新 | 跳过超过 N 秒的过期更新 | 🔲 待开发 |
| **低** | 国际化支持 | 多语言/本地化辅助工具 | 🔲 待开发 |
| **低** | 指标/日志 | 结构化日志和更新处理指标 | 🔲 待开发 |
| **低** | 机器人命令菜单 | 通过 `setMyCommands` 自动向 Telegram 注册命令 | 🔲 待开发 |

> 灵感来自主流框架：[teloxide](https://github.com/teloxide/teloxide)、[grammY](https://grammy.dev/)、[python-telegram-bot](https://python-telegram-bot.org/)

## 快速开始

```rust
use serde::{Deserialize, Serialize};
use tgbot_worker_rs::prelude::*;
use tgbot_worker_rs::session::{Context, KvStorage};
use worker::*;

#[derive(Default, Clone, Serialize, Deserialize)]
struct MyState {
    counter: u32,
}

type Ctx = Context<MyState, KvStorage>;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    let mut app = App::new();
    let storage = KvStorage::from_env(&env, "SESSION_KV", "session")?;

    // 带会话的简单命令处理器
    app.on_command_ctx::<MyState, _, _, _>("count", storage.clone(), |ctx| async move {
        let mut state = ctx.session.get();
        state.counter += 1;
        ctx.session.set(state.clone());
        ctx.reply_and_done(&format!("计数: {}", state.counter)).await
    });

    // 处理所有文本消息
    app.on_update_ctx::<MyState, _, _, _>(storage, |ctx| async move {
        match ctx.text() {
            Some(text) if !text.starts_with('/') => {
                ctx.reply_and_done(&format!("你说: {}", text)).await
            }
            _ => Ctx::skip(), // 不是文本消息，跳到下一个处理器
        }
    });

    app.on_fetch(req, env, ctx).await.map_err(|e| e.into())
}
```

## API 概览

### Context 方法

| 方法 | 描述 |
|------|------|
| `ctx.reply(text)` | 发送文本消息 |
| `ctx.reply_html(text)` | 发送 HTML 格式消息 |
| `ctx.reply_to(text)` | 回复当前消息（引用） |
| `ctx.reply_and_done(text)` | 回复并结束处理 |
| `ctx.edit_text(text)` | 编辑消息文本 |
| `ctx.delete_message()` | 删除当前消息 |
| `ctx.answer_callback(text, show_alert)` | 响应回调查询 |
| `ctx.send_photo(photo)` | 发送图片 |
| `Context::done()` | 结束处理器处理 |
| `Context::skip()` | 跳到下一个处理器 |

### 访问器方法

| 方法 | 描述 |
|------|------|
| `ctx.chat_id()` | 获取聊天 ID |
| `ctx.user_id()` | 获取用户 ID |
| `ctx.message_id()` | 获取消息 ID |
| `ctx.text()` | 获取消息文本 |
| `ctx.command()` | 获取命令名（不含 `/`） |
| `ctx.command_args()` | 获取命令参数 |
| `ctx.callback_data()` | 获取回调查询数据 |
| `ctx.telegram_api()` | 获取原始 Telegram API 客户端 |

## 会话管理

会话按聊天自动加载和保存。使用 KV 存储进行持久化：

```rust
// 定义状态类型
#[derive(Default, Clone, Serialize, Deserialize)]
struct UserState {
    step: String,
    data: Option<String>,
}

// 从 KV 绑定创建存储
let storage = KvStorage::from_env(&env, "SESSION_KV", "prefix")?;

// 在处理器中访问会话
app.on_command_ctx::<UserState, _, _, _>("start", storage, |ctx| async move {
    ctx.session.set(UserState {
        step: "awaiting_input".into(),
        data: None,
    });
    ctx.reply_and_done("请输入你的名字：").await
});
```

## 过滤器组合

使用过滤器配合 `on_update_when` 或在处理器中检查条件：

```rust
use tgbot_worker_rs::filter::*;

// 可用过滤器
is_message(&update)           // 是消息
is_callback_query(&update)    // 是回调查询
has_text(&update)             // 有文本内容
is_command(&update)           // 是命令（以 / 开头）
text_contains("hello")        // 文本包含子串
text_starts_with("hi")        // 文本以前缀开头
callback_data_equals("btn1")  // 回调数据匹配
from_chat(chat_id)            // 来自特定聊天
from_user(user_id)            // 来自特定用户

// 组合器
and(is_message, has_text)     // 两个条件都满足
or(is_message, is_callback_query)  // 任一条件满足
not(is_command)               // 取反过滤器
```

## 示例

请参考以下示例：

- `examples/version`：命令路由、KV、D1 与消息队列集成。[说明](examples/version/README.MD)
- `examples/middleware`：中间件使用和回复消息。[说明](examples/middleware/README.MD)
- `examples/session`：多步骤注册流程与会话状态。

### 运行示例

```bash
# 安装工具链
rustup toolchain install 1.89.0
rustup target add wasm32-unknown-unknown --toolchain 1.89.0

# 安装 Wrangler
npm i -g wrangler

# 本地运行示例
cd examples/session
wrangler secret put API_KEY  # 你的 Telegram 机器人 Token
wrangler dev

# 部署
wrangler publish

# 设置 webhook
curl "https://api.telegram.org/bot<TOKEN>/setWebhook?url=<WORKER_URL>/telegramMessage"
```

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
