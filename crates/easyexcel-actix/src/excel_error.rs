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

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_wraps_error() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req-1");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn error_returns_reference() {
        let error = easyexcel_web::ExcelWebError::ProcessingTimeout;
        let err = ExcelActixError::new(error, "req-2");
        assert_eq!(
            err.error().code(),
            easyexcel_web::ExcelWebErrorCode::ProcessingTimeout
        );
    }

    #[test]
    fn display_delegates() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req-3");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn error_trait() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req-4");
        let dyn_err: &dyn std::error::Error = &err;
        assert!(!dyn_err.to_string().is_empty());
    }

    #[test]
    fn status_code_returns_correct_value() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req-5");
        assert_eq!(err.status_code(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn error_response_returns_json() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req-6");
        let response = err.error_response();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn debug_contains_struct_name() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelActixError::new(error, "req");
        assert!(format!("{err:?}").contains("ExcelActixError"));
    }
}
