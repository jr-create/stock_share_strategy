use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use askama::Template;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "500.html")]
struct InternalErrorTemplate {
    logged_in: bool,
    username: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, msg.clone())
            }
        };

        // 根据状态码返回对应的 HTML 错误页面
        match status {
            StatusCode::NOT_FOUND => {
                let template = NotFoundTemplate {
                    logged_in: false,
                    username: String::new(),
                };
                (status, Html(template.render().unwrap())).into_response()
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                let template = InternalErrorTemplate {
                    logged_in: false,
                    username: String::new(),
                };
                (status, Html(template.render().unwrap())).into_response()
            }
            _ => {
                // 其他错误返回简单的文本响应
                (status, message).into_response()
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
