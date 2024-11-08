use frankenstein::SendMessageParams;
use worker::{console_error, console_log};

pub enum Command {
    Version,
    // 在这里可以添加其他命令
}

impl Command {
    pub fn from_text(text: &str) -> Option<Self> {
        match text {
            "/version" => Some(Command::Version),
            // 添加其他命令匹配
            _ => None,
        }
    }
}

#[worker::send]
pub async fn handle_command(command: Command, chat_id: i64, env: &crate::state::AppState) {
    match command {
        Command::Version => {
            let version_info = crate::state::AppState::version();
            let reply = SendMessageParams::builder()
                .chat_id(chat_id)
                .text(format!("Current version: {}", version_info))
                .build();

            match crate::service::TelegramService::send_message(&reply, env).await {
                Err(e) => {
                    console_error!("Error sending message: {:?}", e);
                }
                Ok(r) => {
                    console_log!("Message sent: {:?}", r);
                }
            }
        }
    }
}
