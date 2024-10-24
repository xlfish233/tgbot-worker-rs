use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use worker::console_error;

// 定义错误处理
#[derive(Debug)]
pub enum ApiError {
    ServiceError(String),
    // Removed JsonError since it was never constructed
}

// 实现 IntoResponse
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::ServiceError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = json!({
            "status": "error",
            "code": status.as_u16(),
            "message": "Error occurred",
            "data": error_message,
        });

        (status, Json(body)).into_response()
    }
}

// Helper function to create success response
pub fn success_response<T: serde::Serialize>(data: T) -> axum::response::Response {
    let body = json!({
        "status": "success",
        "data": data,
    });
    (StatusCode::OK, Json(body)).into_response()
}

// Helper function to create error response
pub fn error_response<E: std::fmt::Debug>(e: E) -> axum::response::Response {
    console_error!("Error with backtrace: {:#?}", e);
    ApiError::ServiceError(format!("{:#?}", e)).into_response()
}
