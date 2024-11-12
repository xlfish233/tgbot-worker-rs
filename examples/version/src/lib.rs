use frankenstein::{AsyncTelegramApi, SendMessageParams};
use worker::*;
use tgbot_worker_rs::{App, AnyhowResult};
use frankenstein::objects::*;


#[event(fetch)]
pub async fn fetch(
    req: HttpRequest,
    env: Env,
    ctx: Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    let mut app = App::new();
    app.set_on_update(handle_update_echo);
    app.on_fetch(req, env, ctx).await.map_err(|e| worker::Error::from(e.to_string()))
}

#[worker::send]
async fn handle_update_echo(update: Update, _env: Env) -> AnyhowResult<()> {
    let tg_api =tgbot_worker_rs::service::get_cli_from_env(&_env)?;
    match update.content {
        UpdateContent::Message(message) => {

            if let Some(text) = message.text {
                if text == "/version" {
                    let response = format!("tgbot-worker-rs version: 0.1.0");
                    let reply = SendMessageParams::builder()
                       .chat_id(message.chat.id)
                       .text(response)
                       .build();
                    tg_api.send_message(&reply).await?;
                }

            }
        }
        _ => {}
    }
    Ok(())
}



#[event(scheduled)]
pub async fn scheduled(
    event: ScheduledEvent,
    env: Env,
    _ctx: ScheduleContext,
) {
    console_log!("Scheduled event: {:?}", event);
}