//! Session example: Multi-step registration flow using KV storage
//!
//! Commands:
//! - /start - Start the registration flow
//! - /cancel - Cancel the current registration
//! - /status - Show current session state

use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};
use tgbot_worker_rs::session::KvStorage;
use tgbot_worker_rs::App;
use worker::*;

#[derive(Default, Clone, Serialize, Deserialize)]
struct RegistrationState {
    step: Step,
    name: Option<String>,
    age: Option<u8>,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
enum Step {
    #[default]
    Idle,
    AwaitName,
    AwaitAge,
    Complete,
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut app = App::new();

    let storage = match KvStorage::from_env(&env, "SESSION_KV", "session") {
        Ok(s) => s,
        Err(e) => return Response::error(format!("KV binding error: {}", e), 500),
    };

    // /start - Begin registration
    app.on_command_ctx::<RegistrationState, _, _, _>("start", storage.clone(), |ctx| async move {
        ctx.session.set(RegistrationState {
            step: Step::AwaitName,
            ..Default::default()
        });
        ctx.reply("Welcome! Let's start registration.\n\nWhat is your name?")
            .await?;
        Ok(ControlFlow::Break(Response::ok("")?))
    });

    // /cancel - Cancel registration
    app.on_command_ctx::<RegistrationState, _, _, _>("cancel", storage.clone(), |ctx| async move {
        let state = ctx.session.get();
        if state.step == Step::Idle {
            ctx.reply("Nothing to cancel.").await?;
        } else {
            ctx.session.set(RegistrationState::default());
            ctx.reply("Registration cancelled.").await?;
        }
        Ok(ControlFlow::Break(Response::ok("")?))
    });

    // /status - Show current state
    app.on_command_ctx::<RegistrationState, _, _, _>("status", storage.clone(), |ctx| async move {
        let state = ctx.session.get();
        let status = match state.step {
            Step::Idle => "No active registration".to_string(),
            Step::AwaitName => "Waiting for your name".to_string(),
            Step::AwaitAge => format!(
                "Name: {}\nWaiting for your age",
                state.name.as_deref().unwrap_or("?")
            ),
            Step::Complete => format!(
                "Registration complete!\nName: {}\nAge: {}",
                state.name.as_deref().unwrap_or("?"),
                state.age.map(|a| a.to_string()).unwrap_or("?".to_string())
            ),
        };
        ctx.reply(&status).await?;
        Ok(ControlFlow::Break(Response::ok("")?))
    });

    // Handle text messages for the registration flow
    app.on_update_ctx::<RegistrationState, _, _, _>(storage.clone(), |ctx| async move {
        let text = match ctx.text() {
            Some(t) if !t.starts_with('/') => t.to_string(),
            _ => return Ok(ControlFlow::Continue(())),
        };

        let mut state = ctx.session.get();

        match state.step {
            Step::AwaitName => {
                state.name = Some(text.clone());
                state.step = Step::AwaitAge;
                ctx.session.set(state);
                ctx.reply(&format!("Nice to meet you, {}!\n\nHow old are you?", text))
                    .await?;
                Ok(ControlFlow::Break(Response::ok("")?))
            }
            Step::AwaitAge => {
                match text.parse::<u8>() {
                    Ok(age) if age > 0 && age < 150 => {
                        state.age = Some(age);
                        state.step = Step::Complete;
                        ctx.session.set(state.clone());
                        ctx.reply(&format!(
                            "Registration complete!\n\nName: {}\nAge: {}\n\nUse /start to register again.",
                            state.name.as_deref().unwrap_or("?"),
                            age
                        ))
                        .await?;
                    }
                    _ => {
                        ctx.reply("Please enter a valid age (1-149).").await?;
                    }
                }
                Ok(ControlFlow::Break(Response::ok("")?))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    });

    app.on_fetch(req, env, ctx)
        .await
        .map_err(|e| worker::Error::from(e.to_string()))
}
