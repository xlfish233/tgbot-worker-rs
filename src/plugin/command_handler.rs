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

pub async fn handle_command(command: Command, chat_id: i64, env: &crate::state::AppState) {
    match command {
        Command::Version => {
            let version_info = crate::state::AppState::version();
            let reply_params = frankenstein::SendMessageParams {
                business_connection_id: None,
                chat_id: frankenstein::ChatId::Integer(chat_id),
                message_thread_id: None,
                text: format!("Current version: {}", version_info),
                parse_mode: None,
                entities: None,
                link_preview_options: None,
                disable_notification: None,
                protect_content: None,
                message_effect_id: None,
                reply_parameters: None,
                reply_markup: None,
            };

            let _ = crate::service::TelegramService::send_message(&reply_params, env).await;
        }
        // 处理其他命令
    }
}
