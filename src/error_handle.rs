use worker::*;

pub async fn on_error(e: Error) {
    console_log!("Error: {:?}", e);
}
