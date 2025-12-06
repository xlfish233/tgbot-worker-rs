//! Session example: Multi-step registration flow using Dialogue/FSM
//!
//! Commands:
//! - /start - Start the registration flow
//! - /cancel - Cancel the current registration
//! - /status - Show current session state
//!
//! This example demonstrates the Dialogue API for multi-step conversations.

use serde::{Deserialize, Serialize};
use tgbot_worker_rs::dialogue::Dialogue;
use tgbot_worker_rs::prelude::*;
use worker::*;

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
enum RegState {
    #[default]
    Idle,
    AwaitName,
    AwaitAge {
        name: String,
    },
    Complete {
        name: String,
        age: u8,
    },
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // /start - Begin registration
    app.command("start", |bot, msg| async move {
        let storage = KvStorage::from_env(&bot.env(), "SESSION_KV", "dialogue")?;
        let dialogue = Dialogue::<RegState, _>::new(storage, msg.chat_id());
        dialogue.update(RegState::AwaitName);
        dialogue.save().await.ok();
        bot.send_message(msg.chat_id(), "Welcome! Let's start registration.\n\nWhat is your name?").await
    });

    // /cancel - Cancel registration
    app.command("cancel", |bot, msg| async move {
        let storage = KvStorage::from_env(&bot.env(), "SESSION_KV", "dialogue")?;
        let dialogue = Dialogue::<RegState, _>::new(storage, msg.chat_id());
        dialogue.load().await.ok();

        if dialogue.get() == RegState::Idle {
            bot.send_message(msg.chat_id(), "Nothing to cancel.").await
        } else {
            dialogue.exit().await.ok();
            bot.send_message(msg.chat_id(), "Registration cancelled.").await
        }
    });

    // /status - Show current state
    app.command("status", |bot, msg| async move {
        let storage = KvStorage::from_env(&bot.env(), "SESSION_KV", "dialogue")?;
        let dialogue = Dialogue::<RegState, _>::new(storage, msg.chat_id());
        dialogue.load().await.ok();

        let status = match dialogue.get() {
            RegState::Idle => "No active registration".to_string(),
            RegState::AwaitName => "Waiting for your name".to_string(),
            RegState::AwaitAge { name } => format!("Name: {}\nWaiting for your age", name),
            RegState::Complete { name, age } => {
                format!("Registration complete!\nName: {}\nAge: {}", name, age)
            }
        };
        bot.send_message(msg.chat_id(), &status).await
    });

    // Handle text messages for the registration flow
    app.on_message(|bot, msg| async move {
        // Skip commands
        let text = match msg.text() {
            Some(t) if !t.starts_with('/') => t.to_string(),
            _ => return Err(BotError::Skip),
        };

        let storage = KvStorage::from_env(&bot.env(), "SESSION_KV", "dialogue")?;
        let dialogue = Dialogue::<RegState, _>::new(storage, msg.chat_id());
        dialogue.load().await.ok();

        match dialogue.get() {
            RegState::AwaitName => {
                dialogue.update(RegState::AwaitAge { name: text.clone() });
                dialogue.save().await.ok();
                bot.send_message(msg.chat_id(), &format!("Nice to meet you, {}!\n\nHow old are you?", text)).await
            }
            RegState::AwaitAge { name } => {
                match text.parse::<u8>() {
                    Ok(age) if age > 0 && age < 150 => {
                        dialogue.update(RegState::Complete { name: name.clone(), age });
                        dialogue.save().await.ok();
                        bot.send_message(
                            msg.chat_id(),
                            &format!("Registration complete!\n\nName: {}\nAge: {}\n\nUse /start to register again.", name, age)
                        ).await
                    }
                    _ => bot.send_message(msg.chat_id(), "Please enter a valid age (1-149).").await,
                }
            }
            _ => Err(BotError::Skip), // Not in registration, skip
        }
    });

    app.run(req, env, ctx).await
}
