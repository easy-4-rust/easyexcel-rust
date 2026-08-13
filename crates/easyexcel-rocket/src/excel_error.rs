use easyexcel_web::ExcelWebError;
use rocket::Request;
use rocket::http::{ContentType, Status};
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

/// Rocket data guard/responder 的统一错误。
#[derive(Debug)]
pub struct ExcelRocketError {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelRocketError {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }

    /// 返回 Rocket 状态码。
    #[must_use]
    pub fn status(&self) -> Status {
        Status::new(self.error.status_code().as_u16())
    }
}

impl std::fmt::Display for ExcelRocketError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ExcelRocketError {}

impl<'r> Responder<'r, 'static> for ExcelRocketError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status();
        let bytes = serde_json::to_vec(&self.error.problem_details(self.request_id))
            .unwrap_or_else(|_| b"{}".to_vec());
        Response::build()
            .status(status)
            .header(ContentType::new("application", "problem+json"))
            .sized_body(bytes.len(), Cursor::new(bytes))
            .ok()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_wraps_error() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelRocketError::new(error, "req-1");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn status_returns_correct_code() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelRocketError::new(error, "req-2");
        // Cancelled maps to 408
        assert_eq!(err.status().code, 408);
    }

    #[test]
    fn display_delegates() {
        let error = easyexcel_web::ExcelWebError::ProcessingTimeout;
        let err = ExcelRocketError::new(error, "req-3");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn error_trait() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelRocketError::new(error, "req-4");
        let dyn_err: &dyn std::error::Error = &err;
        assert!(!dyn_err.to_string().is_empty());
    }

    #[test]
    fn debug_contains_struct_name() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelRocketError::new(error, "req");
        assert!(format!("{err:?}").contains("ExcelRocketError"));
    }
}
