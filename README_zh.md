# Telegram Bot Worker (Rust)

![构建状态](https://img.shields.io/badge/build-passing-brightgreen)
![许可证](https://img.shields.io/badge/license-WTFPL-blue)
![版本](https://img.shields.io/badge/version-0.4.0-orange)

轻量级、无服务器的 Telegram 机器人框架，专为 Cloudflare Workers 设计，使用 Rust 构建。

[View English Description](README.md)

## 目录

- [功能特性](#功能特性)
- [开发路线图](#开发路线图)
- [快速开始](#快速开始)
- [API 概览](#api-概览)
- [过滤器组合](#过滤器组合)
- [键盘构建器](#键盘构建器)
- [命令解析](#命令解析)
- [dptree 风格处理器](#dptree-风格处理器)
- [对话/FSM 系统](#对话fsm-系统)
- [重试工具](#重试工具)
- [示例](#示例)
- [贡献](#贡献)
- [许可证](#许可证)

## 功能特性

- **无服务器优先**：专为 Cloudflare Workers 设计，零冷启动开销
- **teloxide 风格 API**：简洁的处理器签名 `|bot, msg| async { bot.send_message(...) }`
- **会话管理**：内置基于 KV 的会话存储，支持类型安全的状态管理
- **过滤器组合**：可组合的过滤器如 `is_message`、`text_contains`、`callback_data_equals`
- **丰富的 Telegram 方法**：`reply`、`reply_html`、`edit_text`、`delete_message`、`answer_callback`、`send_photo`
- **中间件支持**：请求/响应管道，支持短路返回
- **类型安全**：利用 Rust 类型系统，提供 `BotError` 和 `BotResult`

**项目状态：** 活跃开发中，欢迎贡献！

注意：本项目固定目标为 `wasm32-unknown-unknown`（通过 `.cargo/config.toml` 设置）。请先安装目标：`rustup target add wasm32-unknown-unknown`，并优先使用固定工具链版本（例如 `+1.91.1`）运行相关命令。

## 开发路线图

计划中的功能和改进（欢迎贡献！）：

| 优先级 | 功能 | 描述 | 状态 |
|--------|------|------|------|
| **高** | 键盘构建器 | 类型安全的内联/回复键盘构建 API | ✅ 已完成 |
| **高** | 命令参数解析 | 结构化解析：`/remind 30m "文本"` → `(Duration, String)` | ✅ 已完成 |
| **高** | 速率限制 | 自动重试 + 指数退避 + flood wait 处理 | ✅ 已完成 |
| **高** | dptree 处理器 | 受 teloxide dptree 启发的可组合处理器链 | ✅ 已完成 |
| **高** | 对话/FSM | 支持状态持久化的多步对话流程 | ✅ 已完成 |
| **中** | Guard 中间件 | `only_admin()`、`only_private()`、`only_group()` 权限守卫 | 🔲 待开发 |
| **中** | 菜单系统 | 支持分页的交互式内联按钮菜单 | 🔲 待开发 |
| **中** | 忽略旧更新 | 跳过超过 N 秒的过期更新 | 🔲 待开发 |
| **低** | 国际化支持 | 多语言/本地化辅助工具 | 🔲 待开发 |
| **低** | 指标/日志 | 结构化日志和更新处理指标 | 🔲 待开发 |
| **低** | 机器人命令菜单 | 通过 `setMyCommands` 自动向 Telegram 注册命令 | 🔲 待开发 |

> 灵感来自主流框架：[teloxide](https://github.com/teloxide/teloxide)、[grammY](https://grammy.dev/)、[python-telegram-bot](https://python-telegram-bot.org/)

## 快速开始

### 简单 API（teloxide 风格）

```rust
use tgbot_worker_rs::prelude::*;
use worker::*;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut app = App::new();

    // 处理 /start 命令
    app.command("start", |bot, msg| async move {
        bot.send_message(msg.chat_id(), "你好！我是机器人。").await
    });

    // 处理 /echo <文本> 命令
    app.command("echo", |bot, msg| async move {
        let text = msg.command_args().unwrap_or("无内容");
        bot.send_message(msg.chat_id(), &format!("回显: {}", text)).await
    });

    // 处理回调查询
    app.on_callback_query(|bot, query| async move {
        bot.answer_callback(query.id(), Some("已点击！"), false).await
    });

    // 其他消息的回退处理
    app.on_message(|bot, msg| async move {
        // 跳过命令（已在上面处理）
        if msg.text().map(|t| t.starts_with('/')).unwrap_or(false) {
            return Err(BotError::Skip);
        }
        bot.send_message(msg.chat_id(), "发送 /start 开始").await
    });

    app.run(req, env, ctx).await
}
```

## API 概览

### 简单 API（App 方法）

| 方法 | 描述 |
|------|------|
| `app.command("cmd", \|bot, msg\|)` | 处理 `/cmd` 命令 |
| `app.on_message(\|bot, msg\|)` | 处理所有消息 |
| `app.on_callback_query(\|bot, query\|)` | 处理回调查询 |
| `app.run(req, env, ctx)` | 运行机器人 |

### Bot 方法

| 方法 | 描述 |
|------|------|
| `bot.send_message(chat_id, text)` | 发送文本消息 |
| `bot.send_html(chat_id, text)` | 发送 HTML 格式消息 |
| `bot.reply(&msg, text)` | 回复消息（引用） |
| `bot.reply_html(&msg, text)` | 回复 HTML 格式消息 |
| `bot.answer_callback(id, text, alert)` | 响应回调查询 |
| `bot.edit_message(chat_id, msg_id, text)` | 编辑消息文本 |
| `bot.delete_message(chat_id, msg_id)` | 删除消息 |
| `bot.send_photo(chat_id, photo)` | 发送图片 |

### 管理员方法

| 方法 | 描述 |
|------|------|
| `bot.ban_chat_member(chat_id, user_id, until_date, revoke)` | 封禁用户 |
| `bot.unban_chat_member(chat_id, user_id, only_if_banned)` | 解封用户 |
| `bot.kick_chat_member(chat_id, user_id)` | 踢出用户（封禁后立即解封） |
| `bot.mute_chat_member(chat_id, user_id, until_date)` | 禁言用户 |
| `bot.unmute_chat_member(chat_id, user_id)` | 解除禁言 |
| `bot.restrict_chat_member(chat_id, user_id, perms, until)` | 限制权限 |
| `bot.promote_chat_member(chat_id, user_id, rights)` | 提升为管理员 |
| `bot.pin_message(chat_id, msg_id, silent)` | 置顶消息 |
| `bot.unpin_message(chat_id, msg_id)` | 取消置顶 |
| `bot.unpin_all_messages(chat_id)` | 取消所有置顶 |
| `bot.set_chat_title(chat_id, title)` | 设置群标题 |
| `bot.set_chat_description(chat_id, desc)` | 设置群描述 |

### Message 访问器

| 方法 | 描述 |
|------|------|
| `msg.chat_id()` | 获取聊天 ID |
| `msg.message_id()` | 获取消息 ID |
| `msg.text()` | 获取消息文本 |
| `msg.from()` | 获取发送者 User |
| `msg.command()` | 获取命令名（不含 `/`） |
| `msg.command_args()` | 获取命令参数 |

### CallbackQuery 访问器

| 方法 | 描述 |
|------|------|
| `query.id()` | 获取回调查询 ID |
| `query.data()` | 获取回调数据 |
| `query.from()` | 获取点击用户 |
| `query.chat_id()` | 获取聊天 ID |
| `query.message_id()` | 获取消息 ID |

## 过滤器组合

使用过滤器配合处理器检查条件：

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

## 键盘构建器

使用流式 API 构建内联和回复键盘：

```rust
use tgbot_worker_rs::prelude::*;

// 内联键盘（消息下方的按钮）
let keyboard = InlineKeyboard::new()
    .row([
        InlineButton::callback("是", "yes"),
        InlineButton::callback("否", "no"),
    ])
    .button(InlineButton::url("访问", "https://example.com"));

bot.send_inline_keyboard(chat_id, "请选择：", keyboard).await?;

// 回复键盘（替换默认键盘的自定义键盘）
let keyboard = ReplyKeyboard::new()
    .text("选项 1")
    .text("选项 2")
    .resize()
    .one_time();

bot.send_reply_keyboard(chat_id, "选择：", keyboard).await?;
```

## 命令解析

类型安全地解析命令参数：

```rust
use tgbot_worker_rs::prelude::*;

// 解析 "/ban 123 1h 垃圾信息"
let parser = CommandParser::new(msg.text().unwrap_or(""));
if parser.is_command("ban") {
    let user_id: u64 = parser.arg(0)?;      // 123
    let duration: String = parser.arg(1)?;   // "1h"
    let reason = parser.rest(2);             // Some("垃圾信息")
}

// 解析时长字符串
use tgbot_worker_rs::command::parse_duration;
let seconds = parse_duration("30m")?;  // 1800
let seconds = parse_duration("1h")?;   // 3600
let seconds = parse_duration("1d")?;   // 86400
```

## dptree 风格处理器

构建受 [teloxide dptree](https://github.com/teloxide/dptree) 启发的可组合处理器链：

```rust
use tgbot_worker_rs::dptree;

// 使用 chain() 和 branch() 组合处理器
let handler = dptree::entry()
    // 首先尝试 /start 命令
    .branch(
        dptree::filter_command("start")
            .chain(dptree::endpoint(handle_start))
    )
    // 尝试 /help 命令
    .branch(
        dptree::filter_command("help")
            .chain(dptree::endpoint(handle_help))
    )
    // 处理回调查询
    .branch(
        dptree::filter(|upd| matches!(upd.content, UpdateContent::CallbackQuery(_)))
            .chain(dptree::endpoint_callback(handle_callback))
    )
    // 其他消息的回退处理
    .branch(dptree::endpoint(handle_fallback));

async fn handle_start(bot: Bot, msg: Message) -> BotResult<()> {
    bot.send_message(msg.chat_id(), "欢迎！").await
}
```

处理器控制流：
- `Continue` → 继续执行链中的下一个处理器
- `Skip` → 尝试下一个分支
- `Break(Response)` → 停止处理

## 对话/FSM 系统

构建支持状态持久化的多步对话流程：

```rust
use tgbot_worker_rs::dialogue::Dialogue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
enum RegState {
    #[default]
    Start,
    AwaitingName,
    AwaitingEmail { name: String },
}

async fn handle_registration(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<RegState, KvStorage>,
) -> BotResult<()> {
    // 从存储加载状态
    dialogue.load().await.ok();

    match dialogue.get() {
        RegState::Start => {
            bot.send_message(msg.chat_id(), "你叫什么名字？").await?;
            dialogue.update(RegState::AwaitingName);
        }
        RegState::AwaitingName => {
            let name = msg.text().unwrap_or("").to_string();
            bot.send_message(msg.chat_id(), "你的邮箱是？").await?;
            dialogue.update(RegState::AwaitingEmail { name });
        }
        RegState::AwaitingEmail { name } => {
            let email = msg.text().unwrap_or("");
            bot.send_message(
                msg.chat_id(),
                &format!("完成！{} <{}>", name, email)
            ).await?;
            dialogue.exit().await.ok();
        }
    }
    dialogue.save().await.ok();
    Ok(())
}
```

## 重试工具

检测可重试错误并提取重试时间：

```rust
use tgbot_worker_rs::retry::{is_retryable, get_retry_after};

match bot.send_message(chat_id, "你好").await {
    Ok(_) => { /* 成功 */ }
    Err(e) => {
        let err = e.to_string();
        if is_retryable(&err) {
            // 获取 Telegram 建议的延迟时间（针对 429 错误）
            let delay = get_retry_after(&err).unwrap_or(30);
            // 按需处理重试
        }
    }
}
```

## 示例

请参考以下示例：

- `examples/version`：命令路由、KV、D1 与消息队列集成。[说明](examples/version/README.MD)
- `examples/middleware`：中间件使用和回复消息。[说明](examples/middleware/README.MD)
- `examples/session`：多步骤注册流程与会话状态。

### 运行示例

```bash
# 安装工具链
rustup toolchain install 1.91.1
rustup target add wasm32-unknown-unknown --toolchain 1.91.1

# 安装 Wrangler
npm i -g wrangler

# 本地运行示例
cd examples/version
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

## 致谢

本项目深受 [teloxide](https://github.com/teloxide/teloxide) 的启发，这是一个优雅的 Rust Telegram 机器人框架。许多 API 设计，包括简化的处理器签名、键盘构建器和命令解析模式，都参考了 teloxide 的优秀架构。

特别感谢 teloxide 团队创建了如此精心设计的框架，为 Rust Telegram 机器人生态系统提供了参考。

## 许可证

该项目根据 WTFPL 许可证进行许可 - 详见 [LICENSE](LICENSE) 文件。
