use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use easyexcel_web::ExcelWebError;

/// Actix Web extractor/responder 的统一错误。
#[derive(Debug)]
pub struct ExcelActixError {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelActixError {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }

    /// 返回 Web 内核错误。
    #[must_use]
    pub const fn error(&self) -> &ExcelWebError {
        &self.error
    }
}

impl std::fmt::Display for ExcelActixError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ExcelActixError {}

impl ResponseError for ExcelActixError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.error.status_code().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.error.problem_details(&self.request_id))
    }
}
